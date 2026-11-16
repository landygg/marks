//! Background worker: processes pending bookmarks one at a time, then downloads their favicons.

use crate::extracting;
use crate::notifications::{notify, warn};
use crate::parser_window::ParserWindow;
use crate::state::AppState;
use crate::webview_fetcher::WebviewFetcher;
use marks_core::download::download_favicon;
use marks_core::extract::{FallbackFetcher, Fetcher, HttpFetcher};
use marks_core::notifications::{NewNotification, Severity, bookmark_failure};
use marks_core::pipeline::{favicon_step, process_next};
use marks_core::store::{Store, lock};
use marks_i18n::{AppText, Localize, chain};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Payload-less event emitted when the library changes; same value as `LIBRARY_CHANGED` in `src/lib/api.ts`.
pub const LIBRARY_CHANGED: &str = "library-changed";
/// Fallback polling in case nobody wakes the worker.
const POLL_INTERVAL: Duration = Duration::from_secs(15);

/// Spawns the worker loop on Tauri's runtime.
pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(run_loop(app));
}

/// The app's fetcher, used by the worker and by "Extract again".
pub type AppFetcher = FallbackFetcher<HttpFetcher<ParserWindow>, WebviewFetcher<ParserWindow>>;

/// HTTP first; the hidden webview only if it fails or yields thin content. Both extract in the
/// `parser` window.
pub fn app_fetcher(app: &AppHandle) -> AppFetcher {
    let parser = ParserWindow::new(app.clone());
    FallbackFetcher {
        primary: HttpFetcher {
            extractor: parser.clone(),
        },
        secondary: WebviewFetcher::new(app.clone(), parser),
    }
}

/// Drains the queue, with or without a model, and waits for a wake-up or the next poll.
async fn run_loop(app: AppHandle) {
    let fetcher = app_fetcher(&app);
    let state = app.state::<AppState>();
    loop {
        drain(&app, &state, &fetcher).await;
        tokio::select! {
            _ = state.wake.notified() => {}
            _ = tokio::time::sleep(POLL_INTERVAL) => {}
        }
    }
}

/// Rotates turns among profiles until a full pass makes no progress. Rereads the list on every
/// pass: a new profile joins and a deleted one leaves.
async fn drain(app: &AppHandle, state: &AppState, fetcher: &dyn Fetcher) {
    // Profiles that failed in this call: not retried until the next wake-up or poll.
    let broken = Mutex::new(HashSet::new());
    loop {
        let profiles = match state.library.profiles() {
            Ok(profiles) => profiles,
            Err(e) => {
                fail(app, None, AppText::ProfilesFailed, &marks_i18n::error(&e));
                return;
            }
        };
        let ids: Vec<i64> = {
            let skip = lock_set(&broken);
            profiles
                .iter()
                .map(|p| p.id)
                .filter(|id| !skip.contains(id))
                .collect()
        };
        if !take_turns(&ids, |id| step(app, state, id, fetcher, &broken)).await {
            return;
        }
    }
}

/// One pending bookmark and one favicon from the profile (both if any); `true` if it did anything.
/// A failure is notified and counts as no progress, so a broken profile does not keep the pass spinning.
async fn step(
    app: &AppHandle,
    state: &AppState,
    profile: i64,
    fetcher: &dyn Fetcher,
    broken: &Mutex<HashSet<i64>>,
) -> bool {
    let mut advanced = false;
    // The store is resolved on every step: if the profile is deleted, only the work in progress finishes.
    if let Some(store) = profile_store(app, state, profile) {
        // Also on every step: turning AI on or off takes effect from the next bookmark.
        let model = state.model();
        match process_next(&store, fetcher, model.as_deref(), &|id| {
            Box::new(extracting::mark(app, profile, id))
        })
        .await
        {
            Ok(Some(id)) => {
                // The store is taken only to read the bookmark and is already released when notifying.
                match lock(&store)
                    .and_then(|s| bookmark_failure(&s, profile, id, marks_i18n::bookmark_error))
                {
                    // No `log::warn!`: the failure is already stored on the bookmark.
                    Ok(Some(notice)) => {
                        notify(app, notice);
                    }
                    Ok(None) => {}
                    Err(e) => log::warn!("worker: no se pudo leer el bookmark {id}: {e:#}"),
                }
                notify_changed(app);
                advanced = true;
            }
            Ok(None) => {}
            Err(e) => {
                lock_set(broken).insert(profile);
                fail(
                    app,
                    Some(profile),
                    AppText::StoreFailed,
                    &marks_i18n::error(&e),
                );
            }
        }
    } else {
        lock_set(broken).insert(profile);
        return false;
    }
    if let Some(store) = profile_store(app, state, profile) {
        let favicon =
            tauri::async_runtime::spawn_blocking(move || favicon_step(&store, &download_favicon));
        match favicon.await {
            Ok(Ok(Some(_))) => {
                notify_changed(app);
                advanced = true;
            }
            Ok(Ok(None)) => {}
            Ok(Err(e)) => {
                lock_set(broken).insert(profile);
                fail(
                    app,
                    Some(profile),
                    AppText::FaviconsFailed,
                    &marks_i18n::error(&e),
                );
            }
            Err(e) => {
                lock_set(broken).insert(profile);
                fail(app, Some(profile), AppText::FaviconsFailed, &e.to_string());
            }
        }
    }
    advanced
}

/// The set of broken profiles; poisoning does not matter, the set stays valid.
fn lock_set(set: &Mutex<HashSet<i64>>) -> std::sync::MutexGuard<'_, HashSet<i64>> {
    set.lock().unwrap_or_else(|e| e.into_inner())
}

/// One pass: one step per profile, in order. Returns whether any made progress.
async fn take_turns<F, Fut>(profiles: &[i64], mut step: F) -> bool
where
    F: FnMut(i64) -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let mut advanced = false;
    for &profile in profiles {
        // `|=`, not `||`: every profile gets its turn even if one already made progress.
        advanced |= step(profile).await;
    }
    advanced
}

/// The profile's store, or `None` if it no longer exists or could not be opened (notified).
fn profile_store(app: &AppHandle, state: &AppState, profile: i64) -> Option<Arc<Mutex<Store>>> {
    match state.library.existing_store(profile) {
        Ok(store) => store,
        Err(e) => {
            fail(
                app,
                Some(profile),
                AppText::ProfileOpenFailed,
                &marks_i18n::error(&e),
            );
            None
        }
    }
}

/// A worker failure the user must see, `text` and its cause: to the log and to `profile`'s notifications.
fn fail(app: &AppHandle, profile: Option<i64>, text: AppText, cause: &str) {
    let message = chain(&text.localize(), cause);
    warn(
        app,
        NewNotification {
            profile,
            severity: Severity::Error,
            message,
            detail: None,
            target: None,
        },
    );
}

/// Tells the frontend the library changed; a failure is only logged.
pub(crate) fn notify_changed(app: &AppHandle) {
    if let Err(e) = app.emit(LIBRARY_CHANGED, ()) {
        log::warn!("worker: no se pudo emitir {LIBRARY_CHANGED}: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn take_turns_interleaves_profiles() {
        // Profile 1 has 3 steps and profile 2 has 1.
        let mut left = [(1_i64, 3_u32), (2, 1)];
        let mut order = Vec::new();
        let mut last = true;
        while last {
            last = take_turns(&[1, 2], |id| {
                let entry = left.iter_mut().find(|(p, _)| *p == id);
                let advanced = match entry {
                    Some((_, n)) if *n > 0 => {
                        *n -= 1;
                        order.push(id);
                        true
                    }
                    _ => false,
                };
                async move { advanced }
            })
            .await;
        }
        assert_eq!(order, [1, 2, 1, 1]);
        assert!(!last);
    }
}
