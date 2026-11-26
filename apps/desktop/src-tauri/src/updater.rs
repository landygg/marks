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
/// Log only: the manifest's minimum is not a version; the update is held back.
const LOG_BAD_MINIMUM: &str = "updater: unreadable minimum macOS"; // i18n-allow: log
/// Manifest field with the oldest macOS the update runs on (`scripts/updater-manifest.mjs`).
const MIN_SYSTEM_FIELD: &str = "minimum_system_version";
/// Most parts a macOS version has (major, minor, patch).
const VERSION_PARTS: usize = 3;

/// A macOS version: major, minor and patch.
type MacosVersion = (u32, u32, u32);

/// What a check found.
pub enum Found {
    UpToDate,
    Available(String),
    /// Newer, but not for this Mac: it is not kept for `install`.
    NeedsMacos {
        version: String,
        minimum: String,
        current: String,
    },
}

/// What a manual check tells Settings → Updates; a version this Mac cannot run is a result, not a
/// failure, so it arrives as text to show and not as an error.
#[derive(serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CheckView {
    UpToDate,
    Available { version: String },
    NeedsMacos { message: String },
}

impl From<Found> for CheckView {
    fn from(found: Found) -> Self {
        match found {
            Found::UpToDate => CheckView::UpToDate,
            Found::Available(version) => CheckView::Available { version },
            Found::NeedsMacos {
                version,
                minimum,
                current,
            } => CheckView::NeedsMacos {
                message: needs_macos_text(version, minimum, current),
            },
        }
    }
}

fn parse_version(text: &str) -> Option<MacosVersion> {
    let parts = text
        .split('.')
        .map(|p| p.parse::<u32>().ok())
        .collect::<Option<Vec<_>>>()?;
    if parts.is_empty() || parts.len() > VERSION_PARTS {
        return None;
    }
    let part = |i: usize| parts.get(i).copied().unwrap_or(0);
    Some((part(0), part(1), part(2)))
}

/// Whether `current` is older than `minimum`; an unreadable minimum counts as newer.
fn needs_newer_macos(minimum: &str, current: MacosVersion) -> bool {
    parse_version(minimum).is_none_or(|minimum| current < minimum)
}

/// The manifest's minimum macOS: `None` without the field; an error, with the raw value, when it
/// is not a version.
fn minimum_of(raw: &serde_json::Value) -> Option<Result<String, String>> {
    let value = raw.get(MIN_SYSTEM_FIELD)?;
    Some(match value.as_str() {
        Some(text) if parse_version(text).is_some() => Ok(text.to_string()),
        _ => Err(value
            .as_str()
            .map_or_else(|| value.to_string(), str::to_string)),
    })
}

/// The minimum that keeps the update off this Mac, if any. Fails towards not installing: an
/// unreadable field blocks, because installing blindly is what leaves an app that no longer opens.
fn blocked_by(raw: &serde_json::Value, current: Option<MacosVersion>) -> Option<String> {
    let current = current?;
    match minimum_of(raw)? {
        Ok(minimum) => needs_newer_macos(&minimum, current).then_some(minimum),
        Err(minimum) => {
            log::warn!("{LOG_BAD_MINIMUM}: {minimum}");
            Some(minimum)
        }
    }
}

/// `15.0` or `14.7.2`: the patch only when it is not zero.
fn version_text((major, minor, patch): MacosVersion) -> String {
    if patch == 0 {
        format!("{major}.{minor}")
    } else {
        format!("{major}.{minor}.{patch}")
    }
}

/// The running macOS; elsewhere there is no minimum to check.
#[cfg(target_os = "macos")]
fn current_macos() -> Option<MacosVersion> {
    let v = objc2_foundation::NSProcessInfo::processInfo().operatingSystemVersion();
    let part = |n: isize| u32::try_from(n).unwrap_or(0);
    Some((
        part(v.majorVersion),
        part(v.minorVersion),
        part(v.patchVersion),
    ))
}

#[cfg(not(target_os = "macos"))]
fn current_macos() -> Option<MacosVersion> {
    None
}

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

/// Asks the endpoint for a newer version and keeps its `Update` for `install`, unless it needs a
/// newer macOS: then nothing stays pending.
pub async fn check(app: &AppHandle) -> Result<Found> {
    let update = updater(app)?.check().await?;
    let current = current_macos();
    let found = match &update {
        None => Found::UpToDate,
        Some(u) => match blocked_by(&u.raw_json, current) {
            None => Found::Available(u.version.clone()),
            Some(minimum) => Found::NeedsMacos {
                version: u.version.clone(),
                minimum,
                current: current.map(version_text).unwrap_or_default(),
            },
        },
    };
    *pending(&app.state::<AppState>()) = match found {
        Found::Available(_) => update,
        _ => None,
    };
    Ok(found)
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

/// The text for a version that needs a newer macOS.
fn needs_macos_text(version: String, minimum: String, current: String) -> String {
    AppText::UpdateNeedsNewerMacos {
        version,
        minimum,
        current,
    }
    .localize()
}

fn notify_info(app: &AppHandle, message: String) {
    notify(
        app,
        NewNotification {
            profile: None,
            severity: Severity::Info,
            message,
            detail: None,
            target: Some(Target::Updates),
        },
    );
}

/// The periodic check: 30 s after launch, then once a day. A new version becomes an `Info`
/// notification that opens Settings → Updates; one that needs a newer macOS, too, once per version
/// in the session; a failure only goes to the log.
pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(STARTUP_DELAY).await;
        let mut told_needs_macos: Option<String> = None;
        loop {
            if auto_enabled(&app.state::<AppState>()) {
                match check(&app).await {
                    Ok(Found::Available(version)) => {
                        notify_info(&app, AppText::UpdateAvailable { version }.localize());
                    }
                    Ok(Found::NeedsMacos {
                        version,
                        minimum,
                        current,
                    }) => {
                        if told_needs_macos.as_ref() != Some(&version) {
                            told_needs_macos = Some(version.clone());
                            notify_info(&app, needs_macos_text(version, minimum, current));
                        }
                    }
                    Ok(Found::UpToDate) => {}
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
    fn needs_newer_macos_compares_the_running_version() {
        use super::needs_newer_macos;
        assert!(needs_newer_macos("15.0", (14, 7, 2)));
        assert!(!needs_newer_macos("15.0", (15, 0, 0)));
        assert!(needs_newer_macos("16.0", (15, 6, 1)));
        assert!(!needs_newer_macos("15", (15, 1, 0)));
        assert!(!needs_newer_macos("15.0.1", (15, 1, 0)));
        assert!(needs_newer_macos("15.0.1", (15, 0, 0)));
    }

    #[test]
    fn minimum_of_reads_the_manifest_field() {
        use super::minimum_of;
        use serde_json::json;
        assert!(minimum_of(&json!({ "version": "1.2.0" })).is_none());
        assert_eq!(
            minimum_of(&json!({ "minimum_system_version": "15.0" })).map(|m| m.ok()),
            Some(Some("15.0".to_string()))
        );
        for bad in [
            json!("quince"),
            json!(15),
            json!(""),
            json!("15..1"),
            json!("15.0.0.1"),
        ] {
            let found = minimum_of(&json!({ "minimum_system_version": bad }));
            assert!(matches!(found, Some(Err(_))), "{bad}");
        }
    }

    #[test]
    fn blocked_by_fails_towards_not_installing() {
        use super::blocked_by;
        use serde_json::json;
        let raw = |v: serde_json::Value| json!({ "minimum_system_version": v });
        // No field (manifests before 1.2.0), or no macOS version to compare: no gate.
        assert_eq!(blocked_by(&json!({}), Some((14, 0, 0))), None);
        assert_eq!(blocked_by(&raw(json!("99.0")), None), None);
        assert_eq!(blocked_by(&raw(json!("15.0")), Some((15, 0, 0))), None);
        assert_eq!(
            blocked_by(&raw(json!("99.0")), Some((15, 6, 1))),
            Some("99.0".to_string())
        );
        // Unreadable: blocks, showing what the manifest says.
        assert_eq!(
            blocked_by(&raw(json!("quince")), Some((15, 6, 1))),
            Some("quince".to_string())
        );
        assert_eq!(
            blocked_by(&raw(json!(15)), Some((15, 6, 1))),
            Some("15".to_string())
        );
    }

    #[test]
    fn macos_versions_print_without_a_zero_patch() {
        assert_eq!(super::version_text((15, 0, 0)), "15.0");
        assert_eq!(super::version_text((14, 7, 2)), "14.7.2");
    }

    #[test]
    fn a_check_reaches_the_screen_as_a_tagged_result() {
        use super::{CheckView, Found};
        use serde_json::json;
        let view = |found| serde_json::to_value(CheckView::from(found)).expect("JSON");
        assert_eq!(view(Found::UpToDate), json!({ "kind": "up_to_date" }));
        assert_eq!(
            view(Found::Available("1.3.0".into())),
            json!({ "kind": "available", "version": "1.3.0" })
        );
        let needs = view(Found::NeedsMacos {
            version: "1.3.0".into(),
            minimum: "99.0".into(),
            current: "15.4".into(),
        });
        assert_eq!(needs["kind"], "needs_macos");
        let message = needs["message"].as_str().expect("message");
        assert!(
            message.contains("99.0") && message.contains("15.4"),
            "{message}"
        );
    }

    #[test]
    fn install_without_a_pending_update_fails() {
        let slot: super::PendingUpdate = std::sync::Mutex::new(None);
        assert!(super::pending_update(&slot).is_err());
    }
}
