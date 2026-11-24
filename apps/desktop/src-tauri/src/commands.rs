//! IPC commands: they only take the state and call core.
//!
//! Threads: a synchronous `#[tauri::command]` runs on the main thread and freezes the window
//! while it lasts; only one-off writes go there, so they apply in arrival order (the UI fires some
//! without waiting: notes, tags, title). Reads and long work (keychain, profile files, bulk
//! writes) use `#[tauri::command(async)]` and run on the pool.

use crate::api::API_PORT;
use crate::extracting::{self, Extracting};
use crate::locale;
use crate::notifications;
use crate::parser_window::load_clean_html;
use crate::state::{AppState, PendingImport, set_pending_import, take_pending_import};
use crate::status::{ApiStatus, AppStatusView};
use crate::tray::MAIN_WINDOW;
use crate::updater;
use crate::worker::{app_fetcher, notify_changed};
use anyhow::Result;
use marks_core::ai::{Model, SummarizeInput};
use marks_core::groups::TagGroup;
use marks_core::msg::{Msg, MsgExt};
use marks_core::netscape;
use marks_core::notifications::{NewNotification, Notification};
use marks_core::pipeline;
use marks_core::profiles::Profile;
use marks_core::search::{self, SearchHit};
use marks_core::settings::{self, AiRole, AiSettings, ExtractionSettings, UpdateSettings};
use marks_core::store::{Bookmark, BookmarkPatch, NewBookmark, Stats, Status, Store, lock};
use marks_core::transfer::{self, ImportSummary};
use marks_i18n::{AppText, DataText, Localize, bookmark_error};
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, FilePath};

const SEARCH_LIMIT: usize = 50;
/// Log only: developer text, not user text.
const LOG_UPDATE_FAILED: &str = "updater";
/// Small fixed HTML used to test the extraction model.
const PROBE_HTML: &str = "<h1>hola</h1><p>hola</p>";
const PROBE_URL: &str = "https://example.com";
const PROBE_TEXT: &str = "hola";
const HTML_EXTENSIONS: &[&str] = &["html", "htm"];
const PROBE_INPUT: SummarizeInput<'static> = SummarizeInput {
    url: PROBE_URL,
    title: PROBE_TEXT,
    markdown: PROBE_TEXT,
    existing_categories: &[],
    existing_tags: &[],
};

#[derive(Serialize)]
pub struct SettingsView {
    settings: AiSettings,
    has_api_key: bool,
    extraction: ExtractionSettings,
    has_extraction_key: bool,
}

/// Extraction only reports chat time: no `embed_ms` or `dim`.
#[derive(Serialize)]
pub struct ConnectionReport {
    chat_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    embed_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dim: Option<usize>,
}

#[derive(Serialize)]
pub struct ProfilesView {
    profiles: Vec<Profile>,
    active: i64,
}

#[derive(Serialize)]
pub struct ApiInfo {
    port: u16,
    token: String,
    listening: bool,
}

/// A bookmark for the UI: core's with its error translated (core does not serialize it).
#[derive(Serialize)]
pub struct LocalBookmark {
    #[serde(flatten)]
    pub b: Bookmark,
    pub error: Option<String>,
}

impl From<Bookmark> for LocalBookmark {
    fn from(b: Bookmark) -> Self {
        let error = b.error.as_ref().map(bookmark_error);
        Self { b, error }
    }
}

/// `SearchHit` with its bookmark translated.
#[derive(Serialize)]
pub struct LocalHit {
    pub bookmark: LocalBookmark,
    pub score: f64,
    pub snippet: Option<String>,
}

impl From<SearchHit> for LocalHit {
    fn from(h: SearchHit) -> Self {
        Self {
            bookmark: h.bookmark.into(),
            score: h.score,
            snippet: h.snippet,
        }
    }
}

fn local<T, L: From<T>>(items: Vec<T>) -> Vec<L> {
    items.into_iter().map(L::from).collect()
}

/// Message with the whole context chain, translated, for the UI.
fn to_ipc(e: anyhow::Error) -> String {
    marks_i18n::error(&e)
}

/// Runs `f` with the active profile's store locked; the lock is released on return.
fn with_store<T>(state: &AppState, f: impl FnOnce(&Store) -> Result<T>) -> Result<T, String> {
    let store = state.library.active_store().map_err(to_ipc)?;
    lock(&store).and_then(|store| f(&store)).map_err(to_ipc)
}

/// Like `with_store`, with the store of `profile` (commands that write or name a bookmark or a
/// group: their `id`s repeat across profiles).
fn with_profile<T>(
    state: &AppState,
    profile: i64,
    f: impl FnOnce(&Store) -> Result<T>,
) -> Result<T, String> {
    let store = state.library.store(profile).map_err(to_ipc)?;
    lock(&store).and_then(|store| f(&store)).map_err(to_ipc)
}

/// Like `with_store`, with the primary store (global AI settings).
fn with_primary<T>(state: &AppState, f: impl FnOnce(&Store) -> Result<T>) -> Result<T, String> {
    lock(&state.library.primary())
        .and_then(|store| f(&store))
        .map_err(to_ipc)
}

/// Like `with_store`, and wakes the worker.
fn waking<T>(state: &AppState, f: impl FnOnce(&Store) -> Result<T>) -> Result<T, String> {
    let out = with_store(state, f)?;
    state.wake.notify_one();
    Ok(out)
}

/// Like `with_profile`, and wakes the worker.
fn waking_profile<T>(
    state: &AppState,
    profile: i64,
    f: impl FnOnce(&Store) -> Result<T>,
) -> Result<T, String> {
    let out = with_profile(state, profile, f)?;
    state.wake.notify_one();
    Ok(out)
}

#[tauri::command(async)]
pub fn list(state: State<'_, AppState>) -> Result<Vec<LocalBookmark>, String> {
    with_store(&state, Store::list_without_content).map(local)
}

#[tauri::command]
pub async fn search(query: String, state: State<'_, AppState>) -> Result<Vec<LocalHit>, String> {
    let model = state.model();
    let store = state.library.active_store().map_err(to_ipc)?;
    search::hybrid(&store, model.as_deref(), &query, SEARCH_LIMIT)
        .await
        .map(local)
        .map_err(to_ipc)
}

#[tauri::command(async)]
pub fn get(
    profile: i64,
    id: i64,
    state: State<'_, AppState>,
) -> Result<Option<LocalBookmark>, String> {
    with_profile(&state, profile, |s| s.get(id)).map(|b| b.map(LocalBookmark::from))
}

/// The stored favicon, as a `data:` URL (the CSP does not allow remote images).
#[tauri::command(async)]
pub fn bookmark_favicon(
    profile: i64,
    id: i64,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    with_profile(&state, profile, |s| s.favicon_data_url(id))
}

#[tauri::command]
pub fn add(
    url: String,
    notes: String,
    tags: Vec<String>,
    state: State<'_, AppState>,
) -> Result<LocalBookmark, String> {
    let new = NewBookmark {
        url,
        title: None,
        notes,
        tags,
        content: None,
        ..Default::default()
    };
    waking(&state, |s| s.add(new)).map(LocalBookmark::from)
}

#[tauri::command]
pub fn update(
    profile: i64,
    id: i64,
    patch: BookmarkPatch,
    state: State<'_, AppState>,
) -> Result<LocalBookmark, String> {
    with_profile(&state, profile, |s| s.update(id, patch)).map(LocalBookmark::from)
}

#[tauri::command]
pub fn delete(profile: i64, id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    with_profile(&state, profile, |s| s.delete(id))
}

#[tauri::command(async)]
pub fn tag_groups(state: State<'_, AppState>) -> Result<Vec<TagGroup>, String> {
    with_store(&state, Store::tag_groups)
}

#[tauri::command]
pub fn create_tag_group(
    profile: i64,
    name: String,
    state: State<'_, AppState>,
) -> Result<TagGroup, String> {
    with_profile(&state, profile, |s| s.create_tag_group(&name))
}

#[tauri::command]
pub fn rename_tag_group(
    profile: i64,
    id: i64,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    with_profile(&state, profile, |s| s.rename_tag_group(id, &name))
}

#[tauri::command]
pub fn delete_tag_group(profile: i64, id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    with_profile(&state, profile, |s| s.delete_tag_group(id))
}

/// `group_id`: `None` removes the tag from its group.
#[tauri::command]
pub fn set_tag_group(
    profile: i64,
    tag: String,
    group_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    with_profile(&state, profile, |s| s.set_tag_group(&tag, group_id))
}

#[tauri::command]
pub fn retry(profile: i64, id: i64, state: State<'_, AppState>) -> Result<(), String> {
    with_profile(&state, profile, |s| {
        s.get(id)?.msg(Msg::BookmarkNotFound)?;
        s.set_status(id, Status::Pending, None)
    })?;
    state.wake.notify_one();
    Ok(())
}

#[tauri::command(async)]
pub fn profiles(state: State<'_, AppState>) -> Result<ProfilesView, String> {
    let library = &state.library;
    let profiles = library.profiles().map_err(to_ipc)?;
    let active = library.active().map_err(to_ipc)?;
    Ok(ProfilesView { profiles, active })
}

#[tauri::command(async)]
pub fn create_profile(
    name: String,
    initials: String,
    state: State<'_, AppState>,
) -> Result<Profile, String> {
    state.library.create(&name, &initials).map_err(to_ipc)
}

#[tauri::command]
pub fn rename_profile(
    id: i64,
    name: String,
    initials: String,
    state: State<'_, AppState>,
) -> Result<Profile, String> {
    state.library.rename(id, &name, &initials).map_err(to_ipc)
}

#[tauri::command(async)]
pub fn delete_profile(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state.library.delete(id).map_err(to_ipc)
}

#[tauri::command]
pub fn switch_profile(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state.library.set_active(id).map_err(to_ipc)
}

/// The copy reaches the target as `pending_embed`: wakes the worker.
#[tauri::command(async)]
pub fn move_bookmark(
    id: i64,
    from: i64,
    to: i64,
    state: State<'_, AppState>,
) -> Result<LocalBookmark, String> {
    let moved = state.library.move_bookmark(id, from, to).map_err(to_ipc)?;
    state.wake.notify_one();
    Ok(moved.into())
}

#[tauri::command(async)]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsView, String> {
    let (settings, extraction) = with_primary(&state, |s| {
        Ok((settings::load(s)?, settings::load_extraction(s)?))
    })?;
    let has_key = |role| settings::api_key(role).map(|k| k.is_some()).map_err(to_ipc);
    Ok(SettingsView {
        settings,
        has_api_key: has_key(AiRole::Summary)?,
        extraction,
        has_extraction_key: has_key(AiRole::Extraction)?,
    })
}

/// `api_key`: `None` leaves the keychain alone; otherwise `settings::set_api_key` (empty deletes it).
#[tauri::command(async)]
pub fn save_settings(
    settings: AiSettings,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    with_primary(&state, |s| settings::save(s, &settings))?;
    let key_saved = api_key
        .as_deref()
        .map_or(Ok(()), |k| settings::set_api_key(AiRole::Summary, Some(k)));
    // Rebuild even if the keychain fails: the settings are already saved. An extraction model
    // failure is not about these settings: it is logged and shown when saving its own.
    let rebuilt = state.rebuild_model();
    if let Err(e) = &rebuilt.extraction {
        log::warn!("{}: {e:#}", Msg::NoExtractionModel);
    }
    let requeued = requeue_on_model_change(&state);
    key_saved.and(rebuilt.summary).and(requeued).map_err(to_ipc)
}

/// Like `save_settings`, for AI extraction; no requeue: embeddings do not change.
#[tauri::command(async)]
pub fn save_extraction_settings(
    settings: ExtractionSettings,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    with_primary(&state, |s| settings::save_extraction(s, &settings))?;
    let key_saved = api_key.as_deref().map_or(Ok(()), |k| {
        settings::set_api_key(AiRole::Extraction, Some(k))
    });
    key_saved
        .and(state.rebuild_model().extraction)
        .map_err(to_ipc)
}

/// "Extract again" with Defuddle, through the worker's path and skipping the queue.
#[tauri::command]
pub async fn reextract(
    profile: i64,
    id: i64,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let store = state.library.store(profile).map_err(to_ipc)?;
    let _extracting = extracting::mark(&app, profile, id);
    let replaced = pipeline::reextract(&store, id, &app_fetcher(&app)).await;
    requeued(replaced, &state, &app)
}

/// "Extract with AI" with the extraction model, skipping the queue.
#[tauri::command]
pub async fn extract_with_ai(
    profile: i64,
    id: i64,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let extraction = state.extraction_model().msg(Msg::NoExtractionModel);
    let extraction = extraction.map_err(to_ipc)?;
    let store = state.library.store(profile).map_err(to_ipc)?;
    let _extracting = extracting::mark(&app, profile, id);
    let load = |url| load_clean_html(&app, url);
    let replaced =
        pipeline::extract_with_ai(&store, id, load, &*extraction.model, &extraction.name).await;
    requeued(replaced, &state, &app)
}

/// After replacing a page, wakes the worker to summarize it and notifies the UI.
fn requeued(replaced: Result<()>, state: &AppState, app: &AppHandle) -> Result<(), String> {
    replaced.map_err(to_ipc)?;
    state.wake.notify_one();
    notify_changed(app);
    Ok(())
}

/// With a model, requeues every profile if the embedding model changed and wakes the worker: what
/// was extracted with AI off waits for its summary.
fn requeue_on_model_change(state: &AppState) -> Result<()> {
    let Some(model) = state.model() else {
        return Ok(());
    };
    state
        .library
        .requeue_if_embedding_changed(&model.embedding_id())?;
    state.wake.notify_one();
    Ok(())
}

/// Whether "Extract with AI" can run: an extraction model is built.
#[tauri::command(async)]
pub fn has_extraction_model(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.extraction_model().is_some())
}

/// Tests the role's saved settings.
#[tauri::command]
pub async fn test_connection(
    role: AiRole,
    state: State<'_, AppState>,
) -> Result<ConnectionReport, String> {
    let report = match role {
        AiRole::Summary => match state.model().msg(Msg::NoModel) {
            Ok(model) => probe(&*model).await,
            Err(e) => Err(e),
        },
        AiRole::Extraction => match state.extraction_model().msg(Msg::NoExtractionModel) {
            Ok(extraction) => probe_extraction(&*extraction.model).await,
            Err(e) => Err(e),
        },
    };
    report.map_err(to_ipc)
}

#[tauri::command(async)]
pub fn reindex(profile: i64, state: State<'_, AppState>) -> Result<usize, String> {
    waking_profile(&state, profile, Store::mark_ready_pending_embed)
}

#[tauri::command(async)]
pub fn process_unsummarized(profile: i64, state: State<'_, AppState>) -> Result<usize, String> {
    waking_profile(&state, profile, Store::mark_unsummarized_pending)
}

#[tauri::command(async)]
pub fn stats(state: State<'_, AppState>) -> Result<Stats, String> {
    with_store(&state, Store::stats)
}

/// A minimal summary and embedding, timed.
async fn probe(model: &dyn Model) -> Result<ConnectionReport> {
    let start = Instant::now();
    model.summarize(PROBE_INPUT).await?;
    let chat_ms = elapsed_ms(start);
    let start = Instant::now();
    let vectors = model.embed(&[PROBE_TEXT.to_string()]).await?;
    let embed_ms = elapsed_ms(start);
    let dim = vectors.first().map_or(0, Vec::len);
    Ok(ConnectionReport {
        chat_ms,
        embed_ms: Some(embed_ms),
        dim: Some(dim),
    })
}

/// A minimal extraction, timed.
async fn probe_extraction(model: &dyn Model) -> Result<ConnectionReport> {
    let start = Instant::now();
    model.extract_markdown(PROBE_HTML).await?;
    Ok(ConnectionReport {
        chat_ms: elapsed_ms(start),
        embed_ms: None,
        dim: None,
    })
}

fn elapsed_ms(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[tauri::command(async)]
pub fn api_info(profile: i64, state: State<'_, AppState>) -> Result<ApiInfo, String> {
    Ok(ApiInfo {
        port: API_PORT,
        token: state.library.token(profile).map_err(to_ipc)?,
        listening: state.api_status() == ApiStatus::Listening,
    })
}

/// The AI and local API state, for the status bar.
#[tauri::command(async)]
pub fn app_status(state: State<'_, AppState>) -> Result<AppStatusView, String> {
    Ok(state.app_status())
}

#[tauri::command]
pub fn regenerate_api_token(profile: i64, state: State<'_, AppState>) -> Result<String, String> {
    state.library.regenerate_token(profile).map_err(to_ipc)
}

/// Bookmarks being extracted right now, across all profiles.
#[tauri::command(async)]
pub fn extracting(state: State<'_, AppState>) -> Result<Vec<Extracting>, String> {
    Ok(crate::extracting::snapshot(&state))
}

/// Notifications for `profile` plus the global ones, most recent first. The window passes the
/// profile so the list matches the profile it shows, not the backend's active one.
#[tauri::command(async)]
pub fn notifications(
    profile: i64,
    state: State<'_, AppState>,
) -> Result<Vec<Notification>, String> {
    Ok(notifications::registry(&state).list(profile))
}

#[tauri::command]
pub fn push_notification(notice: NewNotification, app: AppHandle) -> Notification {
    notifications::notify(&app, notice)
}

#[tauri::command]
pub fn mark_notifications_read(profile: i64, state: State<'_, AppState>, app: AppHandle) {
    notifications::registry(&state).mark_read(profile);
    notifications::emit_changed(&app);
}

#[tauri::command]
pub fn dismiss_notification(id: u64, state: State<'_, AppState>, app: AppHandle) {
    notifications::registry(&state).dismiss(id);
    notifications::emit_changed(&app);
}

#[tauri::command]
pub fn clear_notifications(profile: i64, state: State<'_, AppState>, app: AppHandle) {
    notifications::registry(&state).clear(profile);
    notifications::emit_changed(&app);
}

/// The native file dialog with the HTML filter and `title`, as a sheet of `main` if it exists.
/// Only opened from Rust and off the main thread (`async` commands): the path never comes from
/// the webview.
fn html_dialog(app: &AppHandle, title: AppText) -> FileDialogBuilder<tauri::Wry> {
    let dialog = app
        .dialog()
        .file()
        .set_title(title.localize())
        .add_filter(AppText::HtmlFilter.localize(), HTML_EXTENSIONS);
    match app.get_webview_window(MAIN_WINDOW) {
        Some(main) => dialog.set_parent(&main),
        None => dialog,
    }
}

/// The path chosen in the native dialog; `None` if cancelled.
fn chosen(path: Option<FilePath>) -> Result<Option<PathBuf>, String> {
    path.map(FilePath::into_path)
        .transpose()
        .map_err(|e| to_ipc(e.into()))
}

/// Asks for a bookmarks file, reads it without the lock and counts it with the profile's lock,
/// writing nothing; the result waits in `AppState` for `import_confirm`. `None` if the dialog is
/// cancelled. The previous preview is dropped before opening it: cancelling leaves nothing to confirm.
#[tauri::command(async)]
pub fn import_preview(
    profile: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<ImportSummary>, String> {
    set_pending_import(&state.pending_import, None);
    let Some(path) = chosen(html_dialog(&app, AppText::ImportDialogTitle).blocking_pick_file())?
    else {
        return Ok(None);
    };
    let untagged = DataText::UntaggedFolder.all_locales();
    let entries = netscape::parse(&transfer::read_file(&path).map_err(to_ipc)?, &untagged);
    let reading_list = DataText::ReadingListTag.localize();
    let preview = with_profile(&state, profile, |s| {
        transfer::preview(s, entries, &reading_list)
    })?;
    let summary = preview.summary;
    set_pending_import(
        &state.pending_import,
        Some(PendingImport { profile, preview }),
    );
    Ok(Some(summary))
}

/// Applies the preview to its profile. Imports stay `imported`: the worker is not woken.
/// On failure the preview is put back, to retry without picking the file again.
#[tauri::command(async)]
pub fn import_confirm(
    profile: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ImportSummary, String> {
    let preview = take_pending_import(&state.pending_import, profile).map_err(to_ipc)?;
    let applied = with_profile(&state, profile, |s| transfer::apply(s, &preview));
    match &applied {
        Ok(_) => notify_changed(&app),
        Err(_) => set_pending_import(
            &state.pending_import,
            Some(PendingImport { profile, preview }),
        ),
    }
    applied
}

#[tauri::command(async)]
pub fn import_cancel(state: State<'_, AppState>) -> Result<(), String> {
    set_pending_import(&state.pending_import, None);
    Ok(())
}

/// Asks where to save, builds the HTML under the profile's lock and writes it without it. Returns
/// the written path, to show it; `None` if the dialog is cancelled.
#[tauri::command(async)]
pub fn export_bookmarks(
    profile: i64,
    suggested_name: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let dialog = html_dialog(&app, AppText::ExportDialogTitle).set_file_name(suggested_name);
    let Some(path) = chosen(dialog.blocking_save_file())? else {
        return Ok(None);
    };
    let untagged = DataText::UntaggedFolder.localize();
    let html = with_profile(&state, profile, |s| transfer::export_html(s, &untagged))?;
    std::fs::write(&path, html)
        .msg(Msg::ExportWrite)
        .map_err(to_ipc)?;
    Ok(Some(path.display().to_string()))
}

/// Moves the `ids` still `imported` to `pending` and wakes the worker.
#[tauri::command(async)]
pub fn extract_imported(
    profile: i64,
    ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    waking_profile(&state, profile, |s| s.queue_imported(&ids))
}

/// "Retry all" in the Failed filter: requeues the profile's failed bookmarks.
#[tauri::command(async)]
pub fn retry_failed(profile: i64, state: State<'_, AppState>) -> Result<usize, String> {
    waking_profile(&state, profile, Store::retry_failed)
}

/// The language setting and the language the app started with.
#[tauri::command(async)]
pub fn locale_info() -> locale::LocaleInfo {
    locale::info()
}

/// Saves the language setting (`system`, `es` or `en`) and relaunches the app to apply it. On the
/// pool: `restart` from another thread asks the event loop to exit and relaunches when it ends.
#[tauri::command(async)]
pub fn set_locale(app: AppHandle, setting: String) -> Result<(), String> {
    locale::save(&setting).map_err(to_ipc)?;
    app.restart()
}

/// What Settings → Updates shows. `supported`: whether this build checks on its own (`updater`).
#[derive(Serialize)]
pub struct UpdateInfo {
    current: String,
    available: Option<String>,
    supported: bool,
    auto: bool,
}

#[tauri::command(async)]
pub fn update_info(app: AppHandle, state: State<'_, AppState>) -> Result<UpdateInfo, String> {
    Ok(UpdateInfo {
        current: app.package_info().version.to_string(),
        available: updater::pending_version(&state),
        supported: updater::auto_check_supported(),
        auto: with_primary(&state, settings::load_update_settings)?.auto,
    })
}

/// Manual check: the new version, if any. The cause goes to the log; the user gets one sentence.
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<String>, String> {
    updater::check(&app).await.map_err(|e| {
        log::warn!("{LOG_UPDATE_FAILED}: {e:#}");
        AppText::UpdateCheckFailed.localize()
    })
}

/// Installs and restarts, like `set_locale`: it only returns when the install failed.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    updater::install(&app).await.map_err(|e| {
        log::warn!("{LOG_UPDATE_FAILED}: {e:#}");
        AppText::UpdateInstallFailed.localize()
    })?;
    app.restart()
}

#[tauri::command(async)]
pub fn set_update_auto(auto: bool, state: State<'_, AppState>) -> Result<(), String> {
    with_primary(&state, |store| {
        settings::save_update_settings(store, UpdateSettings { auto })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use marks_core::msg::{BookmarkError, ErrorPart};
    use marks_i18n::{Locale, localize_in};

    struct Ipc(fn() -> anyhow::Error);
    impl Localize for Ipc {
        fn localize(&self) -> String {
            to_ipc((self.0)())
        }
    }

    #[test]
    fn to_ipc_translates() {
        let e = Ipc(|| Msg::ProfileNotFound.err());
        assert_eq!(localize_in(Locale::Es, &e), "perfil no encontrado");
        assert_eq!(localize_in(Locale::En, &e), "profile not found");
    }

    #[test]
    fn local_bookmark_serializes_a_translated_error() {
        let store = Store::open_in_memory().expect("store");
        let new = NewBookmark {
            url: "https://example.com/a".into(),
            ..Default::default()
        };
        let b = store.add(new).expect("add");
        let failed = Bookmark {
            error: Some(BookmarkError(vec![ErrorPart::Msg(Msg::Blocked)])),
            ..b.clone()
        };
        struct Json(Bookmark);
        impl Localize for Json {
            fn localize(&self) -> String {
                serde_json::to_string(&LocalBookmark::from(self.0.clone())).expect("json")
            }
        }
        let json = |b: Bookmark| -> serde_json::Value {
            serde_json::from_str(&localize_in(Locale::En, &Json(b))).expect("json")
        };
        let failed = json(failed);
        assert_eq!(failed["error"], localize_in(Locale::En, &Msg::Blocked));
        assert_eq!(failed["id"], b.id);
        assert!(failed.get("error_msg").is_none());
        assert!(json(b)["error"].is_null());
    }
}
