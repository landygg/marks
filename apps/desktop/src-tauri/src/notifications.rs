//! App notifications: stores them in `AppState`'s log and notifies the window.

use crate::state::AppState;
use marks_core::notifications::{NewNotification, Notification, Notifications};
use std::sync::MutexGuard;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// Payload-less event: the list changed. Same value as `NOTIFICATIONS_CHANGED` in `src/lib/api.ts`.
pub const NOTIFICATIONS_CHANGED: &str = "notifications-changed";

/// Milliseconds since the UNIX epoch; 0 if the clock is before 1970.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// The log, with no store held: the list has no half-written data, so poisoning does not matter.
pub fn registry(state: &AppState) -> MutexGuard<'_, Notifications> {
    state
        .notifications
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

pub fn emit_changed<R: Runtime>(app: &AppHandle<R>) {
    if let Err(e) = app.emit(NOTIFICATIONS_CHANGED, ()) {
        log::warn!("notificaciones: no se pudo emitir {NOTIFICATIONS_CHANGED}: {e}");
    }
}

/// Stores the notification with the current time and notifies the window.
pub fn notify<R: Runtime>(app: &AppHandle<R>, new: NewNotification) -> Notification {
    let state = app.state::<AppState>();
    let saved = registry(&state).push(new, now_ms());
    emit_changed(app);
    saved
}

/// For background notices that used to go only to the log: logs and notifies them. A notice that
/// repeats unchanged (a store failing on every worker pass) does not ring the bell again.
pub fn warn<R: Runtime>(app: &AppHandle<R>, new: NewNotification) {
    log::warn!("{}", new.message);
    let state = app.state::<AppState>();
    registry(&state).push_quiet(new, now_ms());
    emit_changed(app);
}
