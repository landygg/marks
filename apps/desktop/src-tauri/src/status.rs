//! What the status bar shows about the whole app: the AI model and the local API.

use marks_core::ai_health::AiStatus;
use marks_i18n::parts_text;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

/// Payload-less event: the AI or API status changed. Same value as `APP_STATUS_CHANGED` in
/// `src/lib/api.ts`.
pub const APP_STATUS_CHANGED: &str = "app-status-changed";

pub fn emit_changed<R: Runtime>(app: &AppHandle<R>) {
    if let Err(e) = app.emit(APP_STATUS_CHANGED, ()) {
        log::warn!("estado: no se pudo emitir {APP_STATUS_CHANGED}: {e}");
    }
}

/// The local API: `Starting` until it tries its port, `Busy` if it could not open it, `Stopped` if
/// the server ended with an error.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiStatus {
    Starting,
    Listening,
    Busy,
    Stopped,
}

/// `AiStatus` with its error translated. Only the model's name: no endpoint and no key.
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum AiView {
    Off,
    Ready {
        model: String,
    },
    Failed {
        model: Option<String>,
        error: String,
    },
}

impl From<AiStatus> for AiView {
    fn from(status: AiStatus) -> Self {
        match status {
            AiStatus::Off => Self::Off,
            AiStatus::Ready { model } => Self::Ready { model },
            AiStatus::Failed { model, error } => Self::Failed {
                model,
                error: parts_text(&error),
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AppStatusView {
    pub ai: AiView,
    pub api: ApiStatus,
    pub api_port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use marks_core::msg::ErrorPart;
    use marks_core::profiles::{Library, ProfileSeed};
    use marks_core::settings::{self, AiConnection, AiSettings, Provider};
    use marks_core::store::lock;
    use serde_json::json;

    #[test]
    fn ai_view_serializes_for_the_frontend() {
        let json = |s: AiStatus| serde_json::to_value(AiView::from(s)).expect("json");
        assert_eq!(json(AiStatus::Off), json!({ "kind": "off" }));
        assert_eq!(
            json(AiStatus::Ready { model: "m".into() }),
            json!({ "kind": "ready", "model": "m" })
        );
        let failed = AiStatus::Failed {
            model: None,
            error: vec![ErrorPart::Raw { raw: "boom".into() }],
        };
        assert_eq!(
            json(failed),
            json!({ "kind": "failed", "model": null, "error": "boom" })
        );
    }

    #[test]
    fn api_status_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(ApiStatus::Busy).expect("json"),
            json!("busy")
        );
    }

    #[test]
    fn app_status_has_no_endpoint_or_key() {
        let dir = std::env::temp_dir().join(format!("marks-status-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let seed = ProfileSeed {
            name: "Personal",
            initials: "PE",
        };
        let state = AppState::new(Library::open(&dir.join("marks.db"), &seed).expect("library"));
        let on = AiSettings {
            enabled: true,
            chat_model: "qwen".into(),
            connection: AiConnection {
                provider: Provider::Ollama,
                endpoint: Some("http://secret-host:1".into()),
            },
            ..AiSettings::default()
        };
        settings::save(&lock(&state.library.primary()).expect("lock"), &on).expect("guardar");
        state.rebuild_model_with(&|_| Ok(Some("secret-key".into())));
        let view = state.app_status();
        assert_eq!(
            view.ai,
            AiView::Ready {
                model: "qwen".into()
            }
        );
        let text = serde_json::to_string(&view).expect("json");
        assert!(!text.contains("secret-host"), "{text}");
        assert!(!text.contains("secret-key"), "{text}");
        std::fs::remove_dir_all(dir).expect("borrar temp dir");
    }
}
