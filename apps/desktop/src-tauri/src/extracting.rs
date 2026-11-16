//! Which bookmark is being extracted right now, so the UI can show it.

use crate::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// Payload-less event: the list changed. Same value as `EXTRACTING_CHANGED` in `src/lib/api.ts`.
pub const EXTRACTING_CHANGED: &str = "extracting-changed";

/// A bookmark being extracted; `id`s repeat across profiles, so they go together.
#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Extracting {
    pub profile: i64,
    pub id: i64,
}

/// Removes its entry and notifies on drop, also if the extraction failed.
pub struct ExtractingGuard<R: Runtime> {
    app: AppHandle<R>,
    entry: Extracting,
}

/// Copy of the list; the lock is not held.
pub fn snapshot(state: &AppState) -> Vec<Extracting> {
    state
        .extracting
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

fn emit<R: Runtime>(app: &AppHandle<R>) {
    if let Err(e) = app.emit(EXTRACTING_CHANGED, ()) {
        log::warn!("extracting: no se pudo emitir {EXTRACTING_CHANGED}: {e}");
    }
}

/// Marks the bookmark as being extracted and notifies; the guard removes it on drop.
pub fn mark<R: Runtime>(app: &AppHandle<R>, profile: i64, id: i64) -> ExtractingGuard<R> {
    let entry = Extracting { profile, id };
    let state = app.state::<AppState>();
    state
        .extracting
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(entry.clone());
    emit(app);
    ExtractingGuard {
        app: app.clone(),
        entry,
    }
}

impl<R: Runtime> Drop for ExtractingGuard<R> {
    fn drop(&mut self) {
        let state = self.app.state::<AppState>();
        {
            let mut list = state.extracting.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(i) = list.iter().position(|e| *e == self.entry) {
                list.remove(i);
            }
        }
        emit(&self.app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use marks_core::profiles::Library;
    use marks_core::profiles::ProfileSeed;

    const TEST_SEED: ProfileSeed = ProfileSeed {
        name: "Personal",
        initials: "PE",
    };

    fn temp_dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("marks-extracting-{}", std::process::id()))
    }

    fn app() -> tauri::App<tauri::test::MockRuntime> {
        let app = tauri::test::mock_app();
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).expect("temp dir");
        let library = Library::open(&dir.join("marks.db"), &TEST_SEED).expect("library");
        app.manage(AppState::new(library));
        app
    }

    /// A failing job drops the guard when leaving through `?`.
    fn failing_job<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
        let _guard = mark(app, 2, 7);
        Err("fallo".into())
    }

    #[test]
    fn guard_removes_its_entry_on_drop() {
        let app = app();
        let handle = app.handle();
        let state = handle.state::<AppState>();
        let guard = mark(handle, 1, 5);
        assert_eq!(snapshot(&state), vec![Extracting { profile: 1, id: 5 }]);
        drop(guard);
        assert!(snapshot(&state).is_empty());

        assert!(failing_job(handle).is_err());
        assert!(snapshot(&state).is_empty());
        std::fs::remove_dir_all(temp_dir()).expect("borrar temp dir");
    }
}
