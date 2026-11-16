//! Detects pages stuck on an anti-bot check.

use crate::msg::{Failure, Msg};

/// Content signals only count below this many words.
pub const BLOCKED_MAX_WORDS: usize = 150;
const CHALLENGE_STATUSES: [u16; 3] = [403, 429, 503];
const VERIFICATION_URLS: [&str; 2] = ["google.com/sorry", "/cdn-cgi/challenge-platform"];
const CAPTCHA_HOSTS: [&str; 4] = [
    "captcha-delivery.com",
    "challenges.cloudflare.com",
    "hcaptcha.com",
    "google.com/recaptcha",
];
const BLOCK_PHRASES: [&str; 9] = // i18n-allow: detection
    [
        "prove your humanity",
        "just a moment",
        "unusual traffic",
        "verify you are human",
        // Cloudflare: "Verifying you are human. This may take a few seconds."
        "verifying you are human",
        "are you a robot",
        "verifica que eres humano",
        "comprueba que eres humano",
        "no soy un robot",
    ];

/// What is known about a download or capture; each path fills in what it has.
pub struct BlockSignals<'a> {
    pub status: Option<u16>,
    /// Lowercase name, value.
    pub headers: &'a [(&'a str, &'a str)],
    pub final_url: Option<&'a str>,
    pub markdown: &'a str,
    /// When `None`, the Markdown's words are counted.
    pub word_count: Option<u32>,
}

/// `true` if any layer of the error is `Msg::Blocked`: `FallbackFetcher` does not treat it as final,
/// and the pipeline and capture mark the bookmark `blocked`.
pub fn is_blocked_error(err: &anyhow::Error) -> bool {
    err.chain().any(|layer| {
        layer
            .downcast_ref::<Failure>()
            .is_some_and(|f| f.msg == Msg::Blocked)
    })
}

fn challenge_headers(s: &BlockSignals<'_>) -> bool {
    s.headers.iter().any(|(name, value)| {
        let value = value.to_lowercase();
        match *name {
            "cf-mitigated" => value.contains("challenge"),
            "x-datadome" => true,
            "server" => value == "cloudflare",
            _ => false,
        }
    })
}

pub fn is_blocked(s: &BlockSignals<'_>) -> bool {
    if s.status.is_some_and(|c| CHALLENGE_STATUSES.contains(&c)) && challenge_headers(s) {
        return true;
    }
    let url = s.final_url.unwrap_or_default().to_lowercase();
    if VERIFICATION_URLS.iter().any(|u| url.contains(u)) {
        return true;
    }
    let words = s
        .word_count
        .map_or_else(|| s.markdown.split_whitespace().count(), |n| n as usize);
    if words >= BLOCKED_MAX_WORDS {
        return false;
    }
    let text = s.markdown.to_lowercase();
    CAPTCHA_HOSTS
        .iter()
        .chain(BLOCK_PHRASES.iter())
        .any(|p| text.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig<'a>(
        status: Option<u16>,
        headers: &'a [(&'a str, &'a str)],
        md: &'a str,
    ) -> BlockSignals<'a> {
        BlockSignals {
            status,
            headers,
            final_url: None,
            markdown: md,
            word_count: None,
        }
    }

    #[test]
    fn cloudflare_challenge_status_is_blocked() {
        assert!(is_blocked(&sig(
            Some(403),
            &[("cf-mitigated", "challenge")],
            ""
        )));
        assert!(!is_blocked(&sig(Some(403), &[], "")));
        assert!(!is_blocked(&sig(
            Some(200),
            &[("cf-mitigated", "challenge")],
            ""
        )));
    }

    #[test]
    fn datadome_captcha_page_is_blocked() {
        assert!(is_blocked(&sig(
            Some(403),
            &[("x-datadome", "protected")],
            ""
        )));
        let md = r#"<iframe src="https://geo.captcha-delivery.com/captcha/?x"></iframe>"#;
        let mut s = sig(None, &[], md);
        s.word_count = Some(0);
        assert!(is_blocked(&s));
    }

    #[test]
    fn cloudflare_verifying_text_is_blocked() {
        let md = "www.example.com Verifying you are human. This may take a few seconds.";
        assert!(is_blocked(&sig(None, &[], md)));
    }

    #[test]
    fn reddit_humanity_page_is_blocked() {
        let md = format!("Prove your humanity {}", "palabra ".repeat(36));
        assert!(is_blocked(&sig(Some(200), &[], &md)));
    }

    #[test]
    fn verification_redirect_is_blocked() {
        let mut s = sig(Some(200), &[], "");
        s.final_url = Some("https://www.google.com/sorry/index?continue=https://youtube.com");
        assert!(is_blocked(&s));
    }

    #[test]
    fn article_about_captchas_is_not_blocked() {
        let md = format!(
            "just a moment captcha-delivery.com {}",
            "texto ".repeat(398)
        );
        assert!(!is_blocked(&sig(Some(200), &[], &md)));
    }

    #[test]
    fn spanish_phrases_count() {
        assert!(is_blocked(&sig(
            None,
            &[],
            "Comprueba que eres humano para continuar"
        )));
    }
}
