//! Web page → Markdown: page and metadata types, `Fetcher`s and thin-content detection.
//! Defuddle produces the Markdown, in the `parser` window or the extension; there is no extractor here.

use crate::blocked::{BlockSignals, is_blocked, is_blocked_error};
use crate::download::Download;
use crate::msg::{Failure, Msg, MsgExt};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) const ERR_FAKE_UNKNOWN_URL: &str = "fake fetcher: url desconocida"; // i18n-allow: test double
/// Dead-link HTTP statuses (not found, gone): the webview would only see the error page.
const DEAD_LINK_STATUSES: [u16; 2] = [404, 410];
/// Server error range (5xx), which the webview cannot rescue either.
const SERVER_ERROR_STATUSES: std::ops::Range<u16> = 500..600;

/// Markdown with fewer non-whitespace characters than this counts as thin.
pub const MIN_CONTENT_CHARS: usize = 200;

/// Metadata caps, in characters (not bytes).
pub const MAX_DESCRIPTION_CHARS: usize = 1_000;
pub const MAX_NAME_CHARS: usize = 300;
pub const MAX_PUBLISHED_CHARS: usize = 100;
pub const MAX_LANGUAGE_CHARS: usize = 35;
pub const MAX_URL_CHARS: usize = 2_048;
pub const MAX_EXTRACTOR_CHARS: usize = 100;

/// Metadata returned by the extractor. It comes from a third-party page: `sanitized` bounds it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PageMeta {
    pub description: Option<String>,
    pub author: Option<String>,
    pub site: Option<String>,
    pub published: Option<String>,
    pub language: Option<String>,
    pub image: Option<String>,
    pub favicon: Option<String>,
    pub word_count: Option<u32>,
    pub extractor: Option<String>,
}

/// Trims, returns `None` if empty and cuts to `max` characters. With `single_line`, any run of
/// whitespace or line breaks becomes a single space.
pub(crate) fn clean(value: Option<String>, max: usize, single_line: bool) -> Option<String> {
    let value = value?;
    let text = if single_line {
        value.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        value.trim().to_string()
    };
    let text: String = text.chars().take(max).collect();
    let text = text.trim_end().to_string();
    (!text.is_empty()).then_some(text)
}

/// Only valid http(s) URLs no longer than `MAX_URL_CHARS`.
fn clean_url(value: Option<String>) -> Option<String> {
    let value = value?;
    let value = value.trim();
    let parsed = url::Url::parse(value).ok()?;
    (matches!(parsed.scheme(), "http" | "https") && value.chars().count() <= MAX_URL_CHARS)
        .then(|| value.to_string())
}

impl PageMeta {
    pub fn sanitized(self) -> PageMeta {
        PageMeta {
            description: clean(self.description, MAX_DESCRIPTION_CHARS, false),
            author: clean(self.author, MAX_NAME_CHARS, true),
            site: clean(self.site, MAX_NAME_CHARS, true),
            published: clean(self.published, MAX_PUBLISHED_CHARS, true),
            language: clean(self.language, MAX_LANGUAGE_CHARS, true),
            image: clean_url(self.image),
            favicon: clean_url(self.favicon),
            word_count: self.word_count,
            extractor: clean(self.extractor, MAX_EXTRACTOR_CHARS, true),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Page {
    #[serde(default)]
    pub title: Option<String>,
    pub markdown: String,
    #[serde(default)]
    pub meta: PageMeta,
}

#[async_trait::async_trait]
pub trait Fetcher: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<Page>;
}

/// Extracts Markdown and metadata from HTML already downloaded or captured.
#[async_trait::async_trait]
pub trait HtmlExtractor: Send + Sync {
    async fn extract(&self, url: &str, html: &str) -> Result<Page>;
}

/// Tests: returns the fixed page or fails with `ERR_FAKE_UNKNOWN_URL`.
#[derive(Default)]
pub struct FakeExtractor {
    pub pages: HashMap<String, Page>,
}

#[async_trait::async_trait]
impl HtmlExtractor for FakeExtractor {
    async fn extract(&self, url: &str, _html: &str) -> Result<Page> {
        self.pages.get(url).cloned().context(ERR_FAKE_UNKNOWN_URL)
    }
}

/// Downloads (on a separate thread, so the runtime is not blocked) and extracts. An anti-bot
/// block page is `Msg::Blocked`, not content. The download is injectable for tests.
pub(crate) async fn fetch_page<E: HtmlExtractor>(
    url: &str,
    extractor: &E,
    download: fn(&str) -> Result<Download>,
) -> Result<Page> {
    let target = url.to_string();
    let downloaded = tokio::task::spawn_blocking(move || download(&target))
        .await
        .msg(Msg::FetchTask)??;
    // The final URL, not the requested one: relative links resolve against where we landed.
    let page = extractor
        .extract(&downloaded.final_url, &downloaded.html)
        .await?;
    let blocked = is_blocked(&BlockSignals {
        status: None,
        headers: &[],
        final_url: Some(&downloaded.final_url),
        markdown: &page.markdown,
        word_count: page.meta.word_count,
    });
    if blocked {
        return Err(Msg::Blocked.err());
    }
    Ok(page)
}

pub struct HttpFetcher<E: HtmlExtractor> {
    pub extractor: E,
}

#[async_trait::async_trait]
impl<E: HtmlExtractor> Fetcher for HttpFetcher<E> {
    async fn fetch(&self, url: &str) -> Result<Page> {
        fetch_page(url, &self.extractor, crate::download::download).await
    }
}

/// Fixed pages for tests; a missing URL is an error.
#[derive(Debug, Default)]
pub struct FakeFetcher {
    pub pages: HashMap<String, Page>,
}

#[async_trait::async_trait]
impl Fetcher for FakeFetcher {
    async fn fetch(&self, url: &str) -> Result<Page> {
        self.pages.get(url).cloned().context(ERR_FAKE_UNKNOWN_URL)
    }
}

/// Tries `primary`; if it fails or its content is thin, tries `secondary`, unless the failure is
/// definitive (`is_definitive_failure`). If `secondary` sees an anti-bot check, its `Msg::Blocked`
/// is returned; if it does not get a good enough page, `primary`'s result is returned.
pub struct FallbackFetcher<P, S> {
    pub primary: P,
    pub secondary: S,
}

#[async_trait::async_trait]
impl<P: Fetcher, S: Fetcher> Fetcher for FallbackFetcher<P, S> {
    async fn fetch(&self, url: &str) -> Result<Page> {
        let first = self.primary.fetch(url).await;
        match &first {
            Ok(page) if !is_thin(&page.markdown) => return first,
            Err(err) if is_definitive_failure(err) => return first,
            _ => {}
        }
        match self.secondary.fetch(url).await {
            Ok(page) if !is_thin(&page.markdown) => Ok(page),
            Err(err) if is_blocked_error(&err) => Err(err),
            _ => first,
        }
    }
}

/// The server answered 404, 410 or 5xx: loading the page another way would not change the result.
fn is_definitive_failure(err: &anyhow::Error) -> bool {
    // Forbidden destination or non-HTML content: the webview would not fix it either.
    matches!(
        err.downcast_ref::<Failure>().map(|f| &f.msg),
        Some(Msg::Destination | Msg::NotHtml)
    ) || matches!(
        err.downcast_ref::<ureq::Error>(),
        Some(ureq::Error::StatusCode(code))
            if DEAD_LINK_STATUSES.contains(code) || SERVER_ERROR_STATUSES.contains(code)
    )
}

/// Removes each `open…close` span whose text after `open` satisfies `valid`; the rest is kept as is.
/// shortcut: a tag `<` or a `](` with no closer counts as text; move to an HTML parser if it gets in the way.
fn strip_spans(text: &str, open: &str, close: char, valid: fn(&str) -> bool) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find(open) {
        let after = &rest[start + open.len()..];
        if !valid(after) {
            out.push_str(&rest[..start + open.len()]);
            rest = after;
            continue;
        }
        // No closer here means none for any later `open` either: searching again would be quadratic.
        let Some(end) = after.find(close) else { break };
        out.push_str(&rest[..start]);
        rest = &after[end + close.len_utf8()..];
    }
    out.push_str(rest);
    out
}

/// A `<` opens a tag only when followed by a letter, `/` or `!`, so `a < b` or `->` do not count.
fn is_tag_start(after: &str) -> bool {
    after
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '/' || c == '!')
}

/// Fewer than `MIN_CONTENT_CHARS` non-whitespace characters, not counting HTML tags or link
/// targets (`](…)`): a captcha iframe or a long URL is not content.
pub fn is_thin(markdown: &str) -> bool {
    let text = strip_spans(
        &strip_spans(markdown, "<", '>', is_tag_start),
        "](",
        ')',
        |_| true,
    );
    text.chars().filter(|c| !c.is_whitespace()).count() < MIN_CONTENT_CHARS
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result, anyhow};
    use std::sync::atomic::{AtomicUsize, Ordering};

    const URL: &str = "https://example.com/post";

    /// Simulated failure: any message, or an HTTP status as `fetch` produces it.
    #[derive(Clone)]
    enum Failure {
        Message(&'static str),
        Status(u16),
        Blocked,
    }

    /// Test fetcher: returns a fixed response and counts calls.
    struct StubFetcher {
        result: std::result::Result<Page, Failure>,
        calls: AtomicUsize,
    }

    impl StubFetcher {
        fn page(chars: usize) -> Self {
            let page = Page {
                title: Some(format!("t{chars}")),
                markdown: "a".repeat(chars),
                ..Page::default()
            };
            Self {
                result: Ok(page),
                calls: AtomicUsize::new(0),
            }
        }
        fn failing(message: &'static str) -> Self {
            Self {
                result: Err(Failure::Message(message)),
                calls: AtomicUsize::new(0),
            }
        }
        fn status(code: u16) -> Self {
            Self {
                result: Err(Failure::Status(code)),
                calls: AtomicUsize::new(0),
            }
        }
        fn blocked() -> Self {
            Self {
                result: Err(Failure::Blocked),
                calls: AtomicUsize::new(0),
            }
        }
        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl Fetcher for StubFetcher {
        async fn fetch(&self, _url: &str) -> Result<Page> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            match self.result.clone() {
                Ok(page) => Ok(page),
                Err(Failure::Message(m)) => Err(anyhow!(m)),
                Err(Failure::Status(code)) => {
                    Err(ureq::Error::StatusCode(code)).with_context(|| format!("GET {URL}"))
                }
                Err(Failure::Blocked) => Err(Msg::Blocked.err()).context("webview"),
            }
        }
    }

    const GOOD: usize = MIN_CONTENT_CHARS;
    const THIN: usize = MIN_CONTENT_CHARS - 1;

    #[tokio::test]
    async fn good_primary_skips_secondary() {
        let f = FallbackFetcher {
            primary: StubFetcher::page(GOOD),
            secondary: StubFetcher::page(GOOD + 1),
        };
        assert_eq!(f.fetch(URL).await.unwrap().markdown.len(), GOOD);
        assert_eq!(f.secondary.calls(), 0);
    }

    #[tokio::test]
    async fn thin_primary_uses_good_secondary() {
        let f = FallbackFetcher {
            primary: StubFetcher::page(THIN),
            secondary: StubFetcher::page(GOOD),
        };
        assert_eq!(f.fetch(URL).await.unwrap().markdown.len(), GOOD);
        assert_eq!(f.secondary.calls(), 1);
    }

    #[tokio::test]
    async fn failed_primary_uses_good_secondary() {
        let f = FallbackFetcher {
            primary: StubFetcher::failing("p"),
            secondary: StubFetcher::page(GOOD),
        };
        assert_eq!(f.fetch(URL).await.unwrap().markdown.len(), GOOD);
        assert_eq!(f.secondary.calls(), 1);
    }

    #[tokio::test]
    async fn thin_primary_is_kept_when_secondary_fails() {
        let f = FallbackFetcher {
            primary: StubFetcher::page(THIN),
            secondary: StubFetcher::failing("s"),
        };
        assert_eq!(f.fetch(URL).await.unwrap().markdown.len(), THIN);
        assert_eq!(f.secondary.calls(), 1);
    }

    #[tokio::test]
    async fn thin_primary_is_kept_when_secondary_is_thin() {
        let f = FallbackFetcher {
            primary: StubFetcher::page(THIN),
            secondary: StubFetcher::page(THIN - 1),
        };
        assert_eq!(f.fetch(URL).await.unwrap().markdown.len(), THIN);
        assert_eq!(f.secondary.calls(), 1);
    }

    #[tokio::test]
    async fn both_fail_returns_primary_error() {
        let f = FallbackFetcher {
            primary: StubFetcher::failing("primario"),
            secondary: StubFetcher::failing("secundario"),
        };
        assert_eq!(f.fetch(URL).await.unwrap_err().to_string(), "primario");
        assert_eq!(f.secondary.calls(), 1);
    }

    /// Dead link or broken server: the webview would only load the error page.
    #[tokio::test]
    async fn dead_link_status_skips_secondary() {
        for code in [404, 410, 500, 503] {
            let f = FallbackFetcher {
                primary: StubFetcher::status(code),
                secondary: StubFetcher::page(GOOD),
            };
            let err = f.fetch(URL).await.unwrap_err();
            assert!(
                matches!(err.downcast_ref::<ureq::Error>(), Some(ureq::Error::StatusCode(c)) if *c == code),
                "{code}: {err:#}"
            );
            assert_eq!(f.secondary.calls(), 0, "{code}");
        }
    }

    #[tokio::test]
    async fn other_status_still_falls_back() {
        for code in [401, 403, 429] {
            let f = FallbackFetcher {
                primary: StubFetcher::status(code),
                secondary: StubFetcher::page(GOOD),
            };
            assert_eq!(f.fetch(URL).await.unwrap().markdown.len(), GOOD, "{code}");
            assert_eq!(f.secondary.calls(), 1, "{code}");
        }
    }

    /// The webview saw the anti-bot check: that verdict beats a thin or failed primary.
    #[tokio::test]
    async fn blocked_secondary_wins_over_thin_or_failed_primary() {
        for primary in [StubFetcher::page(THIN), StubFetcher::failing("p")] {
            let f = FallbackFetcher {
                primary,
                secondary: StubFetcher::blocked(),
            };
            let err = f.fetch(URL).await.unwrap_err();
            assert!(is_blocked_error(&err), "{err:#}");
        }
    }

    /// Repeated unclosed `<a` or `](` must not rescan the rest for each one.
    #[test]
    fn is_thin_is_linear_on_unclosed_openers() {
        const BUDGET: std::time::Duration = std::time::Duration::from_secs(1);
        for opener in ["<a", "]("] {
            let hostile = opener.repeat(500_000);
            let started = std::time::Instant::now();
            assert!(!is_thin(&hostile), "{opener}");
            assert!(
                started.elapsed() < BUDGET,
                "{opener}: {:?}",
                started.elapsed()
            );
        }
    }

    #[test]
    fn is_thin_counts_non_whitespace_chars() {
        let spaced = |n: usize| "a \n".repeat(n);
        assert!(is_thin(&spaced(MIN_CONTENT_CHARS - 1)));
        assert!(!is_thin(&spaced(MIN_CONTENT_CHARS)));
        assert!(is_thin(""));
        assert!(is_thin("  \n\t"));
    }

    fn meta_with(f: impl FnOnce(&mut PageMeta)) -> PageMeta {
        let mut m = PageMeta::default();
        f(&mut m);
        m.sanitized()
    }

    #[test]
    fn contract_fixture_deserializes_as_page() {
        let page: Page = serde_json::from_str(include_str!(
            "../../../packages/extract/fixtures/extracted.json"
        ))
        .unwrap();
        assert!(page.title.is_some());
        let m = &page.meta;
        assert!(
            [
                &m.description,
                &m.author,
                &m.site,
                &m.published,
                &m.language,
                &m.image,
                &m.favicon
            ]
            .iter()
            .all(|f| f.is_some())
        );
        assert!(m.word_count.is_some());
        assert_eq!(m.extractor.as_deref(), Some("defuddle@0.19.6"));
        assert_eq!(page.meta.clone().sanitized(), page.meta);
    }

    #[test]
    fn sanitized_trims_empties_and_bounds() {
        let m = meta_with(|m| {
            m.author = Some("  ".into());
            m.description = Some("d".repeat(1_500));
            m.site = Some("s".repeat(400));
            m.language = Some("l".repeat(50));
        });
        assert_eq!(m.author, None);
        assert_eq!(
            m.description.unwrap().chars().count(),
            MAX_DESCRIPTION_CHARS
        );
        assert_eq!(m.site.unwrap().chars().count(), MAX_NAME_CHARS);
        assert_eq!(m.language.unwrap().chars().count(), MAX_LANGUAGE_CHARS);
    }

    #[test]
    fn single_line_fields_collapse_whitespace() {
        let m = meta_with(|m| {
            m.author = Some("Ana\n  Pérez".into());
            m.site = Some("Mi \t sitio\n".into());
            m.published = Some("2024-05-01\n10:00".into());
            m.language = Some("es\n-ES".into());
            m.description = Some("uno\n\ndos".into());
        });
        assert_eq!(m.author.as_deref(), Some("Ana Pérez"));
        assert_eq!(m.site.as_deref(), Some("Mi sitio"));
        assert_eq!(m.published.as_deref(), Some("2024-05-01 10:00"));
        assert_eq!(m.language.as_deref(), Some("es -ES"));
        assert_eq!(m.description.as_deref(), Some("uno\n\ndos"));
    }

    #[test]
    fn bounds_count_chars_not_bytes() {
        let author = format!("{}é🙂", "a".repeat(299));
        let m = meta_with(|m| m.author = Some(author));
        let author = m.author.unwrap();
        assert_eq!(author.chars().count(), MAX_NAME_CHARS);
        assert!(author.ends_with('é'));
    }

    #[test]
    fn meta_urls_must_be_http() {
        let m = meta_with(|m| {
            m.image = Some("javascript:alert(1)".into());
            m.favicon = Some("data:image/png;base64,x".into());
        });
        assert_eq!((m.image, m.favicon), (None, None));
        let m = meta_with(|m| m.image = Some("https://x/i.png".into()));
        assert_eq!(m.image.as_deref(), Some("https://x/i.png"));
        let m = meta_with(|m| m.image = Some(format!("https://x/{}", "a".repeat(3_000))));
        assert_eq!(m.image, None);
    }

    #[test]
    fn is_thin_ignores_tags_and_link_targets() {
        let iframe = format!(
            r#"<iframe src="https://geo.captcha-delivery.com/{}"></iframe>"#,
            "a".repeat(700)
        );
        assert!(is_thin(&iframe));
        assert!(is_thin(&format!(
            "[texto](https://example.com/{})",
            "a".repeat(500)
        )));
        assert!(!is_thin(&"a".repeat(250)));
    }

    #[test]
    fn is_thin_keeps_prose_with_bare_angle_brackets() {
        let prose = format!(
            "si a < b entonces {} f -> c y {}\n> cita {}",
            "texto ".repeat(20),
            "mas ".repeat(30),
            "fin ".repeat(20)
        );
        assert!(!is_thin(&prose));
    }

    #[test]
    fn definitive_failures_include_destination_and_not_html() {
        for e in [Msg::Destination, Msg::NotHtml] {
            assert!(is_definitive_failure(&e.err()));
        }
        assert!(!is_definitive_failure(&Msg::TooManyRedirects.err()));
    }

    const LONG: &str = "Un párrafo de prueba con palabras suficientes para no ser escaso. ";

    fn fake_download(_url: &str) -> Result<Download> {
        Ok(Download {
            final_url: "https://example.com/final".into(),
            html: String::new(),
        })
    }

    fn extractor_with(url: &str, markdown: String, word_count: Option<u32>) -> FakeExtractor {
        let page = Page {
            markdown,
            meta: PageMeta {
                word_count,
                ..PageMeta::default()
            },
            ..Page::default()
        };
        FakeExtractor {
            pages: HashMap::from([(url.to_string(), page)]),
        }
    }

    #[tokio::test]
    async fn fetch_page_extracts_with_the_final_url() {
        // Only the final URL is registered: with the requested one, the extractor would fail.
        let ex = extractor_with("https://example.com/final", LONG.repeat(10), None);
        let page = fetch_page("https://example.com/short", &ex, fake_download)
            .await
            .unwrap();
        assert!(page.markdown.starts_with("Un párrafo"));
    }

    #[tokio::test]
    async fn blocked_page_is_an_error_not_content() {
        let ex = extractor_with(
            "https://example.com/final",
            "Verify you are human".into(),
            Some(4),
        );
        let err = fetch_page("https://example.com/short", &ex, fake_download)
            .await
            .unwrap_err();
        assert!(is_blocked_error(&err), "{err:#}");
    }

    /// A blocked primary is not a definitive failure: the webview is tried.
    #[tokio::test]
    async fn blocked_primary_still_tries_secondary() {
        let f = FallbackFetcher {
            primary: StubFetcher::blocked(),
            secondary: StubFetcher::page(GOOD),
        };
        assert!(!is_definitive_failure(&Msg::Blocked.err()));
        assert_eq!(f.fetch(URL).await.unwrap().markdown.len(), GOOD);
        assert_eq!(f.secondary.calls(), 1);
    }
}
