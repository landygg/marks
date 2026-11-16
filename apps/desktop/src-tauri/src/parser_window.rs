//! Hidden `parser` window: runs Defuddle (`extract.html`, a Vite entry) on the HTML the app
//! downloads, over HTTP or through the `extractor` window, and cleans that HTML for AI extraction
//! (`cleanHtml`).

use crate::webview_fetcher::{
    CurrentStage, Timeout, WebviewFetcher, close_stale, eval, in_hidden_window, open_hidden,
};
use anyhow::Result;
use marks_core::extract::{HtmlExtractor, Page};
use marks_core::msg::{Msg, MsgExt, Stage};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::time::Duration;
use tauri::{AppHandle, Url, WebviewUrl};

/// Window label; it is in no capability, so it has no IPC.
pub const PARSER_WINDOW: &str = "parser";
/// Timeout for the whole extraction, from opening the window to having the page.
const PARSE_TIMEOUT: Duration = Duration::from_secs(30);
/// The app's local page that loads `marks-extract`.
const PARSER_PAGE: &str = "extract.html";
/// `PARSER_PAGE` URL in release.
const PARSER_URL: &str = "tauri://localhost/extract.html";
/// `PARSER_PAGE` URL in development, served by Vite (`devUrl` in `tauri.conf.json`).
const PARSER_DEV_URL: &str = "http://127.0.0.1:1420/extract.html";
/// How often to check whether the `extract.html` module is ready.
const READY_POLL_INTERVAL: Duration = Duration::from_millis(10);
/// `extract.ts` functions called with `(html, url)`.
const EXTRACT_FUNCTION: &str = "marksExtractHtml";
const CLEAN_FUNCTION: &str = "marksCleanHtml";
/// Synchronous expression telling whether `marksExtractHtml` and `marksCleanHtml` exist yet.
const READY_EXPRESSION: &str = "globalThis.marksReady === true";
/// `READY_EXPRESSION`'s answer serialized as JSON when ready.
const READY: &str = "true";

const PARSER_TIMED_OUT: Timeout = |stage| Msg::ParserTimeout { stage };

/// `HtmlExtractor` over the `parser` window; cheap to clone.
#[derive(Clone)]
pub struct ParserWindow {
    app: AppHandle,
}

impl ParserWindow {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// Clean HTML for AI extraction, with the same lock, timeout and close as `extract`.
    pub async fn clean_html(&self, url: &str, html: &str) -> Result<String> {
        let reply = self.run(CLEAN_FUNCTION, url, html).await?;
        decode(&reply, Msg::ParserClean)
    }

    /// Calls `function(html, url)` in the window (`in_hidden_window`) and returns its raw answer.
    async fn run(&self, function: &str, url: &str, html: &str) -> Result<String> {
        let stage = CurrentStage::default();
        let work = call(&self.app, function, url, html, &stage);
        in_hidden_window(
            &self.app,
            PARSER_WINDOW,
            PARSE_TIMEOUT,
            &stage,
            PARSER_TIMED_OUT,
            work,
        )
        .await
        .inspect_err(|e| log::warn!("parser: {e:#}"))
    }
}

#[async_trait::async_trait]
impl HtmlExtractor for ParserWindow {
    async fn extract(&self, url: &str, html: &str) -> Result<Page> {
        let reply = self.run(EXTRACT_FUNCTION, url, html).await?;
        decode(&reply, Msg::ParserPage)
    }
}

/// Input for "Extract with AI": the final URL and the page rendered in `extractor` and cleaned in
/// `parser`, one window after the other. The final URL is used to detect an anti-bot check
/// (`pipeline::extract_with_ai`).
pub async fn load_clean_html(app: &AppHandle, url: String) -> Result<(String, String)> {
    let parser = ParserWindow::new(app.clone());
    let (final_url, html) = WebviewFetcher::new(app.clone(), parser.clone())
        .load_html(&url)
        .await?;
    let clean = parser.clean_html(&final_url, &html).await?;
    Ok((final_url, clean))
}

/// Opens the window, waits for the module to be ready and calls `function`.
async fn call(
    app: &AppHandle,
    function: &str,
    url: &str,
    html: &str,
    stage: &CurrentStage,
) -> Result<String> {
    close_stale(app, PARSER_WINDOW).await;
    let (window, loaded) = open_hidden(
        app,
        PARSER_WINDOW,
        WebviewUrl::App(PARSER_PAGE.into()),
        is_parser_url,
    )?;
    stage.set(Stage::Load);
    loaded.await.msg(Msg::ParserClosedLoading)?;
    stage.set(Stage::Ready);
    while eval(&window, READY_EXPRESSION)
        .await
        .msg(Msg::ParserClosedLoading)?
        != READY
    {
        tokio::time::sleep(READY_POLL_INTERVAL).await;
    }
    stage.set(Stage::Eval);
    // JSON literals, never the raw HTML inside the code.
    let expression = format!(
        "JSON.stringify({function}({}, {}))",
        serde_json::to_string(html)?,
        serde_json::to_string(url)?
    );
    eval(&window, &expression).await.msg(Msg::ParserClosedPage)
}

/// Only our own page: `tauri://` in release, the Vite one too in development.
fn is_parser_url(url: &Url) -> bool {
    url.as_str() == PARSER_URL || (cfg!(debug_assertions) && url.as_str() == PARSER_DEV_URL)
}

/// What the `extract.ts` functions return: the result or the error they threw.
#[derive(Deserialize)]
#[serde(untagged)]
enum Reply<T> {
    Failed { error: String },
    Done(T),
}

/// Double JSON: `eval_with_callback` serializes the string `JSON.stringify` returns. `message` is
/// the error for a failed answer or one that is not a `T`.
fn decode<T: DeserializeOwned>(json: &str, message: Msg) -> Result<T> {
    let inner: String = serde_json::from_str(json).msg(message.clone())?;
    match serde_json::from_str(&inner).msg(message.clone())? {
        Reply::Done(value) => Ok(value),
        Reply::Failed { error } => Err(anyhow::anyhow!(error)).msg(message),
    }
}

#[cfg(test)]
mod tests {
    use super::{decode, is_parser_url};
    use marks_core::extract::Page;
    use marks_core::msg::Msg;
    use tauri::Url;

    const FIXTURE: &str = include_str!("../../../../packages/extract/fixtures/extracted.json");
    const EXTRACT_TS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../src/extract.ts");
    const EXTRACT_HTML: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../extract.html");

    #[test]
    fn decode_page_rejects_errors_and_garbage() {
        for bad in ["null", "\"{\\\"error\\\":\\\"x\\\"}\"", "<html>"] {
            let err = decode::<Page>(bad, Msg::ParserPage).expect_err("debería fallar");
            assert_eq!(
                err.to_string(),
                Msg::ParserPage.to_string(),
                "entrada {bad:?}"
            );
        }
        let doubled = serde_json::to_string(FIXTURE).expect("cadena JSON");
        let expected: Page = serde_json::from_str(FIXTURE).expect("fixture válido");
        assert_eq!(
            decode::<Page>(&doubled, Msg::ParserPage).expect("página"),
            expected
        );
    }

    #[test]
    fn decode_clean_rejects_errors_and_non_strings() {
        for bad in ["null", "\"{\\\"error\\\":\\\"x\\\"}\"", "\"{}\"", "<html>"] {
            let err = decode::<String>(bad, Msg::ParserClean).expect_err("debería fallar");
            assert_eq!(
                err.to_string(),
                Msg::ParserClean.to_string(),
                "entrada {bad:?}"
            );
        }
        let doubled = serde_json::to_string("\"<p>hola</p>\"").expect("cadena JSON");
        assert_eq!(
            decode::<String>(&doubled, Msg::ParserClean).expect("HTML"),
            "<p>hola</p>"
        );
    }

    #[test]
    fn parser_url_is_only_the_local_page() {
        let url = |s: &str| Url::parse(s).expect("URL válida");
        assert!(is_parser_url(&url("tauri://localhost/extract.html")));
        // Tests run in debug: the Vite server page is allowed too.
        assert!(is_parser_url(&url("http://127.0.0.1:1420/extract.html")));
        for bad in [
            "tauri://localhost/index.html",
            "https://example.com/",
            "http://127.0.0.1:47821/",
        ] {
            assert!(!is_parser_url(&url(bad)), "{bad}");
        }
    }

    #[test]
    fn extract_entry_has_no_tauri_imports() {
        for path in [EXTRACT_TS, EXTRACT_HTML] {
            let source = std::fs::read_to_string(path).expect("entrada de extracción");
            assert!(!source.contains("@tauri-apps/"), "{path} importa Tauri");
        }
    }
}
