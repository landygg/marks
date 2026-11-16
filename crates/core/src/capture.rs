//! Page sent by the browser extension → bookmark.
//! Extracts outside the lock, then looks up and writes in a single critical section.

use crate::blocked::{BlockSignals, is_blocked};
use crate::extract::{Page, PageMeta, is_thin};
use crate::msg::{Msg, MsgExt};
use crate::normalize_url;
use crate::store::{Bookmark, NewBookmark, Status, Store, lock};
use serde::Deserialize;
use std::fmt;
use std::sync::Mutex;
use url::Url;

/// `extractor` for content that comes from `innerText`.
const TEXT_EXTRACTOR: &str = "text";
pub(crate) const ALLOWED_SCHEMES: [&str; 2] = ["http", "https"];

#[derive(Debug, Default, Deserialize)]
pub struct Capture {
    pub url: String,
    #[serde(default)]
    pub title: Option<String>,
    /// Markdown and metadata extracted by Defuddle in the browser.
    #[serde(default)]
    pub page: Option<Page>,
    /// `document.body.innerText`: fallback when the extractor gets too little.
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug)]
pub struct Captured {
    pub bookmark: Bookmark,
    pub created: bool,
    /// The page was an anti-bot check and its content was not stored.
    pub blocked: bool,
}

#[derive(Debug)]
pub enum CaptureError {
    InvalidUrl(Msg),
    Store(anyhow::Error),
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaptureError::InvalidUrl(msg) => msg.fmt(f),
            CaptureError::Store(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for CaptureError {}

impl From<anyhow::Error> for CaptureError {
    fn from(err: anyhow::Error) -> Self {
        CaptureError::Store(err)
    }
}

/// Stores the page: a new bookmark, or the existing one if it lacked content or was `failed`.
pub fn capture(store: &Mutex<Store>, capture: Capture) -> Result<Captured, CaptureError> {
    let url = http_url(&capture.url)?;
    // `add` stores the URL exactly as the browser sent it in `original_url`.
    let original = capture.url.trim().to_string();
    let parts = page_parts(capture);

    let store = lock(store)?;
    let Some(existing) = store.find_by_url(&url)? else {
        let new = if parts.blocked {
            NewBookmark {
                url: original,
                ..Default::default()
            }
        } else {
            NewBookmark {
                url: original,
                title: parts.title,
                content: parts.content,
                meta: parts.meta,
                ..Default::default()
            }
        };
        let bookmark = store.add(new)?;
        if parts.blocked {
            store.set_blocked(bookmark.id)?;
        }
        let bookmark = store.get(bookmark.id)?.msg(Msg::BookmarkNotFound)?;
        return Ok(Captured {
            bookmark,
            created: true,
            blocked: parts.blocked,
        });
    };
    let needs_content = existing.status == Status::Failed
        || existing
            .content
            .as_deref()
            .is_none_or(|c| c.trim().is_empty());
    let blocked = parts.blocked && needs_content;
    if blocked {
        store.set_blocked(existing.id)?;
    } else if !parts.blocked {
        match parts.content {
            Some(markdown) if needs_content => {
                let page = Page {
                    title: parts.title,
                    markdown,
                    meta: parts.meta,
                };
                store.set_page(existing.id, &page)?;
                store.set_status(existing.id, Status::Pending, None)?;
            }
            _ if parts.meta != PageMeta::default() => {
                store.fill_missing_meta(existing.id, &parts.meta)?
            }
            _ => {}
        }
        // Capturing an imported URL queues it even if the page yields no content.
        if existing.status == Status::Imported {
            store.set_status(existing.id, Status::Pending, None)?;
        }
    }
    let bookmark = store.get(existing.id)?.msg(Msg::BookmarkNotFound)?;
    Ok(Captured {
        bookmark,
        created: false,
        blocked,
    })
}

fn http_url(raw: &str) -> Result<String, CaptureError> {
    let url = normalize_url(raw).map_err(|_| CaptureError::InvalidUrl(Msg::InvalidUrl))?;
    let parsed = Url::parse(&url).map_err(|_| CaptureError::InvalidUrl(Msg::InvalidUrl))?;
    if ALLOWED_SCHEMES.contains(&parsed.scheme()) {
        Ok(url)
    } else {
        Err(CaptureError::InvalidUrl(Msg::UnsupportedScheme))
    }
}

/// What a capture decides: title, chosen content, metadata (always stored, with or without
/// content) and whether it is a block page.
struct Parts {
    title: Option<String>,
    content: Option<String>,
    meta: PageMeta,
    blocked: bool,
}

fn blocked(markdown: &str, word_count: Option<u32>) -> bool {
    is_blocked(&BlockSignals {
        status: None,
        headers: &[],
        final_url: None,
        markdown,
        word_count,
    })
}

/// Content: Markdown from `page`, else `text`, else nothing. Title: `page`'s, else the tab's.
/// An `html` field from a v0.5 extension is ignored (serde drops unknown fields).
fn page_parts(capture: Capture) -> Parts {
    let blank = |t: &Option<String>| t.as_deref().is_none_or(|t| t.trim().is_empty());
    let (page_title, mut meta, page_md) = match capture.page {
        Some(p) => (p.title, p.meta, Some(p.markdown)),
        None => (None, PageMeta::default(), None),
    };
    let text = capture.text.as_deref().map(str::trim);
    // Text only decides whether the page is missing or thin; otherwise a short notice would block it.
    let page_thin = page_md.as_deref().is_none_or(is_thin);
    let is_block = page_md
        .as_deref()
        .is_some_and(|m| blocked(m, meta.word_count))
        || (page_thin && text.is_some_and(|t| blocked(t, None)));
    let markdown = match page_md.filter(|m| !is_thin(m)) {
        Some(m) => Some(m),
        None => match text.filter(|t| !is_thin(t)) {
            Some(t) => {
                meta.extractor = Some(TEXT_EXTRACTOR.into());
                Some(t.to_string())
            }
            None => None,
        },
    };
    let title = [page_title, capture.title]
        .into_iter()
        .find(|t| !blank(t))
        .flatten()
        .map(|t| t.trim().to_string());
    Parts {
        title,
        content: markdown,
        meta,
        blocked: is_block,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::BookmarkError;
    use crate::store::ErrorKind;

    const OTHER_TEXT: &str = "Contenido distinto y nuevo que reemplaza al anterior cuando el bookmark estaba fallido o vacío, \
        escrito con la longitud suficiente para no ser considerado escaso por el extractor de contenido. \
        Más texto de relleno para asegurar que pasa de los doscientos caracteres sin espacios que exige la regla.";
    const LONG_TEXT: &str = "Texto plano de respaldo con longitud suficiente para no ser escaso. \
        Se repite lo necesario para pasar de doscientos caracteres sin contar espacios en blanco, \
        porque el límite cuenta solo caracteres visibles y esta frase debe superarlo con margen.";
    const URL: &str = "https://example.com/post";

    fn store() -> Mutex<Store> {
        Mutex::new(Store::open_in_memory().unwrap())
    }

    fn cap(text: Option<&str>) -> Capture {
        Capture {
            url: URL.into(),
            text: text.map(Into::into),
            ..Capture::default()
        }
    }

    fn seed(s: &Mutex<Store>, status: Status, content: Option<&str>) -> i64 {
        let st = lock(s).unwrap();
        let b = st
            .add(NewBookmark {
                url: URL.into(),
                title: Some("Mío".into()),
                notes: String::new(),
                tags: Vec::new(),
                content: content.map(Into::into),
                ..Default::default()
            })
            .unwrap();
        let error = (status == Status::Failed).then(|| BookmarkError::raw("boom"));
        st.set_status(b.id, status, error.as_ref()).unwrap();
        b.id
    }

    #[test]
    fn capture_extracts_an_imported_bookmark() {
        let s = store();
        lock(&s)
            .unwrap()
            .insert_imported(
                &NewBookmark {
                    url: URL.into(),
                    ..Default::default()
                },
                None,
            )
            .unwrap();
        let r = capture(&s, cap(Some(LONG_TEXT))).unwrap();
        assert_eq!(r.bookmark.status, Status::Pending);
        assert_eq!(r.bookmark.content.as_deref(), Some(LONG_TEXT));
    }

    #[test]
    fn capture_queues_an_imported_bookmark_on_a_thin_page() {
        let s = store();
        lock(&s)
            .unwrap()
            .insert_imported(
                &NewBookmark {
                    url: URL.into(),
                    ..Default::default()
                },
                None,
            )
            .unwrap();
        let r = capture(&s, cap(None)).unwrap();
        assert_eq!(r.bookmark.status, Status::Pending);
    }

    #[test]
    fn capture_keeps_the_browser_url() {
        let s = store();
        let raw = "https://a.com/x?utm_source=t#top";
        let c = Capture {
            url: raw.into(),
            ..Capture::default()
        };
        let r = capture(&s, c).unwrap();
        assert_eq!(r.bookmark.url, "https://a.com/x");
        assert_eq!(r.bookmark.original_url, raw);
    }

    #[test]
    fn new_url_with_text_is_pending_with_content() {
        let s = store();
        let r = capture(&s, cap(Some(LONG_TEXT))).unwrap();
        assert!(r.created);
        assert_eq!(r.bookmark.status, Status::Pending);
        assert_eq!(r.bookmark.content.as_deref(), Some(LONG_TEXT));
    }

    #[test]
    fn new_url_without_text_has_no_content() {
        let s = store();
        let r = capture(&s, cap(None)).unwrap();
        assert_eq!(r.bookmark.content, None);
        assert_eq!(r.bookmark.title, URL);
    }

    #[test]
    fn explicit_title_wins() {
        let s = store();
        let mut c = cap(None);
        c.title = Some(" Mi título ".into());
        assert_eq!(capture(&s, c).unwrap().bookmark.title, "Mi título");
    }

    fn page(title: Option<&str>, markdown: &str) -> Page {
        Page {
            title: title.map(Into::into),
            markdown: markdown.into(),
            meta: PageMeta {
                author: Some("Ana".into()),
                word_count: Some(60),
                extractor: Some("defuddle@0.19.6".into()),
                ..Default::default()
            },
        }
    }

    fn cap_page(p: Page, text: Option<&str>) -> Capture {
        Capture {
            url: URL.into(),
            page: Some(p),
            text: text.map(Into::into),
            ..Capture::default()
        }
    }

    const BLOCK_MD: &str = "Just a moment...\n\nVerify you are human";

    #[test]
    fn page_markdown_wins_over_text() {
        let s = store();
        let r = capture(&s, cap_page(page(None, LONG_TEXT), Some(OTHER_TEXT))).unwrap();
        assert_eq!(r.bookmark.content.as_deref(), Some(LONG_TEXT));
        assert_eq!(
            r.bookmark.meta.extractor.as_deref(),
            Some("defuddle@0.19.6")
        );
        assert_eq!(r.bookmark.meta.author.as_deref(), Some("Ana"));
    }

    #[test]
    fn thin_page_falls_back_to_text_with_text_extractor() {
        let s = store();
        let r = capture(&s, cap_page(page(None, "poco"), Some(LONG_TEXT))).unwrap();
        assert_eq!(r.bookmark.content.as_deref(), Some(LONG_TEXT));
        assert_eq!(r.bookmark.meta.extractor.as_deref(), Some("text"));
    }

    #[test]
    fn page_title_wins_over_tab_title() {
        let s = store();
        let mut c = cap_page(page(Some("De la página"), LONG_TEXT), None);
        c.title = Some("De la pestaña".into());
        assert_eq!(capture(&s, c).unwrap().bookmark.title, "De la página");
    }

    #[test]
    fn blocked_capture_creates_failed_blocked_bookmark() {
        let s = store();
        let mut c = cap_page(page(Some("Un momento"), BLOCK_MD), None);
        c.title = Some("Un momento".into());
        let r = capture(&s, c).unwrap();
        assert!(r.created && r.blocked);
        let b = r.bookmark;
        assert_eq!(b.status, Status::Failed);
        assert_eq!(b.error_kind, Some(ErrorKind::Blocked));
        assert_eq!(
            (b.title.as_str(), b.content, b.meta),
            (URL, None, PageMeta::default())
        );
    }

    /// A short text with a blocking phrase does not count when the page already has enough content.
    #[test]
    fn short_blocking_text_is_ignored_when_page_is_not_thin() {
        let s = store();
        let r = capture(
            &s,
            cap_page(page(None, LONG_TEXT), Some("Just a moment...")),
        )
        .unwrap();
        assert!(!r.blocked);
        assert_eq!(r.bookmark.status, Status::Pending);
        assert_eq!(r.bookmark.content.as_deref(), Some(LONG_TEXT));
    }

    #[test]
    fn blocked_capture_never_overwrites_ready() {
        let s = store();
        seed(&s, Status::Ready, Some("contenido propio"));
        let r = capture(&s, cap_page(page(None, BLOCK_MD), None)).unwrap();
        assert!(!r.blocked);
        assert_eq!(r.bookmark.status, Status::Ready);
        assert_eq!(r.bookmark.content.as_deref(), Some("contenido propio"));
    }

    #[test]
    fn later_good_capture_rescues_blocked() {
        let s = store();
        capture(&s, cap_page(page(None, BLOCK_MD), None)).unwrap();
        let r = capture(&s, cap_page(page(Some("Bueno"), LONG_TEXT), None)).unwrap();
        assert!(!r.created && !r.blocked);
        assert_eq!(r.bookmark.status, Status::Pending);
        assert_eq!(r.bookmark.error_kind, None);
        assert_eq!(r.bookmark.content.as_deref(), Some(LONG_TEXT));
        assert_eq!(r.bookmark.title, "Bueno");
    }

    #[test]
    fn duplicate_ready_only_fills_missing_meta() {
        let s = store();
        seed(&s, Status::Ready, Some("contenido propio"));
        let r = capture(&s, cap_page(page(Some("Otro"), LONG_TEXT), None)).unwrap();
        assert_eq!(r.bookmark.content.as_deref(), Some("contenido propio"));
        assert_eq!(r.bookmark.title, "Mío");
        assert_eq!(r.bookmark.meta.author.as_deref(), Some("Ana"));
        assert_eq!(r.bookmark.meta.extractor, None);
        assert_eq!(r.bookmark.status, Status::Ready);
    }

    #[test]
    fn thin_page_without_text_still_stores_metadata() {
        let s = store();
        let r = capture(&s, cap_page(page(None, "poco"), None)).unwrap();
        assert_eq!(r.bookmark.content, None);
        assert_eq!(r.bookmark.meta.author.as_deref(), Some("Ana"));
        let s = store();
        seed(&s, Status::Ready, Some("contenido propio"));
        let r = capture(&s, cap_page(page(None, "poco"), None)).unwrap();
        assert_eq!(r.bookmark.meta.author.as_deref(), Some("Ana"));
        assert_eq!(r.bookmark.content.as_deref(), Some("contenido propio"));
    }

    #[test]
    fn html_field_is_ignored_and_text_is_used() {
        let s = store();
        let body = serde_json::json!({
            "url": URL,
            "html": "<article><p>Cuerpo de una extensión v0.5</p></article>",
            "text": LONG_TEXT,
        });
        let c: Capture = serde_json::from_value(body).unwrap();
        let r = capture(&s, c).unwrap();
        assert_eq!(r.bookmark.content.as_deref(), Some(LONG_TEXT));
    }

    #[test]
    fn blank_title_falls_back_to_page_title() {
        let s = store();
        let mut c = cap_page(page(Some("Título del artículo"), LONG_TEXT), None);
        c.title = Some("   ".into());
        assert_eq!(
            capture(&s, c).unwrap().bookmark.title,
            "Título del artículo"
        );
    }

    #[test]
    fn rejects_non_http_urls() {
        let s = store();
        let bad = |url: &str| Capture {
            url: url.into(),
            ..Capture::default()
        };
        assert!(matches!(
            capture(&s, bad("no es una url")),
            Err(CaptureError::InvalidUrl(Msg::InvalidUrl))
        ));
        for url in [
            "ftp://a.b/c",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "chrome://settings",
            "about:blank",
        ] {
            assert!(
                matches!(
                    capture(&s, bad(url)),
                    Err(CaptureError::InvalidUrl(Msg::UnsupportedScheme))
                ),
                "{url}"
            );
        }
        assert!(lock(&s).unwrap().list().unwrap().is_empty());
    }

    #[test]
    fn duplicate_ready_is_untouched() {
        let s = store();
        let id = seed(&s, Status::Ready, Some("contenido propio"));
        let r = capture(&s, cap(Some(OTHER_TEXT))).unwrap();
        assert!(!r.created);
        assert_eq!(r.bookmark.id, id);
        assert_eq!(r.bookmark.content.as_deref(), Some("contenido propio"));
        assert_eq!(r.bookmark.title, "Mío");
        assert_eq!(r.bookmark.status, Status::Ready);
    }

    #[test]
    fn duplicate_failed_is_refilled_and_requeued() {
        let s = store();
        seed(&s, Status::Failed, Some("viejo"));
        let r = capture(&s, cap(Some(OTHER_TEXT))).unwrap();
        assert!(!r.created);
        assert!(r.bookmark.content.unwrap().contains("Contenido distinto"));
        assert_eq!(r.bookmark.status, Status::Pending);
        assert_eq!(r.bookmark.error, None);
    }

    #[test]
    fn duplicate_without_content_is_filled() {
        let s = store();
        seed(&s, Status::Pending, Some("  "));
        let r = capture(&s, cap(Some(OTHER_TEXT))).unwrap();
        assert!(r.bookmark.content.unwrap().contains("Contenido distinto"));
    }

    #[test]
    fn duplicate_without_new_content_is_untouched() {
        let s = store();
        seed(&s, Status::Failed, None);
        let r = capture(&s, cap(None)).unwrap();
        assert_eq!(r.bookmark.status, Status::Failed);
        assert_eq!(
            r.bookmark
                .error
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("boom")
        );
    }

    #[test]
    fn thin_text_stores_no_content() {
        let s = store();
        let r = capture(&s, cap(Some("poco"))).unwrap();
        assert_eq!(r.bookmark.content, None);
    }

    #[test]
    fn utm_variants_are_one_bookmark() {
        let s = store();
        capture(&s, cap(None)).unwrap();
        let mut c = cap(None);
        c.url = format!("{URL}?utm_source=x");
        assert!(!capture(&s, c).unwrap().created);
    }
}
