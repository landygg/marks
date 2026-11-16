//! State shared by the commands and the worker.

use crate::extracting::Extracting;
use crate::notifications::warn;
use anyhow::{Result, anyhow};
use marks_core::ai::Model;
use marks_core::genai_model::GenaiModel;
use marks_core::msg::{ErrorPart, Msg, MsgExt, parts};
use marks_core::notifications::{NewNotification, Notifications, Severity, Target};
use marks_core::profiles::{Library, ProfileSeed};
use marks_core::settings::{AiRole, AiSettings, ExtractionSettings};
use marks_core::store::lock;
use marks_core::transfer;
use marks_core::{DB_ENV, default_db_path, settings};
use marks_i18n::{AppText, DataText, Localize, chain};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};
use tauri::{AppHandle, Runtime};
use tokio::sync::Notify;

/// Log only: developer text, not user text.
const LOG_NO_MODEL: &str = "AI not configured"; // i18n-allow: log

pub struct AppState {
    pub library: Arc<Library>,
    /// Summary and search.
    pub model: RwLock<Option<Arc<dyn Model>>>,
    pub extraction_model: RwLock<Option<ExtractionModel>>,
    pub wake: Arc<Notify>,
    /// False until the local API manages to open its port.
    pub api_listening: AtomicBool,
    /// A single lock for both hidden windows, `extractor` and `parser`: held only while one is alive,
    /// so there are never two at once, even when the worker and a command both use them
    /// (`webview_fetcher::in_hidden_window`).
    pub hidden_window: tokio::sync::Mutex<()>,
    /// Bookmarks being extracted right now. Only taken to add, remove or copy.
    pub extracting: Mutex<Vec<Extracting>>,
    /// The session's notification log. Never taken while holding a store or across an `.await`
    /// (`notifications::registry`).
    pub notifications: Mutex<Notifications>,
    /// Import preview waiting to be confirmed or cancelled; only one at a time.
    pub pending_import: Mutex<Option<PendingImport>>,
    /// The update found by the last check, waiting for "Install and restart" (`updater`).
    pub pending_update: crate::updater::PendingUpdate,
}

/// What was read from a bookmarks file and the profile it was read for.
pub struct PendingImport {
    pub profile: i64,
    pub preview: transfer::Preview,
}

/// Takes the preview if it belongs to the given profile; if not, leaves it and returns `Msg::ImportOtherProfile`.
pub fn take_pending_import(
    slot: &Mutex<Option<PendingImport>>,
    profile: i64,
) -> Result<transfer::Preview> {
    // Only the whole `Option` is swapped: a poisoned lock leaves no half-written data.
    let mut slot = slot.lock().unwrap_or_else(|e| e.into_inner());
    match slot.take() {
        Some(pending) if pending.profile == profile => Ok(pending.preview),
        Some(other) => {
            *slot = Some(other);
            Err(Msg::ImportOtherProfile.err())
        }
        None => Err(Msg::NoPendingImport.err()),
    }
}

/// Replaces the prepared import; `None` discards it.
pub fn set_pending_import(slot: &Mutex<Option<PendingImport>>, pending: Option<PendingImport>) {
    *slot.lock().unwrap_or_else(|e| e.into_inner()) = pending;
}

/// `rebuild_model`'s result, one per role.
pub struct Rebuilt {
    pub summary: Result<()>,
    pub extraction: Result<()>,
}

/// The extraction model and its name, stored in `extractor` as `ai:<name>`.
#[derive(Clone)]
pub struct ExtractionModel {
    pub model: Arc<dyn Model>,
    pub name: String,
}

impl AppState {
    /// Opens the library at `MARKS_DB` or the default path, still without a model (see `init_model`).
    pub fn open() -> Result<Self> {
        let path = std::env::var_os(DB_ENV).map_or_else(default_db_path, PathBuf::from);
        let seed = ProfileSeed {
            name: &DataText::DefaultProfileName.localize(),
            initials: &DataText::DefaultProfileInitials.localize(),
        };
        Ok(Self::new(Library::open(&path, &seed)?))
    }

    /// State without a model over `library`.
    pub fn new(library: Library) -> Self {
        Self {
            library: Arc::new(library),
            model: RwLock::new(None),
            extraction_model: RwLock::new(None),
            wake: Arc::new(Notify::new()),
            api_listening: AtomicBool::new(false),
            hidden_window: tokio::sync::Mutex::new(()),
            extracting: Mutex::new(Vec::new()),
            notifications: Mutex::new(Notifications::new(AppText::EmptyNotification.localize())),
            pending_import: Mutex::new(None),
            pending_update: Mutex::new(None),
        }
    }

    /// Builds the model if the settings allow it; otherwise logs it and goes on without one. Called
    /// once the logger is active so the notice is not lost. With AI enabled and no summary model it
    /// also notifies every profile; the extraction one only goes to the log.
    pub fn init_model<R: Runtime>(&self, app: &AppHandle<R>) {
        let rebuilt = self.rebuild_model();
        if let Err(e) = &rebuilt.summary {
            // Without readable settings we cannot tell whether it is enabled: log only.
            if self.load_ai().is_ok_and(|(ai, _)| ai.enabled) {
                warn(
                    app,
                    NewNotification {
                        profile: None,
                        severity: Severity::Warning,
                        message: chain(
                            &AppText::ModelUnavailable.localize(),
                            &marks_i18n::error(e),
                        ),
                        detail: None,
                        target: Some(Target::Ai),
                    },
                );
            } else {
                log::warn!("{LOG_NO_MODEL} ({:?}): {e:#}", AiRole::Summary);
            }
        }
        if let Err(e) = &rebuilt.extraction {
            log::warn!("{LOG_NO_MODEL} ({:?}): {e:#}", AiRole::Extraction);
        }
    }

    /// Rebuilds the summary and extraction models from the saved settings; whichever fails, or both
    /// with AI disabled, is left without a model. Each role returns its own result, so one's failure
    /// is not shown when saving the other's settings.
    pub fn rebuild_model(&self) -> Rebuilt {
        self.rebuild_model_with(&settings::api_key)
    }

    /// `rebuild_model` with `api_key` as the key reader: tests do not touch the keychain.
    fn rebuild_model_with(&self, api_key: ApiKey<'_>) -> Rebuilt {
        let (summary, extraction) = match self.load_ai() {
            Ok((ai, own)) => {
                let summary = build_summary(&ai, api_key);
                let shared = summary.as_ref().ok().cloned().flatten();
                (summary, build_extraction(&ai, &own, shared, api_key))
            }
            // Without readable settings there is neither.
            Err(e) => (Err(copy_error(&e)), Err(e)),
        };
        // They only store an `Arc`: a poisoned lock leaves no half-written data.
        *self.model.write().unwrap_or_else(|e| e.into_inner()) =
            summary.as_ref().ok().cloned().flatten();
        *self
            .extraction_model
            .write()
            .unwrap_or_else(|e| e.into_inner()) = extraction.as_ref().ok().cloned().flatten();
        Rebuilt {
            summary: summary.map(drop),
            extraction: extraction.map(drop),
        }
    }

    /// Copy of the current model, to use it without holding the lock.
    pub fn model(&self) -> Option<Arc<dyn Model>> {
        self.model.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Copy of the extraction model: the same `Arc` as `model()` with the summary settings.
    pub fn extraction_model(&self) -> Option<ExtractionModel> {
        self.extraction_model
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// AI settings are global: they live in the primary store.
    fn load_ai(&self) -> Result<(AiSettings, ExtractionSettings)> {
        let primary = self.library.primary();
        let primary = lock(&primary)?;
        Ok((
            settings::load(&primary)?,
            settings::load_extraction(&primary)?,
        ))
    }
}

/// Copy of `e` layer by layer (`Msg` or text), to give both roles the same failure: an
/// `anyhow::Error` cannot be cloned, and turning it into text would lose the translatable codes.
fn copy_error(e: &anyhow::Error) -> anyhow::Error {
    let copy = parts(e)
        .into_iter()
        .rev()
        .fold(None, |source, part| match (part, source) {
            (ErrorPart::Msg(m), None) => Some(m.err()),
            (ErrorPart::Msg(m), Some(s)) => Err::<(), _>(s).msg(m).err(),
            (ErrorPart::Raw { raw }, None) => Some(anyhow!(raw)),
            (ErrorPart::Raw { raw }, Some(s)) => Some(s.context(raw)),
        });
    // With no layers there is nothing to translate.
    copy.unwrap_or_else(|| anyhow!("{e:#}"))
}

/// Reads a role's key from the keychain (`settings::api_key`).
type ApiKey<'a> = &'a dyn Fn(AiRole) -> Result<Option<String>>;

/// `None` with AI disabled: an expected state, not a failure.
fn build_summary(ai: &AiSettings, api_key: ApiKey<'_>) -> Result<Option<Arc<dyn Model>>> {
    if !ai.enabled {
        return Ok(None);
    }
    let key = api_key(AiRole::Summary)?;
    let model = GenaiModel::new(&ai.connection, key, &ai.chat_model, &ai.embed_model)?;
    Ok(Some(Arc::new(model)))
}

/// With the summary settings, the same model (`shared`); otherwise its own.
fn build_extraction(
    ai: &AiSettings,
    own: &ExtractionSettings,
    shared: Option<Arc<dyn Model>>,
    api_key: ApiKey<'_>,
) -> Result<Option<ExtractionModel>> {
    if !ai.enabled {
        return Ok(None);
    }
    let (connection, name) = own.resolve(ai);
    let model = if own.same_as_summary {
        let Some(shared) = shared else {
            return Ok(None);
        };
        shared
    } else {
        let key = api_key(AiRole::Extraction)?;
        // No embedding model: extraction only uses the chat one.
        Arc::new(GenaiModel::new(connection, key, name, "")?)
    };
    Ok(Some(ExtractionModel {
        model,
        name: name.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use marks_core::ai::FakeModel;
    use marks_core::profiles::ProfileSeed;
    use marks_core::settings::{AiConnection, AiSettings, Provider};
    use marks_i18n::{Locale, localize_in};

    const TEST_SEED: ProfileSeed = ProfileSeed {
        name: "Personal",
        initials: "PE",
    };

    /// With no saved settings (new install) or AI disabled there is no model, even if there was one
    /// before, and it is not an error. Does not test AI enabled: it would read the real keychain.
    #[test]
    fn ai_off_builds_no_model() {
        let dir = std::env::temp_dir().join(format!("marks-state-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let state =
            AppState::new(Library::open(&dir.join("marks.db"), &TEST_SEED).expect("library"));
        *state.model.write().expect("lock") = Some(Arc::new(FakeModel::default()));
        state.rebuild_model().summary.expect("sin ajustes");
        assert!(state.model().is_none());
        assert!(state.extraction_model().is_none());

        *state.model.write().expect("lock") = Some(Arc::new(FakeModel::default()));
        let off = AiSettings {
            enabled: false,
            connection: AiConnection {
                provider: Provider::Omlx,
                endpoint: Some("http://h:1".into()),
            },
            ..AiSettings::default()
        };
        settings::save(&lock(&state.library.primary()).expect("lock"), &off).expect("guardar");
        state.rebuild_model().summary.expect("desactivada");
        assert!(state.model().is_none());
        assert!(state.extraction_model().is_none());
        std::fs::remove_dir_all(dir).expect("borrar temp dir");
    }

    /// With the summary settings, extraction uses the same `Arc` and its chat model; with AI disabled
    /// there is none. Does not test separate settings: it would read the real keychain.
    #[test]
    fn extraction_shares_the_summary_model() {
        let shared: Arc<dyn Model> = Arc::new(FakeModel::default());
        let on = AiSettings {
            enabled: true,
            chat_model: "qwen".into(),
            ..AiSettings::default()
        };
        let same = ExtractionSettings::default();
        let built = build_extraction(&on, &same, Some(shared.clone()), &no_key)
            .expect("sin error")
            .expect("modelo");
        assert!(Arc::ptr_eq(&built.model, &shared));
        assert_eq!(built.name, "qwen");
        assert!(
            build_extraction(&on, &same, None, &no_key)
                .expect("sin error")
                .is_none()
        );
        let off = AiSettings {
            enabled: false,
            ..on
        };
        assert!(
            build_extraction(&off, &same, Some(shared), &no_key)
                .expect("sin error")
                .is_none()
        );
    }

    fn no_key(_: AiRole) -> Result<Option<String>> {
        Ok(None)
    }

    /// An extraction model that cannot be built (OpenAI without a key) does not take away the summary
    /// one or fail its result: each role returns its own.
    #[test]
    fn a_broken_extraction_model_does_not_fail_the_summary() {
        let dir = std::env::temp_dir().join(format!("marks-state-roles-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let state =
            AppState::new(Library::open(&dir.join("marks.db"), &TEST_SEED).expect("library"));
        {
            let primary = state.library.primary();
            let primary = lock(&primary).expect("lock");
            let on = AiSettings {
                enabled: true,
                ..AiSettings::default()
            };
            settings::save(&primary, &on).expect("guardar resumen");
            let own = ExtractionSettings {
                same_as_summary: false,
                connection: AiConnection {
                    provider: Provider::Openai,
                    endpoint: None,
                },
                model: "gpt".into(),
            };
            settings::save_extraction(&primary, &own).expect("guardar extracción");
        }
        let rebuilt = state.rebuild_model_with(&no_key);
        rebuilt.summary.expect("el resumen se construye");
        assert!(rebuilt.extraction.is_err());
        assert!(state.model().is_some());
        assert!(state.extraction_model().is_none());
        std::fs::remove_dir_all(dir).expect("borrar temp dir");
    }

    struct ErrorText<'a>(&'a anyhow::Error);
    impl Localize for ErrorText<'_> {
        fn localize(&self) -> String {
            marks_i18n::error(self.0)
        }
    }

    /// With unreadable settings both roles return the same error, with its codes, so it is translated.
    #[test]
    fn unreadable_settings_keep_their_codes_for_both_roles() {
        let dir = std::env::temp_dir().join(format!("marks-state-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let state =
            AppState::new(Library::open(&dir.join("marks.db"), &TEST_SEED).expect("library"));
        // "ai" is the summary settings key (`settings::SETTINGS_KEY`).
        lock(&state.library.primary())
            .expect("lock")
            .set_setting("ai", "{")
            .expect("guardar");
        let rebuilt = state.rebuild_model_with(&no_key);
        let summary = rebuilt.summary.expect_err("resumen");
        let extraction = rebuilt.extraction.expect_err("extracción");
        assert_eq!(parts(&summary), parts(&extraction));
        let invalid = localize_in(Locale::Es, &Msg::InvalidSettings);
        for e in [&summary, &extraction] {
            let text = localize_in(Locale::Es, &ErrorText(e));
            assert!(text.starts_with(&invalid), "{text}");
        }
        std::fs::remove_dir_all(dir).expect("borrar temp dir");
    }

    /// A preview prepared for one profile is not applied to another, and is not lost by trying.
    #[test]
    fn pending_import_is_for_one_profile() {
        let store = marks_core::store::Store::open_in_memory().expect("store");
        let preview = transfer::preview(&store, Vec::new(), "lista de lectura").expect("preview");
        let slot = Mutex::new(Some(PendingImport {
            profile: 2,
            preview,
        }));
        let other = take_pending_import(&slot, 1).err().expect("otro perfil");
        assert_eq!(other.to_string(), Msg::ImportOtherProfile.to_string());
        assert!(slot.lock().expect("lock").is_some());
        let preview = take_pending_import(&slot, 2).expect("su perfil");
        let none = take_pending_import(&slot, 2).err().expect("ya no hay");
        assert_eq!(none.to_string(), Msg::NoPendingImport.to_string());
        // Put back after a failed apply, it can be confirmed again.
        set_pending_import(
            &slot,
            Some(PendingImport {
                profile: 2,
                preview,
            }),
        );
        take_pending_import(&slot, 2).expect("devuelta");
    }
}
