//! Download fallback: loads the URL in a hidden window, takes the HTML already rendered by
//! JavaScript and hands it to an `HtmlExtractor` (the `parser` window).

use crate::parser_window::PARSER_WINDOW;
use crate::state::AppState;
use anyhow::Result;
use marks_core::blocked::{BlockSignals, is_blocked};
use marks_core::download::is_remote_http;
use marks_core::extract::{Fetcher, HtmlExtractor, MIN_CONTENT_CHARS, Page};
use marks_core::msg::{Msg, MsgExt, Stage};
use std::future::Future;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::webview::{NewWindowResponse, PageLoadEvent, PermissionResponse};
use tauri::{AppHandle, Manager, Runtime, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tokio::sync::oneshot;

/// Hidden window label; it is in no capability, so it has no IPC.
pub const EXTRACTOR_WINDOW: &str = "extractor";
/// Hidden window labels: none opens while another remains (`close_stale`).
const HIDDEN_WINDOWS: [&str; 2] = [EXTRACTOR_WINDOW, PARSER_WINDOW];
/// How often the rendered text is measured after `Finished`, while the page's JavaScript
/// finishes rendering.
const SETTLE_POLL_INTERVAL: Duration = Duration::from_millis(250);
/// Cap on waiting for the page to settle, if what is left of `WEBVIEW_TIMEOUT` allows it
/// (`settle_budget`).
const SETTLE_MAX: Duration = Duration::from_secs(8);
/// Share of `WEBVIEW_TIMEOUT` the settle wait always leaves for reading the HTML.
const EVAL_RESERVE: Duration = Duration::from_secs(2);
/// Equal consecutive samples that are enough to call a page with content settled.
const STABLE_SAMPLES: usize = 4;
/// Equal consecutive samples after which we stop waiting for a page that is still thin
/// (3 s without changes, more than the fixed 2 s used before).
const THIN_QUIET_SAMPLES: usize = 12;
/// Visible text without whitespace: the same criterion as `is_thin`. Returns a number, which
/// `eval_with_callback` serializes as JSON.
const TEXT_LEN_EXPRESSION: &str =
    "document.body ? document.body.innerText.replace(/\\s/g, '').length : 0";
/// Timeout for the whole webview load, the same as the HTTP download's.
const WEBVIEW_TIMEOUT: Duration = Duration::from_secs(30);
/// How often to check whether a previous hidden window is gone.
const STALE_POLL_INTERVAL: Duration = Duration::from_millis(50);
/// After waiting this long for a window to disappear, a warning is logged.
const STALE_WARN_AFTER: Duration = Duration::from_secs(1);
/// Cap on waiting for a hidden window to disappear once its work is done.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(2);
/// Synchronous expression: `eval_with_callback` does not await promises.
const HTML_EXPRESSION: &str = "document.documentElement.outerHTML";

/// The expired-timeout `Msg` of a hidden load, with the current stage.
pub(crate) type Timeout = fn(Stage) -> Msg;
const WEBVIEW_TIMED_OUT: Timeout = |stage| Msg::WebviewTimeout { stage };

/// Downloads with a hidden app webview and extracts with `extractor`.
pub struct WebviewFetcher<E: HtmlExtractor> {
    app: AppHandle,
    extractor: E,
}

impl<E: HtmlExtractor> WebviewFetcher<E> {
    pub fn new(app: AppHandle, extractor: E) -> Self {
        Self { app, extractor }
    }

    /// Final URL and rendered HTML of `url`, in the `extractor` window (`in_hidden_window`).
    pub async fn load_html(&self, url: &str) -> Result<(String, String)> {
        let parsed = Url::parse(url).msg(Msg::WebviewUrl)?;
        if !is_remote_http(&parsed) {
            return Err(Msg::WebviewUrl.err());
        }
        log::info!(
            "webview: carga oculta de {}",
            parsed.host_str().unwrap_or_default()
        );
        let stage = CurrentStage::default();
        let work = load_page(&self.app, parsed, &stage);
        in_hidden_window(
            &self.app,
            EXTRACTOR_WINDOW,
            WEBVIEW_TIMEOUT,
            &stage,
            WEBVIEW_TIMED_OUT,
            work,
        )
        .await
        // `FallbackFetcher` drops this error in favor of the HTTP one, so log it here.
        .inspect_err(|e| log::warn!("webview: {e:#}"))
    }
}

#[async_trait::async_trait]
impl<E: HtmlExtractor> Fetcher for WebviewFetcher<E> {
    async fn fetch(&self, url: &str) -> Result<Page> {
        let (final_url, html) = self.load_html(url).await?;
        // The hidden-window lock is already released: `parser` takes it again.
        let page = self.extractor.extract(&final_url, &html).await?;
        // The webview exposes no headers: only the final URL and the text count.
        let blocked = is_blocked(&BlockSignals {
            status: None,
            headers: &[],
            final_url: Some(&final_url),
            markdown: &page.markdown,
            word_count: page.meta.word_count,
        });
        if blocked {
            return Err(Msg::Blocked.err());
        }
        Ok(page)
    }
}

/// Decodes the `eval_with_callback` result, which arrives serialized as JSON.
fn decode_html(json: &str) -> Result<String> {
    serde_json::from_str(json).msg(Msg::WebviewDecode)
}

/// Closes a leftover previous window, opens the new one and returns the final URL and the
/// settled HTML.
async fn load_page(app: &AppHandle, url: Url, stage: &CurrentStage) -> Result<(String, String)> {
    // Starts with `fetch`'s timeout: what is left of it bounds the settle wait.
    let started = Instant::now();
    close_stale(app, EXTRACTOR_WINDOW).await;
    let (window, loaded) = open_hidden(
        app,
        EXTRACTOR_WINDOW,
        WebviewUrl::External(url),
        is_remote_http,
    )?;
    stage.set(Stage::Load);
    loaded.await.msg(Msg::WebviewClosedLoading)?;
    stage.set(Stage::Settle);
    let budget = settle_budget(WEBVIEW_TIMEOUT.saturating_sub(started.elapsed()));
    // The timeout also covers a measurement that never answers: go on with the HTML as it is.
    if tokio::time::timeout(budget, wait_settled(&window))
        .await
        .is_err()
    {
        log::info!("webview: sin asentarse en {budget:?}");
    }
    stage.set(Stage::Eval);
    let html = decode_html(
        &eval(&window, HTML_EXPRESSION)
            .await
            .msg(Msg::WebviewClosedHtml)?,
    )?;
    Ok((window.url()?.to_string(), html))
}

/// How long the settle wait may last with `remaining` of the timeout left: at most `SETTLE_MAX`,
/// always leaving `EVAL_RESERVE` to read the HTML.
fn settle_budget(remaining: Duration) -> Duration {
    remaining.saturating_sub(EVAL_RESERVE).min(SETTLE_MAX)
}

/// Measures the rendered text every `SETTLE_POLL_INTERVAL` until `is_settled` says so; the caller
/// sets the timeout. A failed measurement is not an error: go on with the HTML as it is.
async fn wait_settled(window: &WebviewWindow) {
    let started = Instant::now();
    let mut samples = Vec::new();
    loop {
        tokio::time::sleep(SETTLE_POLL_INTERVAL).await;
        let Ok(json) = eval(window, TEXT_LEN_EXPRESSION).await else {
            return;
        };
        let Ok(len) = serde_json::from_str::<usize>(&json) else {
            return;
        };
        samples.push(len);
        if is_settled(&samples) {
            break;
        }
    }
    log::info!(
        "webview: asentada en {:?} con {} caracteres de texto",
        started.elapsed(),
        samples.last().copied().unwrap_or_default()
    );
}

/// Decides from the text samples (the last is the newest) whether the page has settled: with
/// content, unchanged for `STABLE_SAMPLES` samples; thin, unchanged for `THIN_QUIET_SAMPLES`.
fn is_settled(samples: &[usize]) -> bool {
    let Some(&last) = samples.last() else {
        return false;
    };
    let needed = if last >= MIN_CONTENT_CHARS {
        STABLE_SAMPLES
    } else {
        THIN_QUIET_SAMPLES
    };
    samples.len() >= needed && samples[samples.len() - needed..].iter().all(|&n| n == last)
}

/// Current stage of a hidden load, to name it if the timeout expires.
pub(crate) struct CurrentStage(Mutex<Stage>);

impl Default for CurrentStage {
    fn default() -> Self {
        Self(Mutex::new(Stage::Open))
    }
}

impl CurrentStage {
    pub(crate) fn set(&self, stage: Stage) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = stage;
    }

    /// Expired-timeout error carrying the current stage.
    pub(crate) fn timed_out(&self, timeout: Timeout) -> anyhow::Error {
        timeout(*self.0.lock().unwrap_or_else(|e| e.into_inner())).err()
    }
}

/// Runs `work`, which opens the hidden window `label`, under the single hidden-window lock and
/// with timeout `limit`. Closes the window on success, error or timeout, and waits for it to
/// disappear before releasing the lock: whoever waits for the lock, the worker or a command,
/// finds the previous window closed and never destroys it halfway.
pub(crate) async fn in_hidden_window<R: Runtime, T>(
    app: &AppHandle<R>,
    label: &str,
    limit: Duration,
    stage: &CurrentStage,
    timeout: Timeout,
    work: impl Future<Output = Result<T>>,
) -> Result<T> {
    let state = app.state::<AppState>();
    // Outside the timeout: the lock holder is already bounded by its own, and destroying the window
    // without holding it would close another task's window.
    let _hidden = state.hidden_window.lock().await;
    let result = tokio::time::timeout(limit, work)
        .await
        .map_err(|_| stage.timed_out(timeout))
        .and_then(|r| r);
    close_window(app, label).await;
    result
}

/// Destroys a leftover `label` window and waits until no hidden window remains, including the
/// one with the other label.
///
/// tao names each window by its NSWindow address and tauri-runtime-wry does not forget it on
/// close: a window created before another's `Destroyed` arrives can inherit that address and
/// receive the event. The new one is torn down at birth and both labels stay registered until
/// restart, so no hidden window opens while another is being torn down.
///
/// If any remains after `STALE_WARN_AFTER`, all of them are destroyed once, including the other
/// label's: the caller holds the single lock (`in_hidden_window`), so what is left is a leftover,
/// not another task's window. Without this, a leftover of one label would also stall the other,
/// and with it HTTP extraction, until restart.
pub(crate) async fn close_stale<R: Runtime>(app: &AppHandle<R>, label: &str) {
    destroy_window(app, label);
    let started = Instant::now();
    let mut warned = false;
    while any_hidden(app.webview_windows().keys()) {
        if !warned && started.elapsed() > STALE_WARN_AFTER {
            warned = true;
            let open: Vec<String> = app.webview_windows().into_keys().collect();
            log::warn!(
                "webview: queda una ventana oculta tras {STALE_WARN_AFTER:?} cerrando {label}; se destruyen todas; abiertas: {open:?}"
            );
            for hidden in HIDDEN_WINDOWS {
                destroy_window(app, hidden);
            }
        }
        tokio::time::sleep(STALE_POLL_INTERVAL).await;
    }
}

fn any_hidden<'a>(mut labels: impl Iterator<Item = &'a String>) -> bool {
    labels.any(|label| HIDDEN_WINDOWS.contains(&label.as_str()))
}

/// Destroys the `label` window and waits, with a timeout, for it to disappear before releasing
/// the lock. If the timeout expires, the next open keeps waiting in `close_stale`.
async fn close_window<R: Runtime>(app: &AppHandle<R>, label: &str) {
    if tokio::time::timeout(CLOSE_TIMEOUT, close_stale(app, label))
        .await
        .is_err()
    {
        log::warn!("webview: la ventana {label} no desapareció en {CLOSE_TIMEOUT:?}");
    }
}

/// Evaluates a synchronous expression and returns its result serialized as JSON.
pub(crate) async fn eval(window: &WebviewWindow, expression: &str) -> Result<String> {
    let (tx, rx) = oneshot::channel();
    let tx = Mutex::new(Some(tx));
    window.eval_with_callback(expression, move |json| {
        if let Some(tx) = take(&tx) {
            let _ = tx.send(json);
        }
    })?;
    Ok(rx.await?)
}

/// The one place that decides what the hidden `extractor` and `parser` windows are like (hidden,
/// incognito, unfocused, blocks, permissions). Only the label, the URL and the allowed navigation
/// vary. Returns the window and a `Finished` signal.
pub(crate) fn open_hidden(
    app: &AppHandle,
    label: &str,
    url: WebviewUrl,
    allow: fn(&Url) -> bool,
) -> Result<(WebviewWindow, oneshot::Receiver<()>)> {
    let (tx, rx) = oneshot::channel();
    let tx = Mutex::new(Some(tx));
    let window = WebviewWindowBuilder::new(app, label, url)
        .visible(false)
        .focused(false)
        .focusable(false)
        .incognito(true)
        .on_navigation(allow)
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_download(|_, _| false)
        // Without a handler, wry grants camera and microphone by default.
        .on_permission_request(|_, _| PermissionResponse::Deny)
        .on_page_load(move |_, payload| {
            if payload.event() == PageLoadEvent::Finished
                && let Some(tx) = take(&tx)
            {
                let _ = tx.send(());
            }
        })
        .build()?;
    Ok((window, rx))
}

/// Takes the sender of a one-shot signal out of an `Fn` callback.
fn take<T>(slot: &Mutex<Option<T>>) -> Option<T> {
    slot.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// Destroys the `label` window if it exists; a failure is only logged.
fn destroy_window<R: Runtime>(app: &AppHandle<R>, label: &str) {
    if let Some(window) = app.get_webview_window(label)
        && let Err(e) = window.destroy()
    {
        log::warn!("webview: no se pudo cerrar la ventana {label}: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CurrentStage, EXTRACTOR_WINDOW, Stage, WEBVIEW_TIMED_OUT, any_hidden, decode_html,
        is_settled, settle_budget,
    };
    use crate::parser_window::PARSER_WINDOW;
    use marks_core::msg::{Msg, failure_msg};
    use marks_core::profiles::ProfileSeed;

    const TEST_SEED: ProfileSeed = ProfileSeed {
        name: "Personal",
        initials: "PE",
    };

    const CAPABILITIES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/capabilities");
    const TAURI_CONF: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json");

    #[test]
    fn html_result_is_decoded_from_json() {
        assert_eq!(
            decode_html("\"<html>\\n</html>\"").expect("JSON válido"),
            "<html>\n</html>"
        );
        for bad in ["", "<html>", "null"] {
            let err = decode_html(bad).expect_err("debería fallar");
            assert_eq!(
                err.to_string(),
                Msg::WebviewDecode.to_string(),
                "entrada {bad:?}"
            );
        }
    }

    /// The error carries the current stage as data, not as pasted text.
    #[test]
    fn timeout_names_the_stage_in_progress() {
        let stage = CurrentStage::default();
        assert_eq!(
            failure_msg(&stage.timed_out(WEBVIEW_TIMED_OUT)),
            Some(&Msg::WebviewTimeout { stage: Stage::Open })
        );
        stage.set(Stage::Eval);
        assert_eq!(
            failure_msg(&stage.timed_out(WEBVIEW_TIMED_OUT)),
            Some(&Msg::WebviewTimeout { stage: Stage::Eval })
        );
    }

    /// With content it is enough to stay unchanged for `STABLE_SAMPLES` samples; thin, for
    /// `THIN_QUIET_SAMPLES`.
    #[test]
    fn settled_needs_content_and_stability() {
        use super::{MIN_CONTENT_CHARS as MIN, STABLE_SAMPLES, THIN_QUIET_SAMPLES};
        assert!(!is_settled(&[]));
        assert!(!is_settled(&[MIN]));
        assert!(is_settled(&[MIN; STABLE_SAMPLES]));
        assert!(!is_settled(&[MIN; STABLE_SAMPLES - 1]));
        assert!(!is_settled(&[MIN, MIN, MIN, MIN + 50]));
        assert!(is_settled(&[10, MIN + 50, MIN + 50, MIN + 50, MIN + 50]));
        assert!(!is_settled(&[0; STABLE_SAMPLES]));
        assert!(!is_settled(&[0; THIN_QUIET_SAMPLES - 1]));
        assert!(is_settled(&[0; THIN_QUIET_SAMPLES]));
        assert!(!is_settled(&[5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
    }

    /// The settle wait never exceeds `SETTLE_MAX` and always leaves `EVAL_RESERVE` of the timeout to
    /// read the HTML, also when the load already used up almost all of `WEBVIEW_TIMEOUT`.
    #[test]
    fn settle_budget_fits_inside_the_timeout() {
        use super::{EVAL_RESERVE, SETTLE_MAX, WEBVIEW_TIMEOUT};
        use std::time::Duration;
        assert_eq!(settle_budget(WEBVIEW_TIMEOUT), SETTLE_MAX);
        assert_eq!(
            settle_budget(Duration::from_secs(5)),
            Duration::from_secs(5) - EVAL_RESERVE
        );
        assert_eq!(settle_budget(EVAL_RESERVE / 2), Duration::ZERO);
        for ms in (0..=WEBVIEW_TIMEOUT.as_millis() as u64).step_by(250) {
            let remaining = Duration::from_millis(ms);
            let budget = settle_budget(remaining);
            assert!(budget <= SETTLE_MAX, "{remaining:?}");
            assert!(
                budget + EVAL_RESERVE <= remaining.max(EVAL_RESERVE),
                "{remaining:?}"
            );
        }
    }

    /// A hidden window of either label holds back the next one; the main window does not.
    #[test]
    fn any_hidden_window_blocks_the_next_one() {
        let labels = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(!any_hidden(labels(&[]).iter()));
        assert!(!any_hidden(labels(&["main"]).iter()));
        assert!(any_hidden(labels(&["main", EXTRACTOR_WINDOW]).iter()));
        assert!(any_hidden(labels(&[PARSER_WINDOW]).iter()));
    }

    #[test]
    fn only_main_has_capabilities() {
        let files: Vec<_> = std::fs::read_dir(CAPABILITIES_DIR)
            .expect("directorio de capabilities")
            .map(|entry| entry.expect("entrada").file_name())
            .collect();
        assert_eq!(files, ["default.json"]);
        let json = std::fs::read_to_string(format!("{CAPABILITIES_DIR}/default.json"))
            .expect("capability por defecto");
        let value: serde_json::Value = serde_json::from_str(&json).expect("JSON válido");
        assert_eq!(value["windows"], serde_json::json!(["main"]));
        // An inline capability in `tauri.conf.json` would bypass the directory check.
        let conf = std::fs::read_to_string(TAURI_CONF).expect("tauri.conf.json");
        let conf: serde_json::Value = serde_json::from_str(&conf).expect("JSON válido");
        let inline = &conf["app"]["security"]["capabilities"];
        assert!(
            inline.is_null() || inline == &serde_json::json!([]),
            "capabilities en línea: {inline}"
        );
    }

    /// With the worker in one hidden window, a command that opens the other waits for the lock: its
    /// work, which starts by destroying leftovers (`close_stale`), does not begin until the first has
    /// finished and closed its window, so it never destroys it halfway. The test runtime does not
    /// tear down windows, so the order is checked, not the window.
    #[tokio::test]
    async fn second_hidden_window_user_waits_for_the_first() {
        use super::in_hidden_window;
        use crate::state::AppState;
        use marks_core::profiles::Library;
        use std::time::{Duration, Instant};
        use tauri::Manager;

        const LIMIT: Duration = Duration::from_secs(10);
        const WORK: Duration = Duration::from_millis(300);
        let dir = std::env::temp_dir().join(format!("marks-hidden-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let app = tauri::test::mock_app();
        app.manage(AppState::new(
            Library::open(&dir.join("marks.db"), &TEST_SEED).expect("library"),
        ));
        let app = app.handle();
        let (opened, wait_opened) = tokio::sync::oneshot::channel();
        let (first_stage, second_stage) = (CurrentStage::default(), CurrentStage::default());

        let first = in_hidden_window(
            app,
            EXTRACTOR_WINDOW,
            LIMIT,
            &first_stage,
            WEBVIEW_TIMED_OUT,
            async {
                let _ = opened.send(());
                tokio::time::sleep(WORK).await;
                Ok(Instant::now())
            },
        );
        let second = async {
            wait_opened.await.expect("la primera empezó");
            in_hidden_window(
                app,
                PARSER_WINDOW,
                LIMIT,
                &second_stage,
                WEBVIEW_TIMED_OUT,
                async { Ok(Instant::now()) },
            )
            .await
        };
        let (first, second) = tokio::join!(first, second);
        let (first_done, second_started) = (first.expect("primera"), second.expect("segunda"));
        assert!(
            second_started >= first_done,
            "la segunda empezó mientras la primera tenía su ventana abierta"
        );
        std::fs::remove_dir_all(dir).expect("borrar temp dir");
    }
}
