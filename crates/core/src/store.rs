//! SQLite as the single source of truth, with FTS5 for search.

use crate::ai::Summary;
use crate::download::Image;
use crate::extract::{Page, PageMeta};
use crate::msg::{BookmarkError, ErrorPart, Msg, MsgExt, failure_msg};
use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, Params, Transaction, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, Once};
use std::time::Duration;

/// Five states: `pending`, `pending_embed`, `ready`, `failed` and `imported` (imported and not yet
/// queued: not extracted and not counted as pending). Any other value is a bug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pending,
    PendingEmbed,
    Ready,
    Failed,
    Imported,
}

const STATUS_PENDING: &str = "pending";
const STATUS_PENDING_EMBED: &str = "pending_embed";
const STATUS_READY: &str = "ready";
const STATUS_FAILED: &str = "failed";
macro_rules! status_imported {
    () => {
        "imported"
    };
}
const STATUS_IMPORTED: &str = status_imported!();

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Pending => STATUS_PENDING,
            Status::PendingEmbed => STATUS_PENDING_EMBED,
            Status::Ready => STATUS_READY,
            Status::Failed => STATUS_FAILED,
            Status::Imported => STATUS_IMPORTED,
        }
    }
}

impl std::str::FromStr for Status {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            STATUS_PENDING => Ok(Status::Pending),
            STATUS_PENDING_EMBED => Ok(Status::PendingEmbed),
            STATUS_READY => Ok(Status::Ready),
            STATUS_FAILED => Ok(Status::Failed),
            STATUS_IMPORTED => Ok(Status::Imported),
            other => Err(Msg::UnknownStatus {
                value: other.into(),
            }
            .err()),
        }
    }
}

/// Why a bookmark failed, when the cause has its own handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Blocked,
}

const ERROR_KIND_BLOCKED: &str = "blocked";

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::Blocked => ERROR_KIND_BLOCKED,
        }
    }
}

impl std::str::FromStr for ErrorKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            ERROR_KIND_BLOCKED => Ok(ErrorKind::Blocked),
            other => Err(Msg::UnknownErrorKind {
                value: other.into(),
            }
            .err()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub url: String,
    /// The URL as saved or imported, not normalized; equal to `url` for anything before v0.7.
    pub original_url: String,
    pub title: String,
    pub summary: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub content: Option<String>,
    pub notes: String,
    pub status: Status,
    /// Not serialized: whoever shows it translates it first.
    #[serde(skip)]
    pub error: Option<BookmarkError>,
    #[serde(flatten)]
    pub meta: PageMeta,
    pub error_kind: Option<ErrorKind>,
    pub embedding_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// When the favicon download was attempted; `None` if still due (`next_without_favicon`).
    pub favicon_at: Option<String>,
}

#[derive(Default)]
pub struct NewBookmark {
    pub url: String,
    /// `None` stores the normalized URL as the title.
    pub title: Option<String>,
    pub notes: String,
    pub tags: Vec<String>,
    pub content: Option<String>,
    pub meta: PageMeta,
}

/// User-editable fields; `None` leaves the field as is.
#[derive(Deserialize)]
pub struct BookmarkPatch {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// A bookmark as it appears in the `bookmarks.html` export.
pub struct ExportRow {
    /// The original URL.
    pub url: String,
    pub title: String,
    pub tags: Vec<String>,
    /// `created_at` in Unix seconds.
    pub add_date: i64,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Stats {
    pub total: usize,
    /// `pending` and `pending_embed`.
    pub pending: usize,
    pub failed: usize,
    pub unsummarized: usize,
    /// `pending` with no content: still waiting to be extracted. Without AI, extracted ones stay
    /// `pending` waiting for a model and are not work in progress.
    pub unextracted: usize,
    /// `imported`: imported but not yet queued; not counted as pending.
    pub imported: usize,
}

pub struct Store {
    pub(crate) conn: Connection,
}

/// The v1.0 schema: what migrations 1 to 9 left in v0.7 without `bookmarks.error`, plus `meta`.
/// Published: never edited. `vec_chunks` is not here: `vectors` creates it with the model's
/// dimension. A tag is in at most one group (`tag` is the key of `tag_group_members`).
/// `original_url` is the URL as it came; `url` is the normalized one, which deduplicates.
const SCHEMA_1: &str = r#"
CREATE TABLE bookmarks (
    id               INTEGER PRIMARY KEY,
    url              TEXT NOT NULL UNIQUE,
    title            TEXT NOT NULL,
    summary          TEXT,
    category         TEXT,
    tags             TEXT NOT NULL DEFAULT '',
    content          TEXT,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    notes            TEXT NOT NULL DEFAULT '',
    status           TEXT NOT NULL DEFAULT 'pending',
    embedding_id     TEXT,
    description      TEXT,
    author           TEXT,
    site             TEXT,
    published        TEXT,
    language         TEXT,
    image_url        TEXT,
    favicon_url      TEXT,
    word_count       INTEGER,
    extractor        TEXT,
    error_kind       TEXT,
    favicon          BLOB,
    favicon_type     TEXT,
    favicon_at       TEXT,
    favicon_failures INTEGER NOT NULL DEFAULT 0,
    favicon_retry_at TEXT,
    original_url     TEXT,
    error_msg        TEXT
);
CREATE VIRTUAL TABLE bookmarks_fts USING fts5(
    title, summary, tags, notes, description, author, site, content,
    content='bookmarks', content_rowid='id'
);
CREATE TRIGGER bookmarks_ai AFTER INSERT ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(rowid, title, summary, tags, notes, description, author, site, content)
    VALUES (new.id, new.title, new.summary, new.tags, new.notes, new.description, new.author, new.site, new.content);
END;
CREATE TRIGGER bookmarks_ad AFTER DELETE ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, title, summary, tags, notes, description, author, site, content)
    VALUES ('delete', old.id, old.title, old.summary, old.tags, old.notes, old.description, old.author, old.site, old.content);
END;
CREATE TRIGGER bookmarks_au AFTER UPDATE ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, title, summary, tags, notes, description, author, site, content)
    VALUES ('delete', old.id, old.title, old.summary, old.tags, old.notes, old.description, old.author, old.site, old.content);
    INSERT INTO bookmarks_fts(rowid, title, summary, tags, notes, description, author, site, content)
    VALUES (new.id, new.title, new.summary, new.tags, new.notes, new.description, new.author, new.site, new.content);
END;
CREATE TABLE chunks (
    id          INTEGER PRIMARY KEY,
    bookmark_id INTEGER NOT NULL REFERENCES bookmarks(id) ON DELETE CASCADE,
    ord         INTEGER NOT NULL,
    text        TEXT NOT NULL
);
CREATE INDEX chunks_bookmark_idx ON chunks(bookmark_id);
CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE tag_groups (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE
);
CREATE TABLE tag_group_members (
    tag      TEXT PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES tag_groups(id) ON DELETE CASCADE
);
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
"#;

/// A migration: SQL, or Rust when rows must be read and rewritten.
enum Migration {
    Sql(&'static str),
    // No migration rewrites rows yet; this is the shape one would take.
    #[expect(dead_code)]
    Rust(fn(&Transaction) -> Result<()>),
}

/// Whether an app that stops at the previous version can still read a database after this
/// migration. `Breaking` if it drops or renames columns or tables, changes what a value means, or
/// would make an older app write wrong data; `Additive` otherwise.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Compat {
    #[cfg_attr(not(test), expect(dead_code))]
    Additive,
    Breaking,
}

/// Indexed by version: `MIGRATIONS[n - 1]` takes the database from version n - 1 to n.
const MIGRATIONS: &[(Compat, Migration)] = &[(Compat::Breaking, Migration::Sql(SCHEMA_1))];

/// Key in `meta`: the oldest database version an app must know to read this one.
const META_MIN_READER: &str = "min_reader";

/// The version of the last `Breaking` migration at or below `version` (0 if none).
fn min_reader(migrations: &[(Compat, Migration)], version: usize) -> usize {
    migrations[..version]
        .iter()
        .rposition(|(compat, _)| *compat == Compat::Breaking)
        .map_or(0, |index| index + 1)
}

#[derive(Debug, PartialEq)]
enum Verdict {
    Open,
    TooNew,
    Unknown,
}

/// Whether this app may open a database. `empty`: nothing in `sqlite_master`. `known`: how many
/// migrations this app has. A newer database whose `min_reader` this app knows opens unmigrated.
fn verdict(version: usize, min_reader: Option<usize>, empty: bool, known: usize) -> Verdict {
    match min_reader {
        _ if version == 0 && empty => Verdict::Open,
        None => Verdict::Unknown,
        Some(needed) if needed > known => Verdict::TooNew,
        Some(_) => Verdict::Open,
    }
}

/// Ids per statement in `queue_imported`, below SQLite's variable limit.
const QUEUE_CHUNK: usize = 500;

/// Tags are stored in a single column separated by this character.
const TAG_SEPARATOR: char = ',';

/// Refreshes `updated_at` in any `UPDATE`.
pub(crate) const TOUCH: &str = "updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')";
const UNSUMMARIZED: &str = "summary IS NULL"; // i18n-allow: sql
/// No extracted content: empty or whitespace only.
const NO_CONTENT: &str = "(content IS NULL OR TRIM(content, ' ' || char(9, 10, 13)) = '')"; // i18n-allow: sql

/// The original URL, with `prefix` as the table alias (`""` or `"b."`).
macro_rules! original_url {
    ($prefix:literal) => {
        concat!("COALESCE(", $prefix, "original_url, ", $prefix, "url)")
    };
}
/// `COLUMNS` except the last one (`original_url`); search prefixes them with `b.`.
macro_rules! columns_head {
    () => {
        "id, url, title, summary, category, tags, content, notes, status, error_msg, embedding_id, created_at, updated_at, description, author, site, published, language, image_url, favicon_url, word_count, extractor, error_kind, favicon_at"
    };
}
const COLUMNS: &str = concat!(columns_head!(), ", ", original_url!(""));
/// Like `COLUMNS` but with `content` as NULL, for listings that do not need the Markdown.
const COLUMNS_LIGHT: &str = concat!(
    "id, url, title, summary, category, tags, NULL, notes, status, error_msg, embedding_id, created_at, updated_at, description, author, site, published, language, image_url, favicon_url, word_count, extractor, error_kind, favicon_at",
    ", ",
    original_url!("")
);
/// Favicon not fully attempted yet: neither stored nor discarded (`set_favicon`).
const FAVICON_PENDING: &str = "favicon_at IS NULL AND favicon_url IS NOT NULL"; // i18n-allow: sql
/// Pending and, after a transient failure, past its delay (`retry_favicon`).
const FAVICON_DUE: &str = // i18n-allow: sql
    concat!(
        "status != '",
        status_imported!(),
        "' AND favicon_at IS NULL AND favicon_url IS NOT NULL
    AND (favicon_retry_at IS NULL OR favicon_retry_at <= strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))"
    );

/// Takes the store lock, with a readable error if it was poisoned.
pub fn lock(store: &Mutex<Store>) -> Result<MutexGuard<'_, Store>> {
    store.lock().map_err(|_| Msg::StorePoisoned.err())
}

impl Store {
    pub fn open(path: &Path) -> Result<Store> {
        Self::open_with(path, MIGRATIONS)
    }

    /// `open` with another list of migrations, for tests.
    fn open_with(path: &Path, migrations: &[(Compat, Migration)]) -> Result<Store> {
        register_vec_extension();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).msg(Msg::OpenDatabase {
                path: path.display().to_string(),
            })?;
        }
        let conn = Connection::open(path).msg(Msg::OpenDatabase {
            path: path.display().to_string(),
        })?;
        Self::start(conn, migrations, Some(path)).map_err(|e| with_path(e, path))
    }

    pub fn open_in_memory() -> Result<Store> {
        register_vec_extension();
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Store> {
        Self::init_with(conn, MIGRATIONS)
    }

    /// `init` with another list of migrations, for tests.
    fn init_with(conn: Connection, migrations: &[(Compat, Migration)]) -> Result<Store> {
        Self::start(conn, migrations, None)
    }

    /// Checks, backs up (only a database `file` with migrations pending) and migrates.
    fn start(
        conn: Connection,
        migrations: &[(Compat, Migration)],
        file: Option<&Path>,
    ) -> Result<Store> {
        // `chunks` relies on ON DELETE CASCADE; do not trust SQLite's default.
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let version = check(&conn, migrations)?;
        // The CLI and the app may open the database at the same time. Only once it is known to be
        // ours, since WAL changes the file header; in memory it stays `memory`.
        conn.pragma_update(None, "journal_mode", "wal")?;
        if let Some(path) = file
            && (1..migrations.len()).contains(&version)
        {
            backup_before_migrating(&conn, path, version)?;
        }
        migrate(&conn, migrations)?;
        Ok(Store { conn })
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
                r.get(0)
            })
            .optional()?)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Inserts or, if the normalized URL already exists, returns the existing bookmark untouched,
    /// except an `imported` one, which becomes `pending`.
    pub fn add(&self, new: NewBookmark) -> Result<Bookmark> {
        let url = crate::normalize_url(&new.url)?;
        if let Some(existing) = self.find_by_url(&url)? {
            if existing.status != Status::Imported {
                return Ok(existing);
            }
            self.set_status(existing.id, Status::Pending, None)?;
            return self.get(existing.id)?.msg(Msg::BookmarkNotFound);
        }
        self.insert_new(&new, &url, STATUS_PENDING, None)
    }

    /// Inserts an `imported` bookmark, using `add_date` (Unix seconds) as the creation date if given.
    /// Fails if the URL already exists: the caller checks first.
    pub(crate) fn insert_imported(&self, new: &NewBookmark, add_date: Option<i64>) -> Result<i64> {
        let url = crate::normalize_url(&new.url)?;
        Ok(self.insert_new(new, &url, STATUS_IMPORTED, add_date)?.id)
    }

    /// Moves the `imported` bookmarks in `ids` to `pending`. Returns how many moved.
    pub fn queue_imported(&self, ids: &[i64]) -> Result<usize> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let mut moved = 0;
        for chunk in ids.chunks(QUEUE_CHUNK) {
            let marks = vec!["?"; chunk.len()].join(", ");
            let mut args: Vec<&dyn rusqlite::ToSql> = vec![&STATUS_PENDING, &STATUS_IMPORTED];
            args.extend(chunk.iter().map(|id| id as &dyn rusqlite::ToSql));
            moved += tx.execute(
                &format!(
                    "UPDATE bookmarks SET status = ?1, {TOUCH} WHERE status = ?2 AND id IN ({marks})"
                ),
                args.as_slice(),
            )?;
        }
        tx.commit()?;
        Ok(moved)
    }

    fn insert_new(
        &self,
        new: &NewBookmark,
        url: &str,
        status: &str,
        add_date: Option<i64>,
    ) -> Result<Bookmark> {
        let title = title_or_url(new.title.clone(), url);
        let m = new.meta.clone().sanitized();
        self.conn.execute(
            "INSERT INTO bookmarks (url, title, notes, tags, content, description, author, site,
                 published, language, image_url, favicon_url, word_count, extractor, original_url,
                 status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                 COALESCE(strftime('%Y-%m-%dT%H:%M:%SZ', ?17, 'unixepoch'),
                          strftime('%Y-%m-%dT%H:%M:%SZ', 'now')))",
            params![
                url,
                title,
                new.notes,
                join_tags(&normalize_tags(&new.tags)),
                new.content,
                m.description,
                m.author,
                m.site,
                m.published,
                m.language,
                m.image,
                m.favicon,
                m.word_count,
                m.extractor,
                new.url.trim(),
                status,
                add_date
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        self.get(id)?.msg(Msg::BookmarkNotFound)
    }

    /// Inserts a copy with its fields and favicon (no `id` or `embedding_id`); the URL is already
    /// normalized.
    pub(crate) fn insert_copy(&self, b: &Bookmark, favicon: &Option<Image>) -> Result<Bookmark> {
        let (favicon, favicon_type) = image_columns(favicon);
        self.conn.execute(
            "INSERT INTO bookmarks (url, title, summary, category, tags, content, notes, status, error_msg, created_at,
                 description, author, site, published, language, image_url, favicon_url, word_count, extractor, error_kind,
                 favicon_at, favicon, favicon_type, original_url)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
                 ?21, ?22, ?23, ?24)",
            params![
                b.url,
                b.title,
                b.summary,
                b.category,
                join_tags(&b.tags),
                b.content,
                b.notes,
                b.status.as_str(),
                b.error.as_ref().map(BookmarkError::to_json),
                b.created_at,
                b.meta.description,
                b.meta.author,
                b.meta.site,
                b.meta.published,
                b.meta.language,
                b.meta.image,
                b.meta.favicon,
                b.meta.word_count,
                b.meta.extractor,
                b.error_kind.map(|k| k.as_str()),
                b.favicon_at,
                favicon,
                favicon_type,
                b.original_url
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        self.get(id)?.msg(Msg::BookmarkNotFound)
    }

    /// Applies only the fields present in the patch. Fails if the bookmark does not exist.
    pub fn update(&self, id: i64, patch: BookmarkPatch) -> Result<Bookmark> {
        let current = self.get(id)?.msg(Msg::BookmarkNotFound)?;
        let title = patch
            .title
            .map_or(current.title, |t| title_or_url(Some(t), &current.url));
        let notes = patch.notes.unwrap_or(current.notes);
        let tags = patch.tags.as_deref().map_or(current.tags, normalize_tags);
        self.conn.execute(
            &format!(
                "UPDATE bookmarks SET title = ?1, notes = ?2, tags = ?3, {TOUCH} WHERE id = ?4"
            ),
            params![title, notes, join_tags(&tags), id],
        )?;
        self.get(id)?.msg(Msg::BookmarkNotFound)
    }

    /// Replaces the tags without touching `updated_at`: import changes nothing else about an
    /// existing bookmark, including its place in the queue.
    pub(crate) fn set_tags_only(&self, id: i64, tags: &[String]) -> Result<()> {
        self.conn.execute(
            "UPDATE bookmarks SET tags = ?1 WHERE id = ?2",
            params![join_tags(&normalize_tags(tags)), id],
        )?;
        Ok(())
    }

    /// Returns `true` if it existed. Deletes its vectors first: `chunks` cascades, but `vec_chunks`
    /// does not, and its rowids would be reused.
    pub fn delete(&self, id: i64) -> Result<bool> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        self.delete_bookmark_vectors(id)?;
        let n = tx.execute("DELETE FROM bookmarks WHERE id = ?1", [id])?;
        tx.commit()?;
        Ok(n > 0)
    }

    pub fn set_content(&self, id: i64, markdown: &str) -> Result<()> {
        self.conn.execute(
            &format!("UPDATE bookmarks SET content = ?1, {TOUCH} WHERE id = ?2"),
            params![markdown, id],
        )?;
        Ok(())
    }

    pub fn set_status(&self, id: i64, status: Status, error: Option<&BookmarkError>) -> Result<()> {
        self.conn.execute(
            &format!(
                "UPDATE bookmarks SET status = ?1, error_msg = ?2, error_kind = NULL, {TOUCH}
                 WHERE id = ?3"
            ),
            params![status.as_str(), error.map(BookmarkError::to_json), id],
        )?;
        Ok(())
    }

    pub fn set_failed(&self, id: i64, err: &BookmarkError) -> Result<()> {
        self.set_status(id, Status::Failed, Some(err))
    }

    /// Content, sanitized metadata and `extractor` in one UPDATE; the title only if it is still the
    /// URL. The favicon is fetched again from scratch: no `favicon_at` and no retries.
    pub fn set_page(&self, id: i64, page: &Page) -> Result<()> {
        let current = self.get(id)?.msg(Msg::BookmarkNotFound)?;
        let title = if current.title == current.url {
            title_or_url(page.title.clone(), &current.url)
        } else {
            current.title
        };
        let m = page.meta.clone().sanitized();
        self.conn.execute(
            &format!(
                "UPDATE bookmarks SET content = ?1, title = ?2, description = ?3, author = ?4,
                 site = ?5, published = ?6, language = ?7, image_url = ?8, favicon_url = ?9,
                 word_count = ?10, extractor = ?11, favicon_at = NULL,
                 favicon_failures = 0, favicon_retry_at = NULL, {TOUCH} WHERE id = ?12"
            ),
            params![
                page.markdown,
                title,
                m.description,
                m.author,
                m.site,
                m.published,
                m.language,
                m.image,
                m.favicon,
                m.word_count,
                m.extractor,
                id
            ],
        )?;
        Ok(())
    }

    /// Replaces the page from a re-extraction in one transaction: `set_page` (the user's title and
    /// tags are kept) and back to `pending` with no error or block, so the worker summarizes and
    /// indexes again.
    pub fn replace_page(&self, id: i64, page: &Page) -> Result<()> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        self.set_page(id, page)?;
        self.set_status(id, Status::Pending, None)?;
        tx.commit()?;
        Ok(())
    }

    /// The most recently queued (highest `updated_at` and `id`) whose favicon is still due, without `content`.
    pub fn next_without_favicon(&self) -> Result<Option<Bookmark>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "SELECT {COLUMNS_LIGHT} FROM bookmarks WHERE {FAVICON_DUE}
                     ORDER BY updated_at DESC, id DESC LIMIT 1"
                ),
                [],
                row_to_bookmark,
            )
            .optional()?)
    }

    /// Stores the favicon downloaded from `from` (or none, if it failed) and marks the attempt. Does
    /// nothing if its URL changed meanwhile (a new extraction): it is then tried again.
    pub fn set_favicon(&self, from: &Bookmark, favicon: &Option<Image>) -> Result<()> {
        let (bytes, mime) = image_columns(favicon);
        self.conn.execute(
            &format!(
                "UPDATE bookmarks SET favicon = ?1, favicon_type = ?2,
                 favicon_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?3 AND {FAVICON_PENDING} AND favicon_url IS ?4"
            ),
            params![bytes, mime, from.id, from.meta.favicon],
        )?;
        Ok(())
    }

    /// Transient favicon failures since the last extraction.
    pub fn favicon_failures(&self, id: i64) -> Result<u32> {
        self.conn
            .query_row(
                "SELECT favicon_failures FROM bookmarks WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .optional()?
            .msg(Msg::BookmarkNotFound)
    }

    /// Counts a transient favicon failure for `from` and leaves it due again after `after`.
    /// Like `set_favicon`, does nothing if its URL changed meanwhile.
    pub fn retry_favicon(&self, from: &Bookmark, after: Duration) -> Result<()> {
        self.conn.execute(
            &format!(
                "UPDATE bookmarks SET favicon_failures = favicon_failures + 1,
                 favicon_retry_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?1)
                 WHERE id = ?2 AND {FAVICON_PENDING} AND favicon_url IS ?3"
            ),
            params![
                format!("+{} seconds", after.as_secs()),
                from.id,
                from.meta.favicon
            ],
        )?;
        Ok(())
    }

    pub fn stored_favicon(&self, id: i64) -> Result<Option<Image>> {
        self.conn
            .query_row(
                "SELECT favicon, favicon_type FROM bookmarks WHERE id = ?1",
                [id],
                |r| Ok(stored_image(r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .msg(Msg::BookmarkNotFound)
    }

    /// The stored favicon as a `data:` URL, which the UI can show under its CSP.
    pub fn favicon_data_url(&self, id: i64) -> Result<Option<String>> {
        Ok(self.stored_favicon(id)?.as_ref().map(Image::data_url))
    }

    /// Fills only the metadata columns that are NULL; never `extractor`.
    pub fn fill_missing_meta(&self, id: i64, meta: &PageMeta) -> Result<()> {
        let m = meta.clone().sanitized();
        self.conn.execute(
            &format!(
                "UPDATE bookmarks SET description = COALESCE(description, ?1),
                 author = COALESCE(author, ?2), site = COALESCE(site, ?3),
                 published = COALESCE(published, ?4), language = COALESCE(language, ?5),
                 image_url = COALESCE(image_url, ?6), favicon_url = COALESCE(favicon_url, ?7),
                 word_count = COALESCE(word_count, ?8), {TOUCH} WHERE id = ?9"
            ),
            params![
                m.description,
                m.author,
                m.site,
                m.published,
                m.language,
                m.image,
                m.favicon,
                m.word_count,
                id
            ],
        )?;
        Ok(())
    }

    /// `failed` with the anti-bot check error and its kind.
    pub fn set_blocked(&self, id: i64) -> Result<()> {
        self.conn.execute(
            &format!(
                "UPDATE bookmarks SET status = ?1, error_msg = ?2, error_kind = ?3, {TOUCH}
                 WHERE id = ?4"
            ),
            params![
                STATUS_FAILED,
                BookmarkError(vec![ErrorPart::Msg(Msg::Blocked)]).to_json(),
                ErrorKind::Blocked.as_str(),
                id
            ],
        )?;
        Ok(())
    }

    /// Like `next_pending` but only `pending` without content (NULL or blank), most recently queued first.
    pub fn next_without_content(&self) -> Result<Option<Bookmark>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "SELECT {COLUMNS} FROM bookmarks
                     WHERE status = ?1 AND {NO_CONTENT}
                     ORDER BY updated_at DESC, id DESC LIMIT 1"
                ),
                [STATUS_PENDING],
                row_to_bookmark,
            )
            .optional()?)
    }

    /// `summary` and `category` always come from the AI; `title` and `tags` are only filled in if the
    /// user has not set them (title equal to the URL, no tags).
    pub fn apply_summary(&self, id: i64, s: &Summary) -> Result<()> {
        let current = self.get(id)?.msg(Msg::BookmarkNotFound)?;
        let title = if current.title == current.url {
            title_or_url(Some(s.title.clone()), &current.url)
        } else {
            current.title
        };
        let tags = if current.tags.is_empty() {
            normalize_tags(&s.tags)
        } else {
            current.tags
        };
        let summary = non_empty(&s.summary);
        let category = non_empty(&s.category);
        self.conn.execute(
            &format!(
                "UPDATE bookmarks SET title = ?1, tags = ?2, summary = ?3, category = ?4, {TOUCH}
                 WHERE id = ?5"
            ),
            params![title, join_tags(&tags), summary, category, id],
        )?;
        Ok(())
    }

    /// The next one to process: `pending` before `pending_embed`, and within each the most recently
    /// queued first (highest `updated_at` and `id`).
    pub fn next_pending(&self) -> Result<Option<Bookmark>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "SELECT {COLUMNS} FROM bookmarks WHERE status IN (?1, ?2)
                     ORDER BY (status = ?1) DESC, updated_at DESC, id DESC LIMIT 1"
                ),
                [STATUS_PENDING, STATUS_PENDING_EMBED],
                row_to_bookmark,
            )
            .optional()?)
    }

    pub fn categories(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT category FROM bookmarks WHERE category IS NOT NULL ORDER BY category",
        )?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn tags(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT tags FROM bookmarks")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut all = BTreeSet::new();
        for row in rows {
            all.extend(split_tags(&row?));
        }
        Ok(all.into_iter().collect())
    }

    pub fn count_unsummarized(&self) -> Result<usize> {
        self.count(&format!("status = ?1 AND {UNSUMMARIZED}"), [STATUS_READY])
    }

    pub fn stats(&self) -> Result<Stats> {
        Ok(Stats {
            total: self.count("TRUE", [])?,
            pending: self.count("status IN (?1, ?2)", [STATUS_PENDING, STATUS_PENDING_EMBED])?, // i18n-allow: sql
            failed: self.count("status = ?1", [STATUS_FAILED])?,
            unsummarized: self.count_unsummarized()?,
            unextracted: self.count(&format!("status = ?1 AND {NO_CONTENT}"), [STATUS_PENDING])?,
            imported: self.count("status = ?1", [STATUS_IMPORTED])?,
        })
    }

    /// Requeues `ready` bookmarks without a summary for summarizing. Returns how many moved.
    pub fn mark_unsummarized_pending(&self) -> Result<usize> {
        self.transition(UNSUMMARIZED, Status::Ready, Status::Pending)
    }

    /// Requeues every `ready` bookmark to recompute embeddings. Returns how many moved.
    pub fn mark_ready_pending_embed(&self) -> Result<usize> {
        self.transition("TRUE", Status::Ready, Status::PendingEmbed)
    }

    fn count(&self, filter: &str, params: impl Params) -> Result<usize> {
        let n: i64 = self.conn.query_row(
            &format!("SELECT COUNT(*) FROM bookmarks WHERE {filter}"),
            params,
            |r| r.get(0),
        )?;
        Ok(n as usize)
    }

    fn transition(&self, filter: &str, from: Status, to: Status) -> Result<usize> {
        Ok(self.conn.execute(
            &format!("UPDATE bookmarks SET status = ?2, {TOUCH} WHERE status = ?1 AND {filter}"),
            [from.as_str(), to.as_str()],
        )?)
    }

    pub fn get(&self, id: i64) -> Result<Option<Bookmark>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM bookmarks WHERE id = ?1"),
                [id],
                row_to_bookmark,
            )
            .optional()?)
    }

    pub fn find_by_url(&self, normalized_url: &str) -> Result<Option<Bookmark>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM bookmarks WHERE url = ?1"),
                [normalized_url],
                row_to_bookmark,
            )
            .optional()?)
    }

    pub fn list(&self) -> Result<Vec<Bookmark>> {
        self.list_columns(COLUMNS)
    }

    /// Like `list` but with `content` as `None`, to avoid moving all the Markdown over IPC.
    pub fn list_without_content(&self) -> Result<Vec<Bookmark>> {
        self.list_columns(COLUMNS_LIGHT)
    }

    fn list_columns(&self, columns: &str) -> Result<Vec<Bookmark>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {columns} FROM bookmarks ORDER BY created_at DESC, id DESC"
        ))?;
        let rows = stmt.query_map([], row_to_bookmark)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Full-text search. Each term is quoted so FTS5 syntax does not break on inputs like `c++`;
    /// terms are combined with AND.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Bookmark>> {
        let fts_query = query
            .split_whitespace()
            .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" ");
        if fts_query.is_empty() {
            return Ok(vec![]);
        }
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {}, {} FROM bookmarks_fts f JOIN bookmarks b ON b.id = f.rowid
             WHERE bookmarks_fts MATCH ?1
             ORDER BY bm25(bookmarks_fts, 10.0, 5.0, 5.0, 5.0, 2.0, 2.0, 2.0, 1.0) LIMIT ?2",
            columns_head!()
                .split(", ")
                .map(|c| format!("b.{c}"))
                .collect::<Vec<_>>()
                .join(", "),
            original_url!("b.")
        ))?;
        let rows = stmt.query_map(params![fts_query, limit as i64], row_to_bookmark)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Every bookmark, whatever its status, oldest first.
    pub fn export_rows(&self) -> Result<Vec<ExportRow>> {
        let mut stmt = self.conn.prepare(concat!(
            "SELECT ",
            original_url!(""),
            ", title, tags, CAST(strftime('%s', created_at) AS INTEGER) FROM bookmarks
             ORDER BY created_at, id"
        ))?;
        let rows = stmt.query_map([], |r| {
            Ok(ExportRow {
                url: r.get(0)?,
                title: r.get(1)?,
                tags: split_tags(&r.get::<_, String>(2)?),
                add_date: r.get(3)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Writes one `<id>.md` with front matter per bookmark. Returns how many it wrote.
    pub fn export_markdown(&self, dir: &Path, notes_heading: &str) -> Result<usize> {
        std::fs::create_dir_all(dir).msg(Msg::ExportWrite)?;
        let all = self.list()?;
        for b in &all {
            std::fs::write(
                dir.join(format!("{}.md", b.id)),
                b.to_markdown(notes_heading),
            )
            .msg(Msg::ExportWrite)?;
        }
        Ok(all.len())
    }
}

impl Bookmark {
    pub fn to_markdown(&self, notes_heading: &str) -> String {
        let quote = |s: &str| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
        // One line per key: a `---` at column 0 would close the front matter.
        let one_line = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut out = String::from("---\n");
        out.push_str(&format!("title: {}\n", quote(&one_line(&self.title))));
        out.push_str(&format!("url: {}\n", self.url));
        if let Some(c) = &self.category {
            out.push_str(&format!("category: {}\n", quote(c)));
        }
        if !self.tags.is_empty() {
            let tags: Vec<String> = self.tags.iter().map(|t| quote(t)).collect();
            out.push_str(&format!("tags: [{}]\n", tags.join(", ")));
        }
        let meta = [
            ("author", &self.meta.author),
            ("site", &self.meta.site),
            ("published", &self.meta.published),
            ("language", &self.meta.language),
            ("description", &self.meta.description),
        ];
        for (key, value) in meta {
            if let Some(v) = value {
                out.push_str(&format!("{key}: {}\n", quote(&one_line(v))));
            }
        }
        out.push_str(&format!("created: {}\n---\n\n", self.created_at));
        if !self.notes.is_empty() {
            out.push_str(&format!("{notes_heading}\n\n{}\n\n", self.notes));
        }
        if let Some(s) = &self.summary {
            out.push_str(s);
            out.push_str("\n\n");
        }
        if let Some(c) = &self.content {
            out.push_str(c);
            out.push('\n');
        }
        out
    }
}

/// Trims, lowercases, turns commas into spaces, collapses spaces and drops empty and duplicate tags (first occurrence order).
pub fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    tags.iter()
        .map(|t| {
            t.replace(TAG_SEPARATOR, " ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        })
        .filter(|t| !t.is_empty() && seen.insert(t.clone()))
        .collect()
}

fn join_tags(tags: &[String]) -> String {
    tags.join(&TAG_SEPARATOR.to_string())
}

fn split_tags(stored: &str) -> Vec<String> {
    stored
        .split(TAG_SEPARATOR)
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect()
}

/// Trimmed text, or `None` if empty (stored as NULL).
fn non_empty(text: &str) -> Option<&str> {
    Some(text.trim()).filter(|t| !t.is_empty())
}

fn image_columns(image: &Option<Image>) -> (Option<&[u8]>, Option<&str>) {
    match image {
        Some(i) => (Some(&i.bytes), Some(&i.mime)),
        None => (None, None),
    }
}

/// A stored image only counts if it has both bytes and type.
fn stored_image(bytes: Option<Vec<u8>>, mime: Option<String>) -> Option<Image> {
    Some(Image {
        bytes: bytes?,
        mime: mime?,
    })
}

/// A missing or empty title stands for the URL.
fn title_or_url(title: Option<String>, url: &str) -> String {
    title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| url.to_string())
}

fn row_to_bookmark(row: &rusqlite::Row) -> rusqlite::Result<Bookmark> {
    let tags: String = row.get(5)?;
    let status: String = row.get(8)?;
    let error_kind: Option<String> = row.get(22)?;
    Ok(Bookmark {
        id: row.get(0)?,
        url: row.get(1)?,
        title: row.get(2)?,
        summary: row.get(3)?,
        category: row.get(4)?,
        tags: split_tags(&tags),
        content: row.get(6)?,
        notes: row.get(7)?,
        status: status.parse().map_err(|e: anyhow::Error| {
            rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, e.into())
        })?,
        error: row
            .get::<_, Option<String>>(9)?
            .map(|json| BookmarkError::from_json(&json)),
        meta: PageMeta {
            description: row.get(13)?,
            author: row.get(14)?,
            site: row.get(15)?,
            published: row.get(16)?,
            language: row.get(17)?,
            image: row.get(18)?,
            favicon: row.get(19)?,
            word_count: row.get(20)?,
            extractor: row.get(21)?,
        },
        error_kind: error_kind
            .map(|k| k.parse())
            .transpose()
            .map_err(|e: anyhow::Error| {
                rusqlite::Error::FromSqlConversionFailure(22, rusqlite::types::Type::Text, e.into())
            })?,
        embedding_id: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
        favicon_at: row.get(23)?,
        original_url: row.get(24)?,
    })
}

/// Registers sqlite-vec for every connection in the process; must run before the first one opens.
fn register_vec_extension() {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        // SAFETY: `sqlite3_vec_init` has the extension entry-point signature SQLite expects.
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<
                *const (),
                unsafe extern "C" fn(
                    *mut rusqlite::ffi::sqlite3,
                    *mut *mut std::os::raw::c_char,
                    *const rusqlite::ffi::sqlite3_api_routines,
                ) -> std::os::raw::c_int,
            >(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        }
    });
}

fn user_version(conn: &Connection) -> Result<usize> {
    let version: isize = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    Ok(version as usize)
}

fn apply(tx: &Transaction, migration: &Migration) -> Result<()> {
    match migration {
        Migration::Sql(sql) => Ok(tx.execute_batch(sql)?),
        Migration::Rust(run) => run(tx),
    }
}

/// `user_version`, `meta.min_reader` (if any) and whether `sqlite_master` is empty.
fn read_state(conn: &Connection) -> Result<(usize, Option<usize>, bool)> {
    let version = user_version(conn)?;
    let empty: bool =
        conn.query_row("SELECT NOT EXISTS (SELECT 1 FROM sqlite_master)", [], |r| {
            r.get(0)
        })?;
    let has_meta: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'meta')",
        [],
        |r| r.get(0),
    )?;
    let min_reader = if has_meta {
        conn.query_row(
            "SELECT value FROM meta WHERE key = ?1",
            [META_MIN_READER],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .and_then(|v| v.parse().ok())
    } else {
        None
    };
    Ok((version, min_reader, empty))
}

/// Refuses, without writing anything, a database this app cannot open. Returns its version.
/// The errors carry no path: the connection does not know it (`with_path` adds it).
fn check(conn: &Connection, migrations: &[(Compat, Migration)]) -> Result<usize> {
    let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
    let (version, min_reader, empty) = read_state(&tx)?;
    drop(tx);
    match verdict(version, min_reader, empty, migrations.len()) {
        Verdict::Open => Ok(version),
        Verdict::TooNew => Err(Msg::DatabaseTooNew {
            path: String::new(),
        }
        .err()),
        Verdict::Unknown => Err(Msg::DatabaseUnknown {
            path: String::new(),
        }
        .err()),
    }
}

/// Folder, next to the database, where `backup_before_migrating` leaves its copies.
const BACKUP_DIR: &str = "backups";
/// Copies kept per database (per file stem); older ones are deleted.
const BACKUPS_KEPT: usize = 3;
/// UTC time in the copy's name, from SQLite: no extra dependency, and names sort by date.
const BACKUP_STAMP: &str = "SELECT strftime('%Y%m%dT%H%M%SZ', 'now')";
/// Writes a consistent copy of the database to the file `?1`.
const VACUUM_INTO: &str = "VACUUM INTO ?1"; // i18n-allow: sql

/// Copies `conn` to `<dir>/backups/<stem>-v<from>-<UTC>.db` and prunes to the newest
/// `BACKUPS_KEPT` of that stem. `VACUUM INTO` includes what is committed in the WAL. If the
/// copy cannot be made or pruned, the error stops the opening: nothing is migrated.
fn backup_before_migrating(conn: &Connection, db_path: &Path, from: usize) -> Result<PathBuf> {
    let dir = db_path.parent().unwrap_or(Path::new(".")).join(BACKUP_DIR);
    let stem = db_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stamp: String = conn.query_row(BACKUP_STAMP, [], |r| r.get(0))?;
    let target = dir.join(format!("{stem}-v{from}-{stamp}.db"));
    let failed = || Msg::Backup {
        path: target.display().to_string(),
    };
    std::fs::create_dir_all(&dir).msg(failed())?;
    // Two openings in the same second: the copy is already there. Only a file counts: anything
    // else under that name must not pass for a copy.
    if !target.is_file() {
        write_copy(conn, &target).msg(failed())?;
    }
    prune_backups(&dir, &stem).msg(failed())?;
    Ok(target)
}

/// Suffix of the file a copy is written to before it takes its name; pruning never matches it.
const PARTIAL_SUFFIX: &str = "partial";
/// What SQLite leaves next to a `VACUUM INTO` output that failed half-way.
const JOURNAL_SUFFIX: &str = "-journal";

/// Writes a consistent copy of `conn` to `target`, through a temporary name in the same folder:
/// a copy cut short (a full disk) is deleted instead of left under a name pruning would keep,
/// and a process copying the same second does not make this one fail (the rename replaces it).
fn write_copy(conn: &Connection, target: &Path) -> Result<()> {
    let mut name = target.as_os_str().to_owned();
    name.push(format!(".{}.{PARTIAL_SUFFIX}", std::process::id()));
    let partial = PathBuf::from(name);
    let mut journal = partial.as_os_str().to_owned();
    journal.push(JOURNAL_SUFFIX);
    let remove_partial = || {
        let _ = std::fs::remove_file(&partial);
        let _ = std::fs::remove_file(&journal);
    };
    remove_partial();
    let written = conn
        .execute(VACUUM_INTO, [partial.to_string_lossy()])
        .map_err(anyhow::Error::from)
        .and_then(|_| std::fs::rename(&partial, target).map_err(anyhow::Error::from));
    if written.is_err() {
        remove_partial();
    }
    written
}

/// Deletes all but the newest `BACKUPS_KEPT` files named `<stem>-v<n>-<UTC>.db` in `dir`.
fn prune_backups(dir: &Path, stem: &str) -> std::io::Result<()> {
    let prefix = format!("{stem}-v");
    let mut copies: Vec<(String, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let stamp = name
            .strip_prefix(&prefix)
            .and_then(|rest| rest.strip_suffix(".db"))
            .and_then(|rest| rest.split_once('-'))
            .filter(|(version, _)| {
                !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit())
            })
            .map(|(_, stamp)| stamp.to_string());
        if let Some(stamp) = stamp {
            copies.push((stamp, entry.path()));
        }
    }
    // By date, not by name: `v10` sorts before `v2`.
    copies.sort();
    let excess = copies.len().saturating_sub(BACKUPS_KEPT);
    for (_, path) in &copies[..excess] {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Names the file in the errors of `check`.
fn with_path(err: anyhow::Error, path: &Path) -> anyhow::Error {
    let path = path.display().to_string();
    match failure_msg(&err) {
        Some(Msg::DatabaseTooNew { .. }) => Msg::DatabaseTooNew { path }.err(),
        Some(Msg::DatabaseUnknown { .. }) => Msg::DatabaseUnknown { path }.err(),
        _ if is_unreadable(&err) => Msg::DatabaseUnreadable { path }.err(),
        _ => err,
    }
}

/// `Connection::open` does not read the file: a non-SQLite or damaged one fails on first access.
fn is_unreadable(err: &anyhow::Error) -> bool {
    use rusqlite::ErrorCode::{DatabaseCorrupt, NotADatabase};
    err.chain()
        .filter_map(|e| e.downcast_ref::<rusqlite::Error>())
        .any(|e| matches!(e.sqlite_error_code(), Some(NotADatabase | DatabaseCorrupt)))
}

/// Applies pending migrations, each in its own transaction together with `user_version` and
/// `meta.min_reader`. A database newer than `migrations` is left as it is.
/// The transaction is immediate and rereads the version inside: if the CLI and the app migrate at
/// once, the loser sees the version already advanced and does not repeat the migration.
fn migrate(conn: &Connection, migrations: &[(Compat, Migration)]) -> Result<()> {
    for (index, (_, migration)) in migrations.iter().enumerate() {
        let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
        if user_version(&tx)? > index {
            continue;
        }
        apply(&tx, migration)?;
        tx.execute_batch(&format!("PRAGMA user_version = {}", index + 1))?;
        tx.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            params![
                META_MIN_READER,
                min_reader(migrations, index + 1).to_string()
            ],
        )?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with(url: &str, title: &str, content: &str) -> Store {
        let s = Store::open_in_memory().unwrap();
        s.add(new_bookmark(url, title, Some(content))).unwrap();
        s
    }

    fn new_bookmark(url: &str, title: &str, content: Option<&str>) -> NewBookmark {
        NewBookmark {
            url: url.into(),
            title: Some(title.into()),
            notes: String::new(),
            tags: vec![],
            content: content.map(Into::into),
            ..Default::default()
        }
    }

    #[test]
    fn imported_status_round_trips() {
        let status = "imported".parse::<Status>().unwrap();
        assert_eq!(status, Status::Imported);
        assert_eq!(status.as_str(), "imported");
    }

    /// One line per schema object, sorted by `(type, name)`, without `meta` or SQLite's own.
    /// Tables are described by their pragmas, not their `sql`: `ALTER TABLE ADD COLUMN` rewrites
    /// that text. Columns go without `cid`, so dropping one changes a single line. Each table also
    /// lists its indexes, automatic ones included (they carry `UNIQUE` and the primary key).
    fn describe_schema(conn: &Connection) -> String {
        let squash = |sql: &str| sql.split_whitespace().collect::<Vec<_>>().join(" ");
        let objects: Vec<(String, String, String)> = conn
            .prepare(
                "SELECT type, name, COALESCE(sql, '') FROM sqlite_master
                 WHERE name != 'meta' AND name NOT LIKE 'sqlite_%' AND type != 'index'
                 ORDER BY type, name",
            )
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        let mut out = Vec::new();
        for (kind, name, sql) in objects {
            if kind != "table" || sql.starts_with("CREATE VIRTUAL TABLE") {
                out.push(format!("{kind} {name} {}", squash(&sql)));
                continue;
            }
            out.push(format!("table {name}"));
            let rows = |pragma: &str, f: &dyn Fn(&rusqlite::Row) -> rusqlite::Result<String>| {
                conn.prepare(&format!("SELECT * FROM pragma_{pragma}('{name}')"))
                    .unwrap()
                    .query_map([], |r| f(r))
                    .unwrap()
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .unwrap()
            };
            out.extend(rows("table_xinfo", &|r| {
                Ok(format!(
                    "col {name}.{} {} notnull={} dflt={} pk={} hidden={}",
                    r.get::<_, String>("name")?,
                    r.get::<_, String>("type")?,
                    r.get::<_, i64>("notnull")?,
                    r.get::<_, Option<String>>("dflt_value")?
                        .as_deref()
                        .unwrap_or("NULL"),
                    r.get::<_, i64>("pk")?,
                    r.get::<_, i64>("hidden")?,
                ))
            }));
            out.extend(rows("foreign_key_list", &|r| {
                Ok(format!(
                    "fk {name}.{} -> {}.{} on_delete={}",
                    r.get::<_, String>("from")?,
                    r.get::<_, String>("table")?,
                    r.get::<_, String>("to")?,
                    r.get::<_, String>("on_delete")?,
                ))
            }));
            let mut indexes = rows("index_list", &|r| {
                Ok(format!(
                    "{}\t{}\t{}",
                    r.get::<_, String>("name")?,
                    r.get::<_, i64>("unique")?,
                    r.get::<_, String>("origin")?,
                ))
            });
            indexes.sort();
            for index in indexes {
                let [index, unique, origin] = index.split('\t').collect::<Vec<_>>()[..] else {
                    unreachable!()
                };
                out.push(format!(
                    "index {index} on {name} unique={unique} origin={origin}"
                ));
                let mut keys = conn
                    .prepare(&format!(
                        "SELECT * FROM pragma_index_xinfo('{index}') WHERE key = 1"
                    ))
                    .unwrap();
                let keys = keys
                    .query_map([], |r| {
                        Ok(format!(
                            "key {index}.{} desc={} coll={}",
                            r.get::<_, Option<String>>("name")?
                                .as_deref()
                                .unwrap_or("NULL"),
                            r.get::<_, i64>("desc")?,
                            r.get::<_, String>("coll")?,
                        ))
                    })
                    .unwrap()
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .unwrap();
                out.extend(keys);
            }
        }
        out.join("\n") + "\n"
    }

    /// The v0.7 schema, written by `describe_schema` before migrations 1 to 9 were merged. Never edited.
    const REFERENCE: &str = include_str!("../fixtures/schema-initial.txt");

    /// Lines of the v0.7 reference that v1.0 drops on purpose: one per removed column.
    const REMOVED_IN_V1_0: &[&str] =
        &["col bookmarks.error TEXT notnull=0 dflt=NULL pk=0 hidden=0"];

    #[test]
    fn initial_schema_matches_reference() {
        for line in REMOVED_IN_V1_0 {
            assert_eq!(REFERENCE.lines().filter(|l| l == line).count(), 1, "{line}");
        }
        let conn = Connection::open_in_memory().unwrap();
        at_version(&conn, 1);
        let expected: Vec<&str> = REFERENCE
            .lines()
            .filter(|l| !REMOVED_IN_V1_0.contains(l))
            .collect();
        assert_eq!(describe_schema(&conn).lines().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn initial_schema_has_no_error_column() {
        let conn = Connection::open_in_memory().unwrap();
        at_version(&conn, 1);
        let columns: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_xinfo('bookmarks')")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert!(!columns.iter().any(|c| c == "error"));
        assert!(columns.iter().any(|c| c == "error_msg"));
        assert!(columns.iter().any(|c| c == "error_kind"));
    }

    #[test]
    fn initial_schema_creates_meta() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA_1).unwrap();
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM meta", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 0);
    }

    /// Applies the first `n` migrations, as an older app version did.
    fn at_version(conn: &Connection, n: usize) {
        for (i, (_, m)) in MIGRATIONS[..n].iter().enumerate() {
            let tx = conn.unchecked_transaction().unwrap();
            apply(&tx, m).unwrap();
            tx.execute_batch(&format!("PRAGMA user_version = {}", i + 1))
                .unwrap();
            tx.execute(
                "INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)",
                params![META_MIN_READER, min_reader(MIGRATIONS, i + 1).to_string()],
            )
            .unwrap();
            tx.commit().unwrap();
        }
    }

    fn raw(s: &str) -> ErrorPart {
        ErrorPart::Raw { raw: s.into() }
    }

    #[test]
    fn list_survives_an_unknown_error_code() {
        let s = store_with("https://a.com", "A", "x");
        s.conn
            .execute_batch(r#"UPDATE bookmarks SET error_msg = '[{"code":"zzz"}]'"#)
            .unwrap();
        let rows = s.list().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].error,
            Some(BookmarkError(vec![raw(r#"{"code":"zzz"}"#)]))
        );
    }

    #[test]
    fn set_failed_round_trips() {
        let s = store_with("https://a.com", "A", "x");
        let err = BookmarkError(vec![
            ErrorPart::Msg(Msg::OllamaStatus {
                status: 400,
                detail: "x".into(),
            }),
            raw("y"),
        ]);
        s.set_failed(1, &err).unwrap();
        let b = s.get(1).unwrap().unwrap();
        assert_eq!(
            (b.status, b.error, b.error_kind),
            (Status::Failed, Some(err), None)
        );
    }

    #[test]
    fn migrations_run_in_order() {
        assert_eq!(MIGRATIONS.len(), 1);
        let fresh = Store::open_in_memory().unwrap();
        assert_eq!(store_version(&fresh), 1);
        // If migration 1 ran again on a v1 database, its `CREATE TABLE` would fail.
        let conn = Connection::open_in_memory().unwrap();
        at_version(&conn, 1);
        register_vec_extension();
        let s = Store::init(conn).unwrap();
        assert_eq!(store_version(&s), 1);
    }

    #[test]
    fn add_keeps_original_url_and_dedupes_by_normalized() {
        let s = Store::open_in_memory().unwrap();
        let raw = "https://A.com/x?utm_source=t#top";
        let first = s.add(new_bookmark(raw, "A", None)).unwrap();
        assert_eq!(first.url, "https://a.com/x");
        assert_eq!(first.original_url, raw);
        let again = s.add(new_bookmark("https://a.com/x", "B", None)).unwrap();
        assert_eq!((again.id, again.original_url), (first.id, raw.to_string()));
    }

    #[test]
    fn insert_imported_sets_status_and_date() {
        let s = Store::open_in_memory().unwrap();
        let id = s
            .insert_imported(
                &new_bookmark("https://a.com", "A", None),
                Some(1_700_000_000),
            )
            .unwrap();
        let b = s.get(id).unwrap().unwrap();
        assert_eq!(b.status, Status::Imported);
        assert_eq!(b.created_at, "2023-11-14T22:13:20Z");
        let id = s
            .insert_imported(&new_bookmark("https://b.com", "B", None), None)
            .unwrap();
        let today = s
            .conn
            .query_row("SELECT strftime('%Y-%m-%d', 'now')", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap();
        assert!(s.get(id).unwrap().unwrap().created_at.starts_with(&today));
    }

    #[test]
    fn add_on_imported_queues_it() {
        let s = Store::open_in_memory().unwrap();
        let nb = new_bookmark("https://a.com", "A", None);
        s.insert_imported(&nb, None).unwrap();
        assert_eq!(s.add(nb).unwrap().status, Status::Pending);
    }

    #[test]
    fn queue_imported_only_moves_imported() {
        let s = Store::open_in_memory().unwrap();
        let imported = s
            .insert_imported(&new_bookmark("https://a.com", "A", None), None)
            .unwrap();
        let failed = s.add(new_bookmark("https://b.com", "B", None)).unwrap();
        let ready = s.add(new_bookmark("https://c.com", "C", None)).unwrap();
        s.set_failed(failed.id, &BookmarkError(vec![raw("boom")]))
            .unwrap();
        s.set_status(ready.id, Status::Ready, None).unwrap();
        assert_eq!(
            s.queue_imported(&[imported, failed.id, ready.id]).unwrap(),
            1
        );
        let status = |id| s.get(id).unwrap().unwrap().status;
        assert_eq!(
            (status(imported), status(failed.id), status(ready.id)),
            (Status::Pending, Status::Failed, Status::Ready)
        );
    }

    #[test]
    fn stats_count_imported_apart() {
        let s = Store::open_in_memory().unwrap();
        s.insert_imported(&new_bookmark("https://a.com", "A", None), None)
            .unwrap();
        let stats = s.stats().unwrap();
        assert_eq!(
            (stats.imported, stats.pending, stats.unextracted),
            (1, 0, 0)
        );
    }

    #[test]
    fn queues_skip_imported() {
        let s = Store::open_in_memory().unwrap();
        let mut nb = new_bookmark("https://a.com", "A", None);
        nb.meta.favicon = Some("https://a.com/f.ico".into());
        s.insert_imported(&nb, None).unwrap();
        assert!(s.next_pending().unwrap().is_none());
        assert!(s.next_without_content().unwrap().is_none());
        assert!(s.next_without_favicon().unwrap().is_none());
        assert_eq!(s.mark_unsummarized_pending().unwrap(), 0);
        assert_eq!(s.mark_ready_pending_embed().unwrap(), 0);
    }

    #[test]
    fn add_dedupes_by_normalized_url() {
        let s = store_with("https://a.com/x?utm_source=t", "A", "texto");
        let again = s
            .add(new_bookmark("https://a.com/x#frag", "otro", None))
            .unwrap();
        assert_eq!(again.title, "A");
        assert_eq!(s.list().unwrap().len(), 1);
    }

    #[test]
    fn search_matches_content_and_tolerates_fts_syntax() {
        let s = store_with(
            "https://a.com/rust",
            "Guía de Rust",
            "ownership y borrowing en c++ vs rust",
        );
        s.add(new_bookmark("https://b.com", "Cocina", Some("paella")))
            .unwrap();
        assert_eq!(s.search("borrowing", 10).unwrap()[0].title, "Guía de Rust");
        assert_eq!(s.search("c++ rust", 10).unwrap().len(), 1);
        assert!(s.search("paella rust", 10).unwrap().is_empty());
        assert!(s.search("   ", 10).unwrap().is_empty());
    }

    #[test]
    fn export_writes_front_matter() {
        let s = store_with("https://a.com", "Título \"raro\"", "cuerpo");
        let dir = std::env::temp_dir().join(format!("marks-test-{}", std::process::id()));
        assert_eq!(s.export_markdown(&dir, "## Notas").unwrap(), 1);
        let md = std::fs::read_to_string(dir.join("1.md")).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(md.starts_with("---\ntitle: \"Título \\\"raro\\\"\"\nurl: https://a.com/\n"));
        assert!(md.ends_with("---\n\ncuerpo\n"));
    }

    #[test]
    fn export_names_a_failed_write() {
        let s = store_with("https://a.com", "A", "x");
        // A file where the folder should go: `create_dir_all` fails.
        let file = std::env::temp_dir().join(format!("marks-export-file-{}", std::process::id()));
        std::fs::write(&file, "").unwrap();
        let err = s.export_markdown(&file, "## Notas").unwrap_err();
        std::fs::remove_file(&file).unwrap();
        assert_eq!(crate::msg::failure_msg(&err), Some(&Msg::ExportWrite));
    }

    #[test]
    fn open_names_a_folder_it_cannot_create() {
        // A file where the database folder should go: `create_dir_all` fails.
        let file = std::env::temp_dir().join(format!("marks-open-file-{}", std::process::id()));
        std::fs::write(&file, "").unwrap();
        let path = file.join("marks.db");
        let err = Store::open(&path).err().unwrap();
        std::fs::remove_file(&file).unwrap();
        assert_eq!(
            crate::msg::failure_msg(&err),
            Some(&Msg::OpenDatabase {
                path: path.display().to_string()
            })
        );
    }

    fn store_version(s: &Store) -> usize {
        user_version(&s.conn).unwrap()
    }

    const TEST_V2: &str = "ALTER TABLE bookmarks ADD COLUMN test_extra TEXT;";
    const ADDITIVE_V2: &[(Compat, Migration)] = &[
        (Compat::Breaking, Migration::Sql(SCHEMA_1)),
        (Compat::Additive, Migration::Sql(TEST_V2)),
    ];
    const BREAKING_V2: &[(Compat, Migration)] = &[
        (Compat::Breaking, Migration::Sql(SCHEMA_1)),
        (Compat::Breaking, Migration::Sql(TEST_V2)),
    ];

    fn meta_min_reader(conn: &Connection) -> Option<String> {
        conn.query_row(
            "SELECT value FROM meta WHERE key = ?1",
            [META_MIN_READER],
            |r| r.get(0),
        )
        .optional()
        .unwrap()
    }

    fn has_table(conn: &Connection, name: &str) -> bool {
        conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            [name],
            |r| r.get(0),
        )
        .unwrap()
    }

    /// A database file made only of `sql`, replacing whatever was at `path`.
    fn conn_file(path: &Path, sql: &str) {
        let _ = std::fs::remove_file(path);
        Connection::open(path).unwrap().execute_batch(sql).unwrap();
    }

    /// A v1 in-memory connection, as this app leaves it, with one bookmark.
    fn v1_with_a_row(migrations: &[(Compat, Migration)]) -> Connection {
        register_vec_extension();
        let s = Store::init_with(Connection::open_in_memory().unwrap(), migrations).unwrap();
        s.add(new_bookmark("https://a.com", "A", None)).unwrap();
        s.conn
    }

    /// A fresh folder with a v1 database file (`MIGRATIONS`) holding one bookmark.
    fn v1_file(name: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("marks-backup-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("marks.db");
        let s = Store::open(&path).unwrap();
        s.add(new_bookmark("https://a.com", "A", None)).unwrap();
        (dir, path)
    }

    fn file_names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    #[test]
    fn backs_up_before_migrating() {
        let (dir, path) = v1_file("migrate");
        let s = Store::open_with(&path, ADDITIVE_V2).unwrap();
        assert_eq!(store_version(&s), 2);
        let names = file_names(&dir.join(BACKUP_DIR));
        assert_eq!(names.len(), 1);
        assert!(
            names[0].starts_with("marks-v1-") && names[0].ends_with(".db"),
            "{names:?}"
        );
        let copy = Connection::open(dir.join(BACKUP_DIR).join(&names[0])).unwrap();
        assert_eq!(user_version(&copy).unwrap(), 1);
        let rows: i64 = copy
            .query_row("SELECT COUNT(*) FROM bookmarks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 1);
        drop((s, copy));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn no_backup_for_new_or_current_database() {
        let (dir, path) = v1_file("current");
        Store::open_with(&path, MIGRATIONS).unwrap();
        Store::open_with(&dir.join("new.db"), ADDITIVE_V2).unwrap();
        assert!(!dir.join(BACKUP_DIR).exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn no_backup_when_refused() {
        let (dir, path) = v1_file("refused");
        conn_file(
            &path,
            "CREATE TABLE bookmarks(id); PRAGMA user_version = 4;",
        );
        assert!(Store::open_with(&path, ADDITIVE_V2).is_err());
        let newer = dir.join("newer.db");
        Store::open_with(&newer, BREAKING_V2).unwrap();
        assert!(Store::open_with(&newer, MIGRATIONS).is_err());
        assert!(!dir.join(BACKUP_DIR).exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn keeps_three_newest_backups() {
        let (dir, path) = v1_file("prune");
        let backups = dir.join(BACKUP_DIR);
        std::fs::create_dir_all(&backups).unwrap();
        let old = [
            "marks-v1-20260101T000000Z.db",
            "marks-v1-20260102T000000Z.db",
            "marks-v1-20260103T000000Z.db",
            "marks-v1-20260104T000000Z.db",
            "otro-v1-20260101T000000Z.db",
        ];
        for name in old {
            std::fs::write(backups.join(name), "").unwrap();
        }
        let s = Store::open_with(&path, ADDITIVE_V2).unwrap();
        let names = file_names(&backups);
        let new: Vec<&String> = names
            .iter()
            .filter(|n| !old.contains(&n.as_str()))
            .collect();
        assert_eq!(new.len(), 1, "{names:?}");
        let mut expected = vec![
            "marks-v1-20260103T000000Z.db".to_string(),
            "marks-v1-20260104T000000Z.db".to_string(),
            new[0].clone(),
            "otro-v1-20260101T000000Z.db".to_string(),
        ];
        expected.sort();
        assert_eq!(names, expected);
        drop(s);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn copy_accepts_a_copy_made_meanwhile() {
        // Another process wrote the same copy after this one saw none (same second).
        let (dir, path) = v1_file("race");
        let target = dir.join("marks-v1-20260101T000000Z.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute(VACUUM_INTO, [target.to_string_lossy()])
            .unwrap();
        write_copy(&conn, &target).unwrap();
        assert_eq!(
            user_version(&Connection::open(&target).unwrap()).unwrap(),
            1
        );
        drop(conn);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_copy_leaves_no_partial_file() {
        // The copy is written, then cannot take its name: a folder is in the way.
        let (dir, path) = v1_file("partial");
        let target = dir.join("marks-v1-20260101T000000Z.db");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("keep"), "").unwrap();
        let conn = Connection::open(&path).unwrap();
        assert!(write_copy(&conn, &target).is_err());
        let names = file_names(&dir);
        drop(conn);
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(
            names.iter().all(|n| !n.contains(".partial")),
            "partial copy left: {names:?}"
        );
    }

    /// A 2 MB volume mounted with `hdiutil` (macOS, no root), detached on drop.
    #[cfg(target_os = "macos")]
    struct TinyVolume {
        dir: PathBuf,
        mount: PathBuf,
    }

    #[cfg(target_os = "macos")]
    impl TinyVolume {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("marks-full-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            let image = dir.join("v.dmg");
            let mount = dir.join("mnt");
            let run = |args: &[&std::ffi::OsStr]| {
                let ok = std::process::Command::new("hdiutil")
                    .args(args)
                    .status()
                    .unwrap()
                    .success();
                assert!(ok, "hdiutil {args:?}");
            };
            run(&[
                "create".as_ref(),
                "-quiet".as_ref(),
                "-size".as_ref(),
                "2m".as_ref(),
                "-fs".as_ref(),
                "HFS+".as_ref(),
                image.as_os_str(),
            ]);
            run(&[
                "attach".as_ref(),
                "-quiet".as_ref(),
                "-nobrowse".as_ref(),
                "-mountpoint".as_ref(),
                mount.as_os_str(),
                image.as_os_str(),
            ]);
            TinyVolume { dir, mount }
        }
    }

    #[cfg(target_os = "macos")]
    impl Drop for TinyVolume {
        fn drop(&mut self) {
            let _ = std::process::Command::new("hdiutil")
                .args([
                    "detach".as_ref(),
                    "-quiet".as_ref(),
                    "-force".as_ref(),
                    self.mount.as_os_str(),
                ])
                .status();
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    /// Needs `hdiutil` and mounts a volume: `cargo test -p marks-core disk_full -- --ignored`.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore]
    fn disk_full_copy_leaves_no_partial_file() {
        let (dir, path) = v1_file("full");
        let s = Store::open(&path).unwrap();
        // About 4 MB of content: more than the volume holds.
        let big = "x".repeat(4 * 1024 * 1024);
        s.add(new_bookmark("https://b.com", "B", Some(&big)))
            .unwrap();
        let volume = TinyVolume::new("copy");
        let target = volume.mount.join("marks-v1-20260101T000000Z.db");
        assert!(write_copy(&s.conn, &target).is_err());
        let left: Vec<String> = file_names(&volume.mount)
            .into_iter()
            .filter(|n| !n.starts_with('.'))
            .collect();
        drop((s, volume));
        std::fs::remove_dir_all(dir).unwrap();
        assert!(left.is_empty(), "left on the full volume: {left:?}");
    }

    #[test]
    fn failed_backup_leaves_database_unmigrated() {
        let (dir, path) = v1_file("failed");
        std::fs::write(dir.join(BACKUP_DIR), "not a folder").unwrap();
        let err = Store::open_with(&path, ADDITIVE_V2).err().unwrap();
        assert!(
            matches!(failure_msg(&err), Some(Msg::Backup { .. })),
            "{err:#}"
        );
        let conn = Connection::open(&path).unwrap();
        let columns: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_xinfo('bookmarks')")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(user_version(&conn).unwrap(), 1);
        assert!(!columns.iter().any(|c| c == "test_extra"));
        drop(conn);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn min_reader_is_last_breaking_migration() {
        let list = [
            (Compat::Breaking, Migration::Sql("")),
            (Compat::Additive, Migration::Sql("")),
            (Compat::Breaking, Migration::Sql("")),
            (Compat::Additive, Migration::Sql("")),
        ];
        let got: Vec<usize> = (0..=4).map(|v| min_reader(&list, v)).collect();
        assert_eq!(got, vec![0, 1, 1, 3, 3]);
    }

    #[test]
    fn fresh_database_has_min_reader_1() {
        let s = Store::open_in_memory().unwrap();
        assert_eq!(store_version(&s), 1);
        assert_eq!(meta_min_reader(&s.conn).as_deref(), Some("1"));
    }

    #[test]
    fn verdict_cases() {
        assert_eq!(verdict(0, None, true, 1), Verdict::Open);
        assert_eq!(verdict(0, None, false, 1), Verdict::Unknown);
        assert_eq!(verdict(4, None, false, 1), Verdict::Unknown);
        assert_eq!(verdict(1, Some(1), false, 1), Verdict::Open);
        assert_eq!(verdict(2, Some(1), false, 1), Verdict::Open);
        assert_eq!(verdict(2, Some(2), false, 1), Verdict::TooNew);
    }

    #[test]
    fn migrating_to_v2_updates_min_reader() {
        for (list, expected) in [(BREAKING_V2, "2"), (ADDITIVE_V2, "1")] {
            let conn = v1_with_a_row(MIGRATIONS);
            let s = Store::init_with(conn, list).unwrap();
            assert_eq!(store_version(&s), 2);
            assert_eq!(meta_min_reader(&s.conn).as_deref(), Some(expected));
        }
    }

    #[test]
    fn opens_newer_additive_database_untouched() {
        let conn = v1_with_a_row(ADDITIVE_V2);
        let s = Store::init_with(conn, MIGRATIONS).unwrap();
        assert_eq!(s.get(1).unwrap().unwrap().title, "A");
        assert_eq!(store_version(&s), 2);
        assert_eq!(meta_min_reader(&s.conn).as_deref(), Some("1"));
    }

    #[test]
    fn refuses_database_too_new() {
        let conn = v1_with_a_row(BREAKING_V2);
        let err = Store::init_with(conn, MIGRATIONS).err().unwrap();
        assert!(matches!(
            failure_msg(&err),
            Some(Msg::DatabaseTooNew { .. })
        ));
    }

    #[test]
    fn refuses_unknown_database() {
        // A 0.x build at version 4: no `meta`.
        let path = std::env::temp_dir().join(format!("marks-unknown-{}.db", std::process::id()));
        conn_file(
            &path,
            "CREATE TABLE bookmarks(id); PRAGMA user_version = 4;",
        );
        let before = std::fs::read(&path).unwrap();
        let err = Store::open(&path).err().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), before, "the file changed");
        let conn = Connection::open(&path).unwrap();
        let (version, meta) = (user_version(&conn).unwrap(), has_table(&conn, "meta"));
        drop(conn);
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            failure_msg(&err),
            Some(&Msg::DatabaseUnknown {
                path: path.display().to_string()
            })
        );
        assert_eq!((version, meta), (4, false));

        // Some other SQLite at version 0.
        conn_file(&path, "CREATE TABLE foo(x);");
        let err = Store::open(&path).err().unwrap();
        let conn = Connection::open(&path).unwrap();
        let created = has_table(&conn, "bookmarks") || has_table(&conn, "meta");
        drop(conn);
        let _ = std::fs::remove_file(&path);
        assert!(matches!(
            failure_msg(&err),
            Some(Msg::DatabaseUnknown { .. })
        ));
        assert!(!created);
    }

    #[test]
    fn refuses_unreadable_database() {
        let path = std::env::temp_dir().join(format!("marks-notadb-{}.db", std::process::id()));
        // Longer than a SQLite header, so SQLite reads it and rejects it.
        std::fs::write(&path, "not a database\n".repeat(200)).unwrap();
        let before = std::fs::read(&path).unwrap();
        let err = Store::open(&path).err().unwrap();
        let after = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            failure_msg(&err),
            Some(&Msg::DatabaseUnreadable {
                path: path.display().to_string()
            })
        );
        assert_eq!(after, before, "the file changed");
    }

    #[test]
    fn fresh_database_is_at_the_latest_version() {
        assert_eq!(MIGRATIONS.len(), 1);
        let s = Store::open_in_memory().unwrap();
        assert_eq!(store_version(&s), MIGRATIONS.len());
        let b = s.add(new_bookmark("https://a.com", "A", None)).unwrap();
        assert_eq!(b.status, Status::Pending);
        s.set_setting("k", "v").unwrap();
        assert_eq!(s.get_setting("k").unwrap().as_deref(), Some("v"));
        assert_eq!(s.get_setting("otra").unwrap(), None);
    }

    #[test]
    fn status_round_trips_and_rejects_unknown() {
        for st in [
            Status::Pending,
            Status::PendingEmbed,
            Status::Ready,
            Status::Failed,
        ] {
            assert_eq!(st.as_str().parse::<Status>().unwrap(), st);
        }
        assert!("otro".parse::<Status>().is_err());
    }

    #[test]
    fn export_puts_notes_right_after_front_matter() {
        let s = store_with("https://a.com", "T", "cuerpo");
        s.conn
            .execute("UPDATE bookmarks SET notes = 'mi nota'", [])
            .unwrap();
        let md = s.get(1).unwrap().unwrap().to_markdown("## Notas");
        assert!(md.contains("---\n\n## Notas\n\nmi nota\n\ncuerpo\n"));
    }

    #[test]
    fn notes_heading_is_a_parameter() {
        let s = store_with("https://a.com", "T", "cuerpo");
        s.conn
            .execute("UPDATE bookmarks SET notes = 'mi nota'", [])
            .unwrap();
        let md = s.get(1).unwrap().unwrap().to_markdown("## Notes");
        assert!(md.contains("---\n\n## Notes\n\nmi nota\n\ncuerpo\n"));
        assert!(!md.contains("## Notas"));
    }

    #[test]
    fn to_markdown_includes_metadata() {
        let s = store_with("https://a.com", "T", "cuerpo");
        let mut b = s.get(1).unwrap().unwrap();
        assert!(!b.to_markdown("## Notas").contains("author:"));
        b.meta.author = Some("Ana \"A\" Pérez".into());
        b.meta.site = Some("Blog".into());
        b.meta.published = Some("2026-01-02".into());
        b.meta.language = Some("es".into());
        b.meta.description = Some("Una descripción".into());
        let md = b.to_markdown("## Notas");
        assert!(md.contains("author: \"Ana \\\"A\\\" Pérez\"\n"));
        assert!(md.contains("site: \"Blog\"\n"));
        assert!(md.contains("published: \"2026-01-02\"\n"));
        assert!(md.contains("language: \"es\"\n"));
        assert!(md.contains("description: \"Una descripción\"\n"));
    }

    #[test]
    fn to_markdown_keeps_multiline_metadata_in_one_front_matter() {
        let s = store_with("https://a.com", "T", "cuerpo");
        let mut b = s.get(1).unwrap().unwrap();
        b.meta.description = Some("uno\n---\ndos\n...\ntres".into());
        b.title = "Título\n---\nen  dos".into();
        let md = b.to_markdown("## Notas");
        assert!(md.contains("description: \"uno --- dos ... tres\"\n"));
        assert!(md.contains("title: \"Título --- en dos\"\n"));
        assert_eq!(md.lines().filter(|l| *l == "---").count(), 2);
    }

    #[test]
    fn deleting_a_bookmark_cascades_to_chunks() {
        let s = store_with("https://a.com", "T", "cuerpo");
        s.conn
            .execute(
                "INSERT INTO chunks (bookmark_id, ord, text) VALUES (1, 0, 'x')",
                [],
            )
            .unwrap();
        s.conn
            .execute("DELETE FROM bookmarks WHERE id = 1", [])
            .unwrap();
        let left: i64 = s
            .conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(left, 0);
    }

    fn tag_list(tags: &[&str]) -> Vec<String> {
        tags.iter().map(|t| t.to_string()).collect()
    }

    fn untitled(s: &Store, url: &str, tags: &[&str]) -> Bookmark {
        s.add(NewBookmark {
            url: url.into(),
            title: None,
            notes: String::new(),
            tags: tag_list(tags),
            content: None,
            ..Default::default()
        })
        .unwrap()
    }

    #[test]
    fn tags_are_normalized() {
        assert_eq!(
            normalize_tags(&tag_list(&["Rust", " rust ", "a,b", ""])),
            ["rust", "a b"]
        );
        assert_eq!(normalize_tags(&tag_list(&["x   y"])), ["x y"]);
    }

    #[test]
    fn add_rejects_invalid_url() {
        let s = Store::open_in_memory().unwrap();
        let err = s.add(new_bookmark("no es url", "T", None)).unwrap_err();
        assert_eq!(failure_msg(&err), Some(&Msg::InvalidUrl));
    }

    #[test]
    fn list_without_content_omits_content() {
        let s = store_with("https://a.com", "A", "markdown largo");
        let light = s.list_without_content().unwrap();
        assert_eq!(light.len(), 1);
        assert_eq!(light[0].content, None);
        assert_eq!(light[0].title, "A");
        let full = s.get(light[0].id).unwrap().unwrap();
        assert_eq!(full.content.as_deref(), Some("markdown largo"));
    }

    #[test]
    fn add_without_title_uses_normalized_url() {
        let s = Store::open_in_memory().unwrap();
        let b = untitled(&s, "https://A.com/x#f", &["Uno", "uno"]);
        assert_eq!(b.title, "https://a.com/x");
        assert_eq!(b.tags, ["uno"]);
    }

    #[test]
    fn apply_summary_fills_only_empty_user_fields() {
        let s = Store::open_in_memory().unwrap();
        let empty = untitled(&s, "https://a.com", &[]);
        let mine = s
            .add(NewBookmark {
                title: Some("Mío".into()),
                tags: tag_list(&["x"]),
                ..new_bookmark("https://b.com", "", None)
            })
            .unwrap();
        let ai = Summary {
            title: "De la IA".into(),
            summary: "Resumen".into(),
            category: " Dev ".into(),
            tags: tag_list(&["IA", "ia", "web"]),
        };
        s.apply_summary(empty.id, &ai).unwrap();
        s.apply_summary(mine.id, &ai).unwrap();

        let empty = s.get(empty.id).unwrap().unwrap();
        assert_eq!(empty.title, "De la IA");
        assert_eq!(empty.tags, ["ia", "web"]);
        let mine = s.get(mine.id).unwrap().unwrap();
        assert_eq!(mine.title, "Mío");
        assert_eq!(mine.tags, ["x"]);
        for b in [empty, mine] {
            assert_eq!(b.summary.as_deref(), Some("Resumen"));
            assert_eq!(b.category.as_deref(), Some("Dev"));
        }
    }

    #[test]
    fn apply_summary_trims_and_nulls_empty_fields() {
        let s = Store::open_in_memory().unwrap();
        let b = untitled(&s, "https://a.com", &[]);
        let mut ai = Summary {
            title: "T".into(),
            summary: "  Resumen  ".into(),
            category: "".into(),
            tags: vec![],
        };
        s.apply_summary(b.id, &ai).unwrap();
        let got = s.get(b.id).unwrap().unwrap();
        assert_eq!(got.summary.as_deref(), Some("Resumen"));
        assert_eq!(got.category, None);
        ai.summary = " ".into();
        s.apply_summary(b.id, &ai).unwrap();
        assert_eq!(s.get(b.id).unwrap().unwrap().summary, None);
    }

    #[test]
    fn next_pending_prefers_pending_over_pending_embed() {
        let s = Store::open_in_memory().unwrap();
        assert!(s.next_pending().unwrap().is_none());
        let old = untitled(&s, "https://a.com", &[]);
        let new = untitled(&s, "https://b.com", &[]);
        s.set_status(old.id, Status::PendingEmbed, None).unwrap();
        assert_eq!(s.next_pending().unwrap().unwrap().id, new.id);
        s.set_status(new.id, Status::Ready, None).unwrap();
        assert_eq!(s.next_pending().unwrap().unwrap().id, old.id);
    }

    #[test]
    fn update_and_delete_roundtrip() {
        let s = Store::open_in_memory().unwrap();
        let b = untitled(&s, "https://a.com", &[]);
        let patch = BookmarkPatch {
            title: Some("Nuevo".into()),
            notes: Some("nota".into()),
            tags: Some(tag_list(&["B", "a"])),
        };
        let updated = s.update(b.id, patch).unwrap();
        assert_eq!(
            (updated.title.as_str(), updated.notes.as_str()),
            ("Nuevo", "nota")
        );
        assert_eq!(updated.tags, ["b", "a"]);
        let only_notes = BookmarkPatch {
            title: None,
            notes: Some("otra".into()),
            tags: None,
        };
        let kept = s.update(b.id, only_notes).unwrap();
        assert_eq!((kept.title.as_str(), kept.tags.len()), ("Nuevo", 2));
        let missing = BookmarkPatch {
            title: None,
            notes: None,
            tags: None,
        };
        assert!(s.update(b.id + 1, missing).is_err());
        assert!(s.delete(b.id).unwrap());
        assert!(!s.delete(b.id).unwrap());
    }

    #[test]
    fn categories_tags_and_stats() {
        let s = Store::open_in_memory().unwrap();
        let a = untitled(&s, "https://a.com", &["z", "a"]);
        let b = untitled(&s, "https://b.com", &["a"]);
        untitled(&s, "https://c.com", &[]);
        let ai = |category: &str| Summary {
            title: "T".into(),
            summary: "R".into(),
            category: category.into(),
            tags: vec![],
        };
        s.apply_summary(a.id, &ai("Web")).unwrap();
        s.apply_summary(b.id, &ai("Dev")).unwrap();
        assert_eq!(s.categories().unwrap(), ["Dev", "Web"]);
        assert_eq!(s.tags().unwrap(), ["a", "z"]);

        s.set_status(a.id, Status::Ready, None).unwrap();
        s.set_failed(b.id, &BookmarkError(vec![raw("boom")]))
            .unwrap();
        s.set_status(3, Status::Ready, None).unwrap();
        assert_eq!(s.count_unsummarized().unwrap(), 1);
        assert_eq!(
            s.stats().unwrap(),
            Stats {
                total: 3,
                pending: 0,
                failed: 1,
                unsummarized: 1,
                unextracted: 0,
                imported: 0
            }
        );
        assert_eq!(s.mark_unsummarized_pending().unwrap(), 1);
        assert_eq!(s.mark_ready_pending_embed().unwrap(), 1);
        assert_eq!(s.stats().unwrap().pending, 2);
    }

    #[test]
    fn unextracted_counts_only_pending_without_content() {
        let s = Store::open_in_memory().unwrap();
        s.add(new_bookmark("https://a.com", "A", Some("extraído")))
            .unwrap();
        s.add(new_bookmark("https://b.com", "B", Some("  \n")))
            .unwrap();
        let c = s.add(new_bookmark("https://c.com", "C", None)).unwrap();
        s.add(new_bookmark("https://d.com", "D", None)).unwrap();
        s.set_failed(c.id, &BookmarkError(vec![raw("boom")]))
            .unwrap();
        let stats = s.stats().unwrap();
        assert_eq!((stats.pending, stats.unextracted), (3, 2));
    }

    #[test]
    fn writes_refresh_updated_at() {
        let s = Store::open_in_memory().unwrap();
        let b = untitled(&s, "https://a.com", &[]);
        s.conn
            .execute("UPDATE bookmarks SET updated_at = 'viejo'", [])
            .unwrap();
        s.set_content(b.id, "md").unwrap();
        let got = s.get(b.id).unwrap().unwrap();
        assert_ne!(got.updated_at, "viejo");
        assert_eq!(got.content.as_deref(), Some("md"));
    }

    fn meta(f: impl FnOnce(&mut PageMeta)) -> PageMeta {
        let mut m = PageMeta::default();
        f(&mut m);
        m
    }

    fn png(tag: &str) -> Image {
        Image {
            mime: "image/png".into(),
            bytes: [b"\x89PNG\r\n\x1a\n".as_slice(), tag.as_bytes()].concat(),
        }
    }

    fn with_favicon_url(s: &Store, url: &str) -> Bookmark {
        let b = s.add(new_bookmark(url, "", None)).unwrap();
        let page = Page {
            markdown: "cuerpo".into(),
            meta: meta(|m| m.favicon = Some(format!("{url}/favicon.ico"))),
            ..Page::default()
        };
        s.set_page(b.id, &page).unwrap();
        s.get(b.id).unwrap().unwrap()
    }

    #[test]
    fn favicon_is_stored_once_and_due_again_after_extraction() {
        let s = Store::open_in_memory().unwrap();
        let plain = s.add(new_bookmark("https://plain.com", "", None)).unwrap();
        let b = with_favicon_url(&s, "https://a.com");
        let due = s.next_without_favicon().unwrap().unwrap();
        assert_eq!(
            due.id, b.id,
            "sin URL de favicon no es candidato: {}",
            plain.id
        );
        assert_eq!(due.content, None, "sin el Markdown");

        s.set_favicon(&due, &Some(png("fav"))).unwrap();
        assert_eq!(s.stored_favicon(b.id).unwrap(), Some(png("fav")));
        assert!(s.get(b.id).unwrap().unwrap().favicon_at.is_some());
        assert_eq!(s.next_without_favicon().unwrap(), None);
        let url = s.favicon_data_url(b.id).unwrap().unwrap();
        assert!(url.starts_with("data:image/png;base64,"));
        assert_eq!(s.favicon_data_url(plain.id).unwrap(), None);

        // A new extraction requests it again and keeps the stored one until then.
        let page = Page {
            markdown: "otra".into(),
            meta: due.meta.clone(),
            ..Page::default()
        };
        s.set_page(b.id, &page).unwrap();
        assert_eq!(s.next_without_favicon().unwrap().map(|b| b.id), Some(b.id));
        assert_eq!(s.stored_favicon(b.id).unwrap(), Some(png("fav")));
        assert!(s.stored_favicon(99).is_err());
    }

    #[test]
    fn set_favicon_skips_a_bookmark_whose_url_changed_meanwhile() {
        let s = Store::open_in_memory().unwrap();
        let b = with_favicon_url(&s, "https://a.com");
        let due = s.next_without_favicon().unwrap().unwrap();
        let page = Page {
            markdown: "cuerpo".into(),
            meta: meta(|m| m.favicon = Some("https://a.com/nuevo.ico".into())),
            ..Page::default()
        };
        s.set_page(b.id, &page).unwrap();
        s.set_favicon(&due, &Some(png("viejo"))).unwrap();
        assert_eq!(s.stored_favicon(b.id).unwrap(), None);
        assert_eq!(s.next_without_favicon().unwrap().map(|b| b.id), Some(b.id));
    }

    #[test]
    fn insert_copy_carries_the_favicon() {
        let source = Store::open_in_memory().unwrap();
        let b = with_favicon_url(&source, "https://a.com");
        source.set_favicon(&b, &Some(png("fav"))).unwrap();
        let b = source.get(b.id).unwrap().unwrap();

        let target = Store::open_in_memory().unwrap();
        target
            .add(new_bookmark("https://otro.com", "", None))
            .unwrap();
        let copy = target
            .insert_copy(&b, &source.stored_favicon(b.id).unwrap())
            .unwrap();
        assert_ne!(copy.id, b.id);
        assert_eq!(target.stored_favicon(copy.id).unwrap(), Some(png("fav")));
        assert_eq!(copy.favicon_at, b.favicon_at);
        assert_eq!(target.next_without_favicon().unwrap(), None);
    }

    #[test]
    fn set_page_writes_meta_and_titles_only_url_titles() {
        let s = Store::open_in_memory().unwrap();
        let bare = s.add(new_bookmark("https://a.com", "", None)).unwrap();
        let mine = s.add(new_bookmark("https://b.com", "Mío", None)).unwrap();
        let page = Page {
            title: Some("De la página".into()),
            markdown: "cuerpo".into(),
            meta: meta(|m| {
                m.author = Some("Ana".into());
                m.extractor = Some("defuddle".into());
            }),
        };
        s.set_page(bare.id, &page).unwrap();
        s.set_page(mine.id, &page).unwrap();
        let bare = s.get(bare.id).unwrap().unwrap();
        let mine = s.get(mine.id).unwrap().unwrap();
        assert_eq!(bare.title, "De la página");
        assert_eq!(mine.title, "Mío");
        assert_eq!(bare.content.as_deref(), Some("cuerpo"));
        assert_eq!(mine.meta.author.as_deref(), Some("Ana"));
        assert_eq!(mine.meta.extractor.as_deref(), Some("defuddle"));
    }

    #[test]
    fn set_page_sanitizes() {
        let s = store_with("https://a.com", "A", "x");
        let page = Page {
            title: None,
            markdown: "y".into(),
            meta: meta(|m| m.image = Some("javascript:x".into())),
        };
        s.set_page(1, &page).unwrap();
        assert_eq!(s.get(1).unwrap().unwrap().meta.image, None);
    }

    #[test]
    fn replace_page_keeps_user_title_and_tags_and_requeues() {
        let s = Store::open_in_memory().unwrap();
        let b = s
            .add(NewBookmark {
                tags: vec!["mía".into()],
                ..new_bookmark("https://a.com", "Mío", Some("viejo"))
            })
            .unwrap();
        s.set_failed(b.id, &BookmarkError::raw("falló")).unwrap();
        let page = Page {
            title: Some("De la página".into()),
            markdown: "nuevo".into(),
            meta: meta(|m| {
                m.author = Some("Ana".into());
                m.image = Some("javascript:x".into());
                m.extractor = Some("ai:reader".into());
            }),
        };
        s.replace_page(b.id, &page).unwrap();
        let b = s.get(b.id).unwrap().unwrap();
        assert_eq!(b.title, "Mío");
        assert_eq!(b.tags, vec!["mía".to_string()]);
        assert_eq!(b.content.as_deref(), Some("nuevo"));
        assert_eq!(b.meta.author.as_deref(), Some("Ana"));
        assert_eq!(b.meta.image, None);
        assert_eq!(b.meta.extractor.as_deref(), Some("ai:reader"));
        assert_eq!(b.status, Status::Pending);
        assert_eq!(b.error, None);
    }

    #[test]
    fn replace_page_clears_blocked() {
        let s = store_with("https://a.com", "A", "x");
        s.set_blocked(1).unwrap();
        assert_eq!(
            s.get(1).unwrap().unwrap().error_kind,
            Some(ErrorKind::Blocked)
        );
        let page = Page {
            markdown: "y".into(),
            ..Page::default()
        };
        s.replace_page(1, &page).unwrap();
        let b = s.get(1).unwrap().unwrap();
        assert_eq!(b.status, Status::Pending);
        assert_eq!((b.error, b.error_kind), (None, None));
    }

    #[test]
    fn fill_missing_meta_never_overwrites() {
        let s = store_with("https://a.com", "A", "x");
        let page = Page {
            title: None,
            markdown: "y".into(),
            meta: meta(|m| {
                m.author = Some("A".into());
                m.extractor = Some("uno".into());
            }),
        };
        s.set_page(1, &page).unwrap();
        s.fill_missing_meta(
            1,
            &meta(|m| {
                m.author = Some("B".into());
                m.site = Some("Sitio".into());
                m.extractor = Some("dos".into());
            }),
        )
        .unwrap();
        let m = s.get(1).unwrap().unwrap().meta;
        assert_eq!(m.author.as_deref(), Some("A"));
        assert_eq!(m.site.as_deref(), Some("Sitio"));
        assert_eq!(m.extractor.as_deref(), Some("uno"));
    }

    #[test]
    fn search_finds_by_author_and_site() {
        let s = Store::open_in_memory().unwrap();
        s.add(NewBookmark {
            meta: meta(|m| {
                m.author = Some("Pérez".into());
                m.site = Some("Ejemplo".into());
            }),
            ..new_bookmark("https://a.com", "T", None)
        })
        .unwrap();
        assert_eq!(s.search("Pérez", 10).unwrap().len(), 1);
        assert_eq!(s.search("Ejemplo", 10).unwrap().len(), 1);
    }

    #[test]
    fn blocked_kind_is_cleared_when_leaving_failed() {
        let s = store_with("https://a.com", "A", "x");
        s.set_blocked(1).unwrap();
        let b = s.get(1).unwrap().unwrap();
        assert_eq!(b.status, Status::Failed);
        assert_eq!(
            b.error,
            Some(BookmarkError(vec![ErrorPart::Msg(Msg::Blocked)]))
        );
        assert_eq!(b.error_kind, Some(ErrorKind::Blocked));
        s.set_status(1, Status::Pending, None).unwrap();
        let b = s.get(1).unwrap().unwrap();
        assert_eq!((b.error, b.error_kind), (None, None));
    }

    #[test]
    fn next_without_content_skips_filled_and_ready() {
        let s = Store::open_in_memory().unwrap();
        s.add(new_bookmark("https://a.com", "A", Some("lleno")))
            .unwrap();
        s.add(new_bookmark("https://b.com", "B", Some("  \n")))
            .unwrap();
        let c = s.add(new_bookmark("https://c.com", "C", None)).unwrap();
        s.set_status(c.id, Status::Ready, None).unwrap();
        assert_eq!(s.next_without_content().unwrap().unwrap().title, "B");
        s.set_status(2, Status::Ready, None).unwrap();
        assert!(s.next_without_content().unwrap().is_none());
    }

    fn set_updated(s: &Store, id: i64, at: &str) {
        s.conn
            .execute(
                "UPDATE bookmarks SET updated_at = ?1 WHERE id = ?2",
                params![at, id],
            )
            .unwrap();
    }

    #[test]
    fn next_pending_takes_the_last_queued_first() {
        let s = Store::open_in_memory().unwrap();
        let a = untitled(&s, "https://a.com", &[]);
        let b = untitled(&s, "https://b.com", &[]);
        let c = untitled(&s, "https://c.com", &[]);
        set_updated(&s, a.id, "2026-01-01T00:00:00Z");
        set_updated(&s, b.id, "2026-01-02T00:00:00Z");
        set_updated(&s, c.id, "2026-01-03T00:00:00Z");
        s.set_status(c.id, Status::PendingEmbed, None).unwrap();
        set_updated(&s, c.id, "2026-01-03T00:00:00Z");
        assert_eq!(s.next_pending().unwrap().unwrap().id, b.id);
        s.delete(b.id).unwrap();
        s.delete(c.id).unwrap();
        assert_eq!(s.next_pending().unwrap().unwrap().id, a.id);
        let d = untitled(&s, "https://d.com", &[]);
        set_updated(&s, a.id, "2026-01-01T00:00:00Z");
        set_updated(&s, d.id, "2026-01-01T00:00:00Z");
        assert_eq!(s.next_pending().unwrap().unwrap().id, d.id);
    }

    #[test]
    fn next_without_content_takes_the_last_queued_first() {
        let s = Store::open_in_memory().unwrap();
        let a = s.add(new_bookmark("https://a.com", "A", None)).unwrap();
        let b = s.add(new_bookmark("https://b.com", "B", None)).unwrap();
        set_updated(&s, a.id, "2026-01-01T00:00:00Z");
        set_updated(&s, b.id, "2026-01-02T00:00:00Z");
        assert_eq!(s.next_without_content().unwrap().unwrap().id, b.id);
        s.delete(b.id).unwrap();
        assert_eq!(s.next_without_content().unwrap().unwrap().id, a.id);
        let d = s.add(new_bookmark("https://d.com", "D", None)).unwrap();
        set_updated(&s, a.id, "2026-01-01T00:00:00Z");
        set_updated(&s, d.id, "2026-01-01T00:00:00Z");
        assert_eq!(s.next_without_content().unwrap().unwrap().id, d.id);
    }

    #[test]
    fn next_without_favicon_takes_the_newest_first() {
        let s = Store::open_in_memory().unwrap();
        let a = with_favicon_url(&s, "https://a.com");
        let b = with_favicon_url(&s, "https://b.com");
        set_updated(&s, a.id, "2026-01-01T00:00:00Z");
        set_updated(&s, b.id, "2026-01-02T00:00:00Z");
        assert_eq!(s.next_without_favicon().unwrap().map(|x| x.id), Some(b.id));
    }
}
