//! HTTP page download: destinations validated on every hop, a size cap, HTML only and decoding
//! by the declared charset. Page favicons come only from public destinations, with a cap and the
//! type sniffed from their bytes.

use crate::blocked::{BlockSignals, is_blocked};
use crate::msg::{Failure, Msg, MsgExt};
use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use quick_xml::events::Event;
use quick_xml::name::{Namespace, ResolveResult};
use quick_xml::{NsReader, XmlVersion};
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use ureq::unversioned::resolver::{DefaultResolver, ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::{DefaultConnector, NextTimeout};
use url::Url;

pub const MAX_REDIRECTS: usize = 10;
pub const MAX_HTML_BYTES: u64 = 8 * 1024 * 1024;
/// Per-request (per-hop) timeout, not for the whole download: without it a hung server blocks
/// the worker forever.
/// shortcut: the timeout is per hop, up to 11 × 30 s with `MAX_REDIRECTS`; switch to a single
/// deadline if redirect chains stall the worker.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(30);
const USER_AGENT: &str = concat!(
    "marks/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/landygg/marks)"
);
const SCHEMES: [&str; 2] = ["http", "https"];
const LOCALHOST: &str = "localhost";
/// `<meta charset>` is only looked for at the start of the document.
const META_SCAN_BYTES: usize = 1_024;
const HTML_TYPES: [&str; 2] = ["text/html", "application/xhtml+xml"];
const REDIRECT_STATUSES: [u16; 5] = [301, 302, 303, 307, 308];
pub const MAX_FAVICON_BYTES: u64 = 256 * 1024;
/// Deadline for a whole favicon download, redirects included.
const FAVICON_TIMEOUT: Duration = Duration::from_secs(15);
/// Type, offset and signatures of each `sniff_image` format.
const IMAGE_MAGIC: [(&str, usize, &[&[u8]]); 6] = [
    ("image/png", 0, &[b"\x89PNG\r\n\x1a\n"]),
    ("image/jpeg", 0, &[b"\xff\xd8\xff"]),
    ("image/gif", 0, &[b"GIF87a", b"GIF89a"]),
    (WEBP_MIME, 0, &[b"RIFF"]),
    ("image/x-icon", 0, &[b"\x00\x00\x01\x00"]),
    ("image/avif", 4, &[b"ftypavif", b"ftypavis"]),
];
/// WebP: `RIFF`, the size, and `WEBP` at byte 8.
const WEBP_MIME: &str = "image/webp";
const WEBP_OFFSET: usize = 8;
const WEBP_TAG: &[u8] = b"WEBP";
/// SVG: XML with an `<svg>` root in this namespace (`is_svg`).
const SVG_MIME: &str = "image/svg+xml";
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
const SVG_ROOT: &str = "svg";
const UTF8_BOM: &str = "\u{feff}";
const UTF8: &str = "utf-8";
/// Inert inside `<img>`, but a favicon has no use for them: rejected just in case.
const SVG_ACTIVE_ELEMENTS: [&str; 2] = ["script", "foreignObject"];
const EVENT_ATTRIBUTE_PREFIX: &str = "on";
/// The only named entities available without a DOCTYPE.
const XML_ENTITIES: [&str; 5] = ["lt", "gt", "amp", "apos", "quot"];
/// Responses that usually clear up on their own: too many requests (and 5xx, `SERVER_ERRORS`).
const TOO_MANY_REQUESTS: u16 = 429;
const SERVER_ERRORS: u16 = 500;
/// CGNAT, `100.64.0.0/10`: carrier-shared addresses, not Internet ones.
const CGNAT_FIRST: u8 = 100;
const CGNAT_SECOND: std::ops::Range<u8> = 64..128;

pub struct Download {
    pub final_url: String,
    pub html: String,
}

/// `true` only for http(s) whose host is neither `localhost` nor a loopback IP.
pub fn is_remote_http(url: &Url) -> bool {
    let Some(host) = url.host_str().filter(|_| SCHEMES.contains(&url.scheme())) else {
        return false;
    };
    // `host_str` wraps IPv6 addresses in brackets.
    let host = host.trim_start_matches('[').trim_end_matches(']');
    match host.parse::<IpAddr>() {
        Ok(ip) => !is_local_ip(ip),
        // `localhost.` with a trailing dot resolves the same as `localhost`.
        Err(_) => !is_localhost_name(host.trim_end_matches('.')),
    }
}

/// `localhost` or a subdomain of it, which WebKit also resolves to loopback.
fn is_localhost_name(host: &str) -> bool {
    host == LOCALHOST || host.ends_with(&format!(".{LOCALHOST}"))
}

/// The IP reaches this machine: loopback or unspecified (`0.0.0.0` connects locally), also when
/// embedded in an IPv6 address (mapped `::ffff:a.b.c.d` or compatible `::a.b.c.d`).
fn is_local_ip(ip: IpAddr) -> bool {
    let is_local = |ip: IpAddr| ip.is_loopback() || ip.is_unspecified();
    // Check the original too: `to_ipv4` turns `::1` into `0.0.0.1`, which is not loopback.
    is_local(ip)
        || matches!(ip, IpAddr::V6(v6) if v6.to_ipv4().is_some_and(|v4| is_local(IpAddr::V4(v4))))
}

/// Resolves `location` (absolute or relative) against `current` and checks the destination and the hop limit.
pub fn next_hop(current: &Url, location: &str, hops: usize) -> Result<Url> {
    if hops >= MAX_REDIRECTS {
        return Err(Msg::TooManyRedirects.err());
    }
    let next = current.join(location).msg(Msg::Destination)?;
    if !is_remote_http(&next) {
        return Err(Msg::Destination.err());
    }
    Ok(next)
}

/// `text/html` or `application/xhtml+xml`, ignoring parameters and case.
pub fn is_html(content_type: Option<&str>) -> bool {
    content_type
        .and_then(|ct| ct.split(';').next())
        .is_some_and(|mime| HTML_TYPES.contains(&mime.trim().to_ascii_lowercase().as_str()))
}

fn header_charset(content_type: &str) -> Option<&str> {
    content_type.split(';').skip(1).find_map(|param| {
        let (key, value) = param.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case("charset")
            .then(|| value.trim().trim_matches(['"', '\'']))
    })
}

/// Charset from a leading `<meta charset=…>` or `<meta http-equiv … content="…; charset=…">`.
fn meta_charset(bytes: &[u8]) -> Option<String> {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(META_SCAN_BYTES)]).to_lowercase();
    let rest = &head[head.find("charset")? + "charset".len()..];
    let rest = rest.trim_start().strip_prefix('=')?.trim_start();
    let rest = rest.trim_start_matches(['"', '\'']);
    let end = rest
        .find(|c: char| {
            !(c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '.')
        })
        .unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

/// Charset from the header, else `<meta charset>` in the first 1 024 bytes, else lossy UTF-8.
pub fn decode_html(bytes: &[u8], content_type: Option<&str>) -> String {
    let encoding = content_type
        .and_then(header_charset)
        .map(str::to_string)
        .or_else(|| meta_charset(bytes))
        .and_then(|label| encoding_rs::Encoding::for_label(label.as_bytes()))
        .unwrap_or(encoding_rs::UTF_8);
    encoding.decode(bytes).0.into_owned()
}

type Response = ureq::http::Response<ureq::Body>;

/// Follows redirects up to the first non-redirect response. Every hop, the first one included,
/// goes through `next_hop` and `allowed` before it is requested.
fn follow(agent: &ureq::Agent, url: &str, allowed: fn(&Url) -> bool) -> Result<(Url, Response)> {
    let mut current = Url::parse(url).msg(Msg::Destination)?;
    let mut hops = 0;
    loop {
        if !allowed(&current) {
            return Err(Msg::Destination.err());
        }
        let response = agent
            .get(current.as_str())
            .header("User-Agent", USER_AGENT)
            .call()
            .with_context(|| format!("GET {current}"))?;
        if !REDIRECT_STATUSES.contains(&response.status().as_u16()) {
            return Ok((current, response));
        }
        let location = header(&response, "location").msg(Msg::NoLocation)?;
        current = next_hop(&current, &location, hops)?;
        hops += 1;
    }
}

fn header(response: &Response, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

/// Blocking download. Every hop, the first one included, goes through `is_remote_http` before it is requested.
pub fn download(url: &str) -> Result<Download> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(DOWNLOAD_TIMEOUT))
        .max_redirects(0)
        .http_status_as_error(false)
        .build()
        .into();
    let (current, mut response) = follow(&agent, url, is_remote_http)?;
    let status = response.status().as_u16();
    let content_type = header(&response, "content-type");
    if status >= 400 {
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| Some((k.as_str().to_lowercase(), v.to_str().ok()?.to_string())))
            .collect();
        let headers: Vec<(&str, &str)> = headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let blocked = is_blocked(&BlockSignals {
            status: Some(status),
            headers: &headers,
            final_url: Some(current.as_str()),
            markdown: "",
            word_count: None,
        });
        return Err(if blocked {
            Msg::Blocked.err()
        } else {
            // `is_definitive_failure` still reads this error.
            ureq::Error::StatusCode(status).into()
        });
    }
    if !is_html(content_type.as_deref()) {
        return Err(Msg::NotHtml.err());
    }
    let mut bytes = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(MAX_HTML_BYTES)
        .read_to_end(&mut bytes)?;
    Ok(Download {
        final_url: current.to_string(),
        html: decode_html(&bytes, content_type.as_deref()),
    })
}

/// A downloaded image: its type, sniffed from its own bytes, and the bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub mime: String,
    pub bytes: Vec<u8>,
}

impl Image {
    /// `data:` URL: the app's CSP only allows images from `self` and `data:`.
    pub fn data_url(&self) -> String {
        format!("data:{};base64,{}", self.mime, BASE64.encode(&self.bytes))
    }
}

/// Image type from its bytes, never trusting the server's `Content-Type`: bitmap formats by
/// signature, and SVG only if it really is one (`is_svg`).
pub fn sniff_image(bytes: &[u8]) -> Option<&'static str> {
    let at = |offset: usize, magic: &[u8]| bytes.get(offset..offset + magic.len()) == Some(magic);
    IMAGE_MAGIC
        .iter()
        .find(|(_, offset, magics)| magics.iter().any(|magic| at(*offset, magic)))
        .filter(|(mime, ..)| *mime != WEBP_MIME || at(WEBP_OFFSET, WEBP_TAG))
        .map(|(mime, ..)| *mime)
        .or_else(|| is_svg(bytes).then_some(SVG_MIME))
}

/// `true` only for an SVG: well-formed UTF-8 XML with a single `<svg>` root in the SVG namespace.
/// No DOCTYPE, hence no custom entities to expand (XML bombs), no processing instructions
/// (`xml-stylesheet`), no `<script>`, `<foreignObject>` or `on…` attributes. It is only shown
/// through `<img>`, where WebKit runs no scripts and loads nothing but `data:`.
pub fn is_svg(bytes: &[u8]) -> bool {
    check_svg(bytes).is_some()
}

fn check_svg(bytes: &[u8]) -> Option<()> {
    let text = std::str::from_utf8(bytes).ok()?;
    let mut reader = NsReader::from_str(text.strip_prefix(UTF8_BOM).unwrap_or(text));
    let config = reader.config_mut();
    config.check_comments = true;
    config.check_end_names = true;
    config.allow_unmatched_ends = false;
    config.allow_dangling_amp = false;
    let (mut depth, mut roots, mut first) = (0usize, 0, true);
    loop {
        let (ns, event) = reader.read_resolved_event().ok()?;
        let at_start = std::mem::replace(&mut first, false);
        match event {
            Event::Decl(ref decl) if at_start => {
                if let Some(encoding) = decl.encoding() {
                    encoding.ok()?.eq_ignore_ascii_case(UTF8).then_some(())?;
                }
            }
            Event::Start(ref e) | Event::Empty(ref e) => {
                if depth == 0 {
                    roots += 1;
                    let in_svg =
                        matches!(ns, ResolveResult::Bound(Namespace(n)) if n == SVG_NAMESPACE);
                    (roots == 1 && in_svg && e.local_name().as_ref() == SVG_ROOT).then_some(())?;
                }
                let name = e.local_name();
                (!SVG_ACTIVE_ELEMENTS.contains(&name.as_ref())).then_some(())?;
                for attr in e.attributes().with_checks(true) {
                    let attr = attr.ok()?;
                    attr.normalized_value(XmlVersion::Implicit1_0).ok()?;
                    let key = attr.key.local_name();
                    let prefix = key.as_ref().get(..EVENT_ATTRIBUTE_PREFIX.len());
                    let handler =
                        prefix.is_some_and(|p| p.eq_ignore_ascii_case(EVENT_ATTRIBUTE_PREFIX));
                    (!handler).then_some(())?;
                }
                if matches!(event, Event::Start(_)) {
                    depth += 1;
                }
            }
            Event::End(_) => depth = depth.checked_sub(1)?,
            Event::Text(ref t) => (depth > 0 || t.trim().is_empty()).then_some(())?,
            Event::CData(_) => (depth > 0).then_some(())?,
            Event::GeneralRef(ref r) => {
                let known = if r.is_char_ref() {
                    matches!(r.resolve_char_ref(), Ok(Some(_)))
                } else {
                    XML_ENTITIES.contains(&&**r)
                };
                (depth > 0 && known).then_some(())?;
            }
            Event::Comment(_) => {}
            Event::Eof => return (roots == 1 && depth == 0).then_some(()),
            // DOCTYPE, processing instructions and a misplaced declaration.
            Event::DocType(_) | Event::PI(_) | Event::Decl(_) => return None,
        }
    }
}

/// `true` if the failure may clear up on its own: connection, timeout, DNS, cut-off read, 5xx or
/// 429. Not a blocked destination, a 4xx, a non-image or an oversized one.
pub fn is_transient(err: &anyhow::Error) -> bool {
    match err.downcast_ref::<ureq::Error>() {
        Some(ureq::Error::StatusCode(status)) => {
            *status == TOO_MANY_REQUESTS || *status >= SERVER_ERRORS
        }
        Some(e) => matches!(
            e,
            ureq::Error::Io(_)
                | ureq::Error::Timeout(_)
                | ureq::Error::HostNotFound
                | ureq::Error::ConnectionFailed
                | ureq::Error::BodyStalled
        ),
        None => err.downcast_ref::<std::io::Error>().is_some(),
    }
}

/// Like `is_remote_http`, and an IP literal in the URL must also be public. Host names are
/// filtered by `PublicResolver` when resolved.
pub fn is_public_http(url: &Url) -> bool {
    is_remote_http(url)
        && match url.host() {
            Some(url::Host::Ipv4(ip)) => is_public_ip(ip.into()),
            Some(url::Host::Ipv6(ip)) => is_public_ip(ip.into()),
            _ => true,
        }
}

/// `false` for anything that is not the Internet: loopback, unspecified, private, link-local,
/// CGNAT, broadcast and multicast; for IPv6 also ULA, link-local and those embedding an IPv4.
pub fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_public_v4(v4),
        // `to_ipv4` turns `::` and `::1` into `0.0.0.0` and `0.0.0.1`, which fall in `0.0.0.0/8`.
        IpAddr::V6(v6) => v6.to_ipv4().map_or_else(
            || {
                !(v6.is_loopback()
                    || v6.is_unspecified()
                    || v6.is_unique_local()
                    || v6.is_unicast_link_local()
                    || v6.is_multicast())
            },
            is_public_v4,
        ),
    }
}

fn is_public_v4(ip: Ipv4Addr) -> bool {
    let [first, second, ..] = ip.octets();
    let this_network = first == 0;
    let shared = first == CGNAT_FIRST && CGNAT_SECOND.contains(&second);
    !(this_network
        || shared
        || ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_multicast())
}

/// Resolver that only lets public addresses through. It filters the same resolution used to
/// connect, so a DNS answer that changes between check and connect cannot sneak in a local IP.
#[derive(Debug, Default)]
struct PublicResolver(DefaultResolver);

impl Resolver for PublicResolver {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        config: &ureq::config::Config,
        timeout: NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let mut public = self.empty();
        for addr in self.0.resolve(uri, config, timeout)?.iter() {
            if is_public_ip(addr.ip()) {
                public.push(*addr);
            }
        }
        // Not `HostNotFound`: that one is transient (`is_transient`) and this is not.
        if public.is_empty() {
            return Err(ureq::Error::Other(Box::new(Failure::new(Msg::Destination))));
        }
        Ok(public)
    }
}

/// Blocking download of a page's favicon: public destinations only (`is_public_http` on every hop
/// and `PublicResolver`), no proxy, at most `MAX_FAVICON_BYTES`, and a type from `sniff_image`.
pub fn download_favicon(url: &str) -> Result<Image> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(FAVICON_TIMEOUT))
        .max_redirects(0)
        .http_status_as_error(false)
        // A proxy would be resolved through `PublicResolver` and, if local, could not be used.
        .proxy(None)
        .build();
    let agent = ureq::Agent::with_parts(
        config,
        DefaultConnector::default(),
        PublicResolver::default(),
    );
    let (_, mut response) = follow(&agent, url, is_public_http)?;
    let status = response.status().as_u16();
    if status >= 400 {
        return Err(ureq::Error::StatusCode(status).into());
    }
    let mut bytes = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(MAX_FAVICON_BYTES + 1)
        .read_to_end(&mut bytes)?;
    image_from_bytes(bytes, MAX_FAVICON_BYTES)
}

fn image_from_bytes(bytes: Vec<u8>, max: u64) -> Result<Image> {
    if bytes.len() as u64 > max {
        return Err(Msg::FaviconTooLarge.err());
    }
    let mime = sniff_image(&bytes).msg(Msg::NotImage)?;
    Ok(Image {
        mime: mime.to_string(),
        bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remote(url: &str) -> bool {
        is_remote_http(&Url::parse(url).expect("URL de test válida"))
    }

    #[test]
    fn is_remote_http_rejects_local_and_non_http() {
        for url in ["https://example.com/a", "http://example.com"] {
            assert!(remote(url), "{url} debería admitirse");
        }
        for url in [
            "http://localhost:1420",
            "http://LOCALHOST",
            "http://localhost.",
            "http://app.localhost",
            "http://127.0.0.1:47821",
            "http://127.1.2.3",
            "http://0.0.0.0",
            "http://[::1]:8080",
            "http://[::ffff:127.0.0.1]",
            "http://[::127.0.0.1]",
            "tauri://localhost",
            "file:///etc/passwd",
            "about:blank",
            "data:text/html,x",
            "javascript:1",
        ] {
            assert!(!remote(url), "{url} debería rechazarse");
        }
    }

    fn kind(err: anyhow::Error) -> Option<Msg> {
        err.downcast::<Failure>().ok().map(|f| f.msg)
    }

    #[test]
    fn next_hop_rejects_loopback_and_caps_hops() {
        let base = Url::parse("https://example.com/a").expect("URL");
        let err = next_hop(&base, "http://127.0.0.1/", 0).expect_err("loopback");
        assert_eq!(kind(err), Some(Msg::Destination));
        assert_eq!(
            next_hop(&base, "/b", 0).expect("relativa").as_str(),
            "https://example.com/b"
        );
        assert_eq!(
            next_hop(&base, "https://other.org/x", 3)
                .expect("otro dominio")
                .as_str(),
            "https://other.org/x"
        );
        let err = next_hop(&base, "/b", MAX_REDIRECTS).expect_err("tope");
        assert_eq!(kind(err), Some(Msg::TooManyRedirects));
    }

    #[test]
    fn is_html_accepts_only_html() {
        assert!(is_html(Some("text/html; charset=utf-8")));
        assert!(is_html(Some("application/xhtml+xml")));
        assert!(!is_html(Some("application/pdf")));
        assert!(!is_html(None));
    }

    #[test]
    fn decode_html_honours_declared_charset() {
        let latin1 = b"canci\xf3n";
        assert_eq!(
            decode_html(latin1, Some("text/html; charset=windows-1252")),
            "canción"
        );
        let mut doc = b"<meta charset=\"iso-8859-1\"><p>".to_vec();
        doc.extend_from_slice(latin1);
        assert!(decode_html(&doc, Some("text/html")).ends_with("canción"));
        assert!(decode_html(&doc, None).ends_with("canción"));
        assert_eq!(decode_html("canción".as_bytes(), None), "canción");
    }

    #[test]
    fn sniff_image_reads_the_bytes_not_the_name() {
        let cases: [(&[u8], Option<&str>); 11] = [
            (b"\x89PNG\r\n\x1a\nIHDR", Some("image/png")),
            (b"\xff\xd8\xff\xe0JFIF", Some("image/jpeg")),
            (b"GIF89a....", Some("image/gif")),
            (b"RIFF\x10\x00\x00\x00WEBPVP8 ", Some("image/webp")),
            (b"RIFF\x10\x00\x00\x00WAVEfmt ", None),
            (b"\x00\x00\x01\x00\x01\x00", Some("image/x-icon")),
            (b"\x00\x00\x00\x1cftypavif", Some("image/avif")),
            (
                b"<svg xmlns='http://www.w3.org/2000/svg'><script/></svg>",
                None,
            ),
            (
                b"<svg xmlns='http://www.w3.org/2000/svg'><circle r='1'/></svg>",
                Some("image/svg+xml"),
            ),
            (b"<!doctype html><html>", None),
            (b"", None),
        ];
        for (bytes, mime) in cases {
            assert_eq!(
                sniff_image(bytes),
                mime,
                "{:?}",
                String::from_utf8_lossy(bytes)
            );
        }
    }

    #[test]
    fn image_from_bytes_caps_the_size_and_checks_the_type() {
        let png = b"\x89PNG\r\n\x1a\n1234".to_vec();
        let max = png.len() as u64;
        let image = image_from_bytes(png.clone(), max).expect("justo en el tope");
        assert_eq!(image.mime, "image/png");
        assert_eq!(image.data_url(), "data:image/png;base64,iVBORw0KGgoxMjM0");
        assert_eq!(
            kind(image_from_bytes(png, max - 1).expect_err("un byte de más")),
            Some(Msg::FaviconTooLarge)
        );
        assert_eq!(
            kind(image_from_bytes(b"<html>".to_vec(), max).expect_err("no es imagen")),
            Some(Msg::NotImage)
        );
    }

    #[test]
    fn is_public_ip_rejects_local_networks() {
        for ip in [
            "93.184.216.34",
            "8.8.8.8",
            "2606:4700::1111",
            "::ffff:8.8.8.8",
        ] {
            assert!(is_public_ip(ip.parse().expect("IP")), "{ip} es pública");
        }
        for ip in [
            "10.0.0.1",
            "172.16.5.4",
            "172.31.255.255",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "100.127.255.255",
            "127.0.0.1",
            "0.0.0.0",
            "0.1.2.3",
            "255.255.255.255",
            "224.0.0.1",
            "::1",
            "::",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "ff02::1",
            "::ffff:192.168.1.1",
            "::ffff:169.254.169.254",
        ] {
            assert!(!is_public_ip(ip.parse().expect("IP")), "{ip} no es pública");
        }
        assert!(
            is_public_ip("100.128.0.1".parse().expect("IP")),
            "fuera de CGNAT"
        );
    }

    #[test]
    fn is_public_http_rejects_private_literals() {
        let public = |url: &str| is_public_http(&Url::parse(url).expect("URL"));
        assert!(public("https://example.com/i.png"));
        assert!(public("https://8.8.8.8/i.png"));
        for url in [
            "http://192.168.1.1/i.png",
            "http://10.0.0.1/",
            "http://169.254.169.254/latest",
            "http://[fe80::1]/",
            "http://[fd00::1]/",
            "http://localhost/i.png",
            "file:///etc/passwd",
        ] {
            assert!(!public(url), "{url} debería rechazarse");
        }
    }

    #[test]
    fn download_favicon_refuses_a_private_destination_before_connecting() {
        for url in ["http://192.168.1.1/i.png", "http://[::1]/i.png", "nada"] {
            let err = download_favicon(url).expect_err(url);
            assert_eq!(kind(err), Some(Msg::Destination), "{url}");
        }
    }

    #[test]
    fn public_resolver_drops_local_addresses() {
        let resolve = |uri: &str| {
            PublicResolver::default().resolve(
                &uri.parse().expect("URI"),
                &ureq::config::Config::default(),
                NextTimeout::default(),
            )
        };
        // No network: IP literals and `localhost` resolve locally.
        let public = resolve("http://8.8.8.8/").expect("pública");
        assert_eq!(public.len(), 1);
        for uri in ["http://10.0.0.1/", "http://localhost/", "http://[fe80::1]/"] {
            assert!(
                matches!(resolve(uri), Err(ureq::Error::Other(e)) if e.is::<Failure>()),
                "{uri} debería quedarse sin direcciones"
            );
        }
    }

    /// A real SVG favicon (svelte.dev's, shortened), with a declaration and a comment.
    const SVG_FAVICON: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<!-- favicon -->
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 98.1 118">
  <title>Logo &amp; marca &#169;</title>
  <path fill="#FF3E00" d="M91.8,15.6C80.9-0.1,59.2-4.7,43.6,5.2L16.1,22.8z"/>
  <style><![CDATA[ path { opacity: .9 } ]]></style>
</svg>
"##;

    #[test]
    fn a_real_svg_favicon_is_accepted() {
        assert_eq!(sniff_image(SVG_FAVICON.as_bytes()), Some("image/svg+xml"));
        let bom = format!("\u{feff}{SVG_FAVICON}");
        assert_eq!(sniff_image(bom.as_bytes()), Some("image/svg+xml"));
        let prefixed = r#"<s:svg xmlns:s="http://www.w3.org/2000/svg"><s:rect/></s:svg>"#;
        assert!(is_svg(prefixed.as_bytes()));
        let image = image_from_bytes(SVG_FAVICON.as_bytes().to_vec(), MAX_FAVICON_BYTES)
            .expect("SVG dentro del tope");
        assert!(image.data_url().starts_with("data:image/svg+xml;base64,"));
    }

    #[test]
    fn anything_that_is_not_plainly_svg_is_rejected() {
        let ns = r#"xmlns="http://www.w3.org/2000/svg""#;
        let cases = [
            // HTML posing as SVG.
            format!("<!doctype html><html><body><svg {ns}></svg></body></html>"),
            format!("<html><svg {ns}></svg></html>"),
            format!("<svg {ns}><p>sin cerrar</svg>"),
            // DOCTYPE and entities (XML bombs).
            format!(
                r#"<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd"><svg {ns}/>"#
            ),
            format!(
                r#"<!DOCTYPE svg [<!ENTITY a "aaaa"><!ENTITY b "&a;&a;&a;">]><svg {ns}><text>&b;</text></svg>"#
            ),
            format!("<svg {ns}><text>&b;</text></svg>"),
            format!(r#"<svg {ns} id="&b;"/>"#),
            // Another root, or `svg` outside its namespace.
            format!("<math {ns}/>"),
            "<svg/>".to_string(),
            r#"<svg xmlns="http://www.w3.org/1999/xhtml"/>"#.to_string(),
            r#"<x:svg xmlns:x="urn:otro"/>"#.to_string(),
            // Two roots, text outside, unclosed, processing instruction, another encoding.
            format!("<svg {ns}/><svg {ns}/>"),
            format!("<svg {ns}/>texto"),
            format!("<svg {ns}><g>"),
            format!(r#"<?xml-stylesheet href="https://x.test/a.css"?><svg {ns}/>"#),
            format!(r#"<?xml version="1.0" encoding="ISO-8859-1"?><svg {ns}/>"#),
            format!(r#"<svg {ns} a="1" a="2"/>"#),
            // Active content, even though `<img>` would not run it.
            format!("<svg {ns}><script>alert(1)</script></svg>"),
            format!(r#"<svg {ns} onload="alert(1)"/>"#),
            format!(r#"<svg {ns}><g ONclick="x()"/></svg>"#),
            format!("<svg {ns}><foreignObject><div/></foreignObject></svg>"),
            String::new(),
        ];
        for case in cases {
            assert_eq!(sniff_image(case.as_bytes()), None, "{case}");
        }
        assert_eq!(
            sniff_image(b"\xff\xfe<\x00s\x00v\x00g\x00"),
            None,
            "no es UTF-8"
        );
    }

    #[test]
    fn only_passing_failures_are_transient() {
        let transient: [anyhow::Error; 6] = [
            ureq::Error::StatusCode(503).into(),
            ureq::Error::StatusCode(429).into(),
            ureq::Error::ConnectionFailed.into(),
            ureq::Error::HostNotFound.into(),
            anyhow::Error::from(ureq::Error::Io(std::io::ErrorKind::TimedOut.into()))
                .context("GET https://a.test/f.ico"),
            std::io::Error::from(std::io::ErrorKind::ConnectionReset).into(),
        ];
        for err in &transient {
            assert!(is_transient(err), "{err:#}");
        }
        let permanent: [anyhow::Error; 6] = [
            ureq::Error::StatusCode(404).into(),
            ureq::Error::StatusCode(403).into(),
            ureq::Error::Other(Box::new(Failure::new(Msg::Destination))).into(),
            Msg::Destination.err(),
            Msg::NotImage.err(),
            Msg::FaviconTooLarge.err(),
        ];
        for err in &permanent {
            assert!(!is_transient(err), "{err:#}");
        }
    }
}
