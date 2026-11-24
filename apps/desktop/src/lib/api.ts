// Typed wrappers over the Tauri commands in src-tauri/src/commands.rs.
// Tauri maps snake_case Rust parameters to camelCase on the JS side (`api_key` → `apiKey`);
// returned structs keep their snake_case field names.
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { Locale } from './i18n';
import { t } from './strings';

/** Emitted by the backend (no payload) whenever bookmarks change; same value as `LIBRARY_CHANGED` in src-tauri/src/worker.rs. */
export const LIBRARY_CHANGED = 'library-changed';
/** Emitted by the tray menu (no payload) to open the add dialog; same value as `OPEN_ADD_DIALOG` in src-tauri/src/tray.rs. */
export const OPEN_ADD_DIALOG = 'open-add-dialog';
/** Emitted by the backend (no payload) when the set of bookmarks being extracted changes; same value as `EXTRACTING_CHANGED` in src-tauri/src/extracting.rs. */
export const EXTRACTING_CHANGED = 'extracting-changed';
/** Emitted by the backend (no payload) when the notification log changes; same value as `NOTIFICATIONS_CHANGED` in src-tauri/src/notifications.rs. */
export const NOTIFICATIONS_CHANGED = 'notifications-changed';
/** Emitted by the backend (no payload) when the AI or local API status changes; same value as `APP_STATUS_CHANGED` in src-tauri/src/status.rs. */
export const APP_STATUS_CHANGED = 'app-status-changed';

const EXTERNAL_URL = /^https?:\/\//i;

/** Opens an http(s) URL in the system browser; other schemes are rejected here and by the capability scope. */
export const openExternal = (url: string): Promise<void> =>
  EXTERNAL_URL.test(url) ? openUrl(url) : Promise.reject(new Error(t.errors.invalidExternalUrl));

/** `imported`: from a bookmarks file, not queued for extraction until the user asks. */
export type Status = 'pending' | 'pending_embed' | 'ready' | 'failed' | 'imported';

export type Provider = 'ollama' | 'omlx' | 'openai' | 'anthropic' | 'openai_compatible';

export type ErrorKind = 'blocked';

export interface Bookmark {
  id: number;
  /** Normalized: what deduplication compares. */
  url: string;
  /** As it arrived; what "Open in browser" opens. */
  original_url: string;
  title: string;
  summary: string | null;
  category: string | null;
  tags: string[];
  content: string | null;
  notes: string;
  status: Status;
  error: string | null;
  description: string | null;
  author: string | null;
  site: string | null;
  published: string | null;
  language: string | null;
  image: string | null;
  favicon: string | null;
  word_count: number | null;
  extractor: string | null;
  error_kind: ErrorKind | null;
  embedding_id: string | null;
  created_at: string;
  updated_at: string;
  /** When core tried to download `favicon`; `null` while still due. */
  favicon_at: string | null;
}

export interface SearchHit {
  bookmark: Bookmark;
  score: number;
  snippet: string | null;
}

/** A missing or `null` field is left unchanged. */
export interface BookmarkPatch {
  title?: string | null;
  notes?: string | null;
  tags?: string[] | null;
}

export interface AiSettings {
  enabled: boolean;
  provider: Provider;
  endpoint: string | null;
  chat_model: string;
  embed_model: string;
}

/** AI extraction; with `same_as_summary` it uses the summary connection and chat model. */
export interface ExtractionSettings {
  same_as_summary: boolean;
  provider: Provider;
  endpoint: string | null;
  model: string;
}

/** Each role has its own connection and its own key in the keychain. */
export type AiRole = 'summary' | 'extraction';

export interface SettingsView {
  settings: AiSettings;
  has_api_key: boolean;
  extraction: ExtractionSettings;
  has_extraction_key: boolean;
}

export interface ConnectionReport {
  chat_ms: number;
  embed_ms: number;
  dim: number;
}

/** The extraction test only times the chat model. */
export interface ExtractionReport {
  chat_ms: number;
}

export interface Stats {
  total: number;
  pending: number;
  failed: number;
  unsummarized: number;
  /** Pending and not extracted yet; without AI the extracted ones stay pending, waiting for a model. */
  unextracted: number;
  /** Imported and not queued yet; not counted in `pending`. */
  imported: number;
}

export interface TagGroup {
  id: number;
  name: string;
  tags: string[];
}

export interface Profile {
  id: number;
  name: string;
  initials: string;
}

export interface ProfilesView {
  profiles: Profile[];
  active: number;
}

export interface ApiInfo {
  port: number;
  token: string;
  listening: boolean;
}

/** The summary model: off, ready, or failed (`model: null` when it could not be built). `error` comes translated. */
export type AiStatus =
  | { kind: 'off' }
  | { kind: 'ready'; model: string }
  | { kind: 'failed'; model: string | null; error: string };
export type ApiStatus = 'starting' | 'listening' | 'busy' | 'stopped';
export interface AppStatus {
  ai: AiStatus;
  api: ApiStatus;
  api_port: number;
}

// One function per command, camelCased; `delete` is a reserved word, so it is `remove`.
export const list = () => invoke<Bookmark[]>('list');
export const search = (query: string) => invoke<SearchHit[]>('search', { query });
// Bookmark and group ids repeat across profiles: the commands that name one or write also name
// the profile they were started in.
export const get = (profile: number, id: number) => invoke<Bookmark | null>('get', { profile, id });
/** The stored favicon as a `data:` URL; `null` when there is none. */
export const bookmarkFavicon = (profile: number, id: number) =>
  invoke<string | null>('bookmark_favicon', { profile, id });
export const add = (url: string, notes: string, tags: string[]) =>
  invoke<Bookmark>('add', { url, notes, tags });
export const update = (profile: number, id: number, patch: BookmarkPatch) =>
  invoke<Bookmark>('update', { profile, id, patch });
export const remove = (profile: number, id: number) => invoke<boolean>('delete', { profile, id });
export const retry = (profile: number, id: number) => invoke<void>('retry', { profile, id });
/** Queues the `imported` bookmarks among `ids`; resolves to how many it queued. */
export const extractImported = (profile: number, ids: number[]) =>
  invoke<number>('extract_imported', { profile, ids });
/** Queues every failed bookmark of `profile`; resolves to how many it queued. */
export const retryFailed = (profile: number) => invoke<number>('retry_failed', { profile });
export const getSettings = () => invoke<SettingsView>('get_settings');
/** `apiKey`: `null` keeps the stored key, `""` deletes it. */
export const saveSettings = (settings: AiSettings, apiKey: string | null) =>
  invoke<void>('save_settings', { settings, apiKey });
export const saveExtractionSettings = (settings: ExtractionSettings, apiKey: string | null) =>
  invoke<void>('save_extraction_settings', { settings, apiKey });
/** Tests the saved settings of `role`. */
export function testConnection(role: 'summary'): Promise<ConnectionReport>;
export function testConnection(role: 'extraction'): Promise<ExtractionReport>;
export function testConnection(role: AiRole) {
  return invoke('test_connection', { role });
}
/** Re-extracts with Defuddle; the content changes only if the result is not thin. */
export const reextract = (profile: number, id: number) =>
  invoke<void>('reextract', { profile, id });
/** Extracts with the AI extraction model; the content changes only if the result is not thin. */
export const extractWithAi = (profile: number, id: number) =>
  invoke<void>('extract_with_ai', { profile, id });
/** Whether "Extract with AI" can run: the extraction model was built. */
export const hasExtractionModel = () => invoke<boolean>('has_extraction_model');
export const reindex = (profile: number) => invoke<number>('reindex', { profile });
export const processUnsummarized = (profile: number) =>
  invoke<number>('process_unsummarized', { profile });
export const stats = () => invoke<Stats>('stats');
/** Mirrors `ImportSummary` in crates/core: what a file would add to a profile. */
export interface ImportSummary {
  new: number;
  existing: number;
  unsupported: number;
  invalid: number;
}
/** Opens the native open dialog and reads the file without writing; `null` if it was cancelled. */
export const importPreview = (profile: number) =>
  invoke<ImportSummary | null>('import_preview', { profile });
/** Applies the pending preview; rejected if it was read for another profile. */
export const importConfirm = (profile: number) =>
  invoke<ImportSummary>('import_confirm', { profile });
export const importCancel = () => invoke<void>('import_cancel');
/** Opens the native save dialog and writes the file; the path written, or `null` if cancelled. */
export const exportBookmarks = (profile: number, suggestedName: string) =>
  invoke<string | null>('export_bookmarks', { profile, suggestedName });
/** A bookmark being extracted right now; ids repeat across profiles, so both travel together. */
export interface Extracting {
  profile: number;
  id: number;
}
export const extracting = () => invoke<Extracting[]>('extracting');
export type Severity = 'error' | 'warning' | 'info';
/** What a notification leads to; decides its actions in the panel. */
export type Target =
  { kind: 'bookmark'; id: number } | { kind: 'ai' } | { kind: 'extension' } | { kind: 'updates' };
/** `profile: null` is global: it shows in every profile. */
export interface NewNotification {
  profile: number | null;
  severity: Severity;
  message: string;
  detail: string | null;
  target: Target | null;
}
/** A grouped entry of the log: `count` repeats, `last_at` in ms since the UNIX epoch. */
export interface Notification extends NewNotification {
  id: number;
  count: number;
  last_at: number;
  read: boolean;
}
/** The profile's notifications plus the global ones, newest first. */
export const notifications = (profile: number) =>
  invoke<Notification[]>('notifications', { profile });
export const pushNotification = (notice: NewNotification) =>
  invoke<Notification>('push_notification', { notice });
export const markNotificationsRead = (profile: number) =>
  invoke<void>('mark_notifications_read', { profile });
export const dismissNotification = (id: number) => invoke<void>('dismiss_notification', { id });
/** Also clears the global ones, in every profile. */
export const clearNotifications = (profile: number) =>
  invoke<void>('clear_notifications', { profile });
export const apiInfo = (profile: number) => invoke<ApiInfo>('api_info', { profile });
export const appStatus = () => invoke<AppStatus>('app_status');
export const regenerateApiToken = (profile: number) =>
  invoke<string>('regenerate_api_token', { profile });
export const tagGroups = () => invoke<TagGroup[]>('tag_groups');
export const createTagGroup = (profile: number, name: string) =>
  invoke<TagGroup>('create_tag_group', { profile, name });
export const renameTagGroup = (profile: number, id: number, name: string) =>
  invoke<void>('rename_tag_group', { profile, id, name });
export const deleteTagGroup = (profile: number, id: number) =>
  invoke<boolean>('delete_tag_group', { profile, id });
export const profiles = () => invoke<ProfilesView>('profiles');
export const createProfile = (name: string, initials: string) =>
  invoke<Profile>('create_profile', { name, initials });
export const renameProfile = (id: number, name: string, initials: string) =>
  invoke<Profile>('rename_profile', { id, name, initials });
export const deleteProfile = (id: number) => invoke<void>('delete_profile', { id });
export const switchProfile = (id: number) => invoke<void>('switch_profile', { id });
export const moveBookmark = (id: number, from: number, to: number) =>
  invoke<Bookmark>('move_bookmark', { id, from, to });
/** `groupId: null` takes the tag out of its group. */
export const setTagGroup = (profile: number, tag: string, groupId: number | null) =>
  invoke<void>('set_tag_group', { profile, tag, groupId });

/** The language setting and the locale the backend resolved at startup (`pseudo` only with `MARKS_LOCALE=pseudo`). */
export interface LocaleInfo {
  setting: 'system' | Locale;
  active: Locale | 'pseudo';
}

export const localeInfo = () => invoke<LocaleInfo>('locale_info');
/** Writes the language and relaunches marks: it only returns (rejecting) when that failed. */
export const setLocale = (setting: LocaleInfo['setting']) =>
  invoke<void>('set_locale', { setting });

/** Settings → Updates. `supported`: this build checks on its own (Apple silicon, release builds). */
export interface UpdateInfo {
  current: string;
  /** The version the last check found, waiting to be installed. */
  available: string | null;
  supported: boolean;
  auto: boolean;
}

export const updateInfo = () => invoke<UpdateInfo>('update_info');
/** The new version, or `null` when this one is the latest; rejects with a translated sentence. */
export const checkUpdate = () => invoke<string | null>('check_update');
/** Downloads and installs the version `checkUpdate` found and restarts marks: it only returns (rejecting) when that failed. */
export const installUpdate = () => invoke<void>('install_update');
export const setUpdateAuto = (auto: boolean) => invoke<void>('set_update_auto', { auto });
