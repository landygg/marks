//! Updates from GitHub Releases, only from Rust (like `tauri-plugin-dialog`): the capability grants
//! no `updater:` permission. The frontend asks through the commands in `commands.rs`.

use crate::notifications::notify;
use crate::state::AppState;
use anyhow::Result;
use marks_core::notifications::{NewNotification, Severity, Target};
use marks_core::settings;
use marks_core::store::lock;
use marks_i18n::{AppText, Localize};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;
use tauri::{AppHandle, Manager, Url};
use tauri_plugin_updater::{Update, Updater, UpdaterExt};

const STARTUP_DELAY: Duration = Duration::from_secs(30);
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
/// Replaces the endpoint of `tauri.conf.json`, only for testing; the signature is still required.
const ENDPOINT_ENV: &str = "MARKS_UPDATE_ENDPOINT";
/// Log only: developer text, not user text.
const LOG_CHECK_FAILED: &str = "updater: automatic check failed"; // i18n-allow: log
/// Log only (`install_update` shows `UpdateInstallFailed`).
const LOG_NOTHING_PENDING: &str = "no update pending: check first"; // i18n-allow: log

/// aarch64 macOS release builds only: Intel ships as a manual, untested download, and dev builds
/// never update.
pub fn auto_check_supported() -> bool {
    cfg!(all(
        target_os = "macos",
        target_arch = "aarch64",
        not(debug_assertions)
    ))
}

/// The update found by the last check, kept for `install`.
fn pending(state: &AppState) -> MutexGuard<'_, Option<Update>> {
    lock_slot(&state.pending_update)
}

fn lock_slot(slot: &PendingUpdate) -> MutexGuard<'_, Option<Update>> {
    slot.lock().unwrap_or_else(|e| e.into_inner())
}

/// The update to install; an error if a later check found none, so the app does not restart as
/// if it had updated.
fn pending_update(slot: &PendingUpdate) -> Result<Update> {
    lock_slot(slot)
        .clone()
        .ok_or_else(|| anyhow::anyhow!(LOG_NOTHING_PENDING))
}

pub fn pending_version(state: &AppState) -> Option<String> {
    pending(state).as_ref().map(|u| u.version.clone())
}

fn updater(app: &AppHandle) -> Result<Updater> {
    Ok(match std::env::var(ENDPOINT_ENV) {
        Ok(url) => app
            .updater_builder()
            .endpoints(vec![Url::parse(&url)?])?
            .build()?,
        Err(_) => app.updater()?,
    })
}

/// Asks the endpoint for a newer version; keeps its `Update` for `install` and returns its version.
pub async fn check(app: &AppHandle) -> Result<Option<String>> {
    let update = updater(app)?.check().await?;
    let version = update.as_ref().map(|u| u.version.clone());
    *pending(&app.state::<AppState>()) = update;
    Ok(version)
}

/// Downloads and installs the update found by the last check; `install_update` restarts after it.
pub async fn install(app: &AppHandle) -> Result<()> {
    let update = pending_update(&app.state::<AppState>().pending_update)?;
    Ok(update.download_and_install(|_, _| {}, || {}).await?)
}

/// Whether the user leaves the automatic check on; a store failure counts as the default.
fn auto_enabled(state: &AppState) -> bool {
    lock(&state.library.primary())
        .and_then(|store| settings::load_update_settings(&store))
        .unwrap_or_default()
        .auto
}

/// The periodic check: 30 s after launch, then once a day. A new version becomes an `Info`
/// notification that opens Settings → Updates; a failure only goes to the log.
pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(STARTUP_DELAY).await;
        loop {
            if auto_enabled(&app.state::<AppState>()) {
                match check(&app).await {
                    Ok(Some(version)) => {
                        notify(
                            &app,
                            NewNotification {
                                profile: None,
                                severity: Severity::Info,
                                message: AppText::UpdateAvailable { version }.localize(),
                                detail: None,
                                target: Some(Target::Updates),
                            },
                        );
                    }
                    Ok(None) => {}
                    Err(e) => log::warn!("{LOG_CHECK_FAILED}: {e:#}"),
                }
            }
            tokio::time::sleep(CHECK_INTERVAL).await;
        }
    });
}

/// The slot `AppState` keeps.
pub type PendingUpdate = Mutex<Option<Update>>;

#[cfg(test)]
mod tests {
    #[test]
    fn auto_check_supported() {
        assert_eq!(
            super::auto_check_supported(),
            cfg!(all(
                target_os = "macos",
                target_arch = "aarch64",
                not(debug_assertions)
            ))
        );
        assert!(!super::auto_check_supported(), "cargo test builds in debug");
    }

    #[test]
    fn install_without_a_pending_update_fails() {
        let slot: super::PendingUpdate = std::sync::Mutex::new(None);
        assert!(super::pending_update(&slot).is_err());
    }
}
