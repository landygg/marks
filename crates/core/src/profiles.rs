//! Profile registry: each profile is its own SQLite database; the primary one is profile 1 and
//! also holds the registry (`profiles`) and the global settings.

use crate::msg::{Msg, MsgExt, failure_msg};
use crate::settings::{API_TOKEN_KEY, random_token};
use crate::store::{Bookmark, Status, Store, lock};
use anyhow::Result;
use rusqlite::{OptionalExtension, Transaction, TransactionBehavior, params};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use subtle::ConstantTimeEq;

pub const DEFAULT_PROFILE_ID: i64 = 1;

/// Name and initials of the first profile, set by the caller (per language).
pub struct ProfileSeed<'a> {
    pub name: &'a str,
    pub initials: &'a str,
}

/// Primary-database setting holding the active profile id.
const ACTIVE_PROFILE_KEY: &str = "active_profile";
const MAX_NAME_CHARS: usize = 40;
const MAX_INITIALS_CHARS: usize = 2;
const PROFILES_DIR_SUFFIX: &str = "-profiles";
const PROFILE_DB_EXTENSION: &str = "db";

/// Outside `MIGRATIONS`: migrations run on every database and this table lives only in the primary.
const CREATE_PROFILES: &str = "CREATE TABLE IF NOT EXISTS profiles (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    name     TEXT NOT NULL UNIQUE COLLATE NOCASE,
    initials TEXT NOT NULL,
    token    TEXT NOT NULL UNIQUE
)";
const COUNT_PROFILES: &str = "SELECT COUNT(*) FROM profiles";
const INSERT_PROFILE: &str =
    "INSERT INTO profiles (id, name, initials, token) VALUES (?1, ?2, ?3, ?4)";
const INSERT_NEW_PROFILE: &str = "INSERT INTO profiles (name, initials, token) VALUES (?1, ?2, ?3)";
const SELECT_PROFILES: &str = "SELECT id, name, initials FROM profiles ORDER BY id";
const SELECT_PROFILE: &str = "SELECT id, name, initials FROM profiles WHERE id = ?1";
const SELECT_BY_NAME: &str = "SELECT id, name, initials FROM profiles WHERE name = ?1";
const NAME_TAKEN: &str = "SELECT EXISTS (SELECT 1 FROM profiles WHERE name = ?1 AND id <> ?2)";
const UPDATE_PROFILE: &str = "UPDATE profiles SET name = ?2, initials = ?3 WHERE id = ?1";
const SELECT_TOKEN: &str = "SELECT token FROM profiles WHERE id = ?1";
const UPDATE_TOKEN: &str = "UPDATE profiles SET token = ?2 WHERE id = ?1";
const SELECT_TOKENS: &str = "SELECT id, name, initials, token FROM profiles";
const DELETE_PROFILE: &str = "DELETE FROM profiles WHERE id = ?1";
/// Files of a WAL-mode SQLite database, as suffixes of its path.
const DB_FILE_SUFFIXES: [&str; 3] = ["", "-wal", "-shm"];
const DELETE_SETTING: &str = "DELETE FROM settings WHERE key = ?1";
/// No profile has this id: `NAME_TAKEN` checks every row on create.
const NO_PROFILE_ID: i64 = 0;

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub initials: String,
}

fn profile_from_row(r: &rusqlite::Row) -> rusqlite::Result<Profile> {
    Ok(Profile {
        id: r.get(0)?,
        name: r.get(1)?,
        initials: r.get(2)?,
    })
}

/// `<primary without extension>-profiles/<id>.db`; for `id == 1`, the primary.
pub fn profile_db_path(primary: &Path, id: i64) -> PathBuf {
    if id == DEFAULT_PROFILE_ID {
        return primary.to_path_buf();
    }
    let stem = primary.file_stem().unwrap_or_default().to_string_lossy();
    primary
        .with_file_name(format!("{stem}{PROFILES_DIR_SUFFIX}"))
        .join(id.to_string())
        .with_extension(PROFILE_DB_EXTENSION)
}

/// Trimmed name and trimmed uppercase initials, or the validation error.
fn validate(name: &str, initials: &str) -> Result<(String, String)> {
    let name = name.trim();
    if !(1..=MAX_NAME_CHARS).contains(&name.chars().count()) {
        return Err(Msg::ProfileName {
            max: MAX_NAME_CHARS,
        }
        .err());
    }
    // Counted after uppercasing: some letters grow (ß → SS).
    let initials = initials.trim().to_uppercase();
    if !(1..=MAX_INITIALS_CHARS).contains(&initials.chars().count()) {
        return Err(Msg::ProfileInitials {
            max: MAX_INITIALS_CHARS,
        }
        .err());
    }
    Ok((name.to_string(), initials))
}

/// Sole owner of the open stores: each profile is opened once and reused.
///
/// Careful: the registry and profile 1's data share the same `Mutex<Store>` (the primary). Do
/// not call any `Library` method while holding a guard from `primary()` or `store(1)`: it would
/// deadlock.
pub struct Library {
    primary_path: PathBuf,
    primary: Arc<Mutex<Store>>,
    stores: Mutex<HashMap<i64, Arc<Mutex<Store>>>>,
}

impl Library {
    /// Opens the primary and, the first time, creates the registry with profile 1, adopting the
    /// v0.4 API token if there was one.
    pub fn open(primary: &Path, seed: &ProfileSeed) -> Result<Library> {
        let store = Store::open(primary)?;
        store.conn.execute_batch(CREATE_PROFILES)?;
        let tx = Transaction::new_unchecked(&store.conn, TransactionBehavior::Immediate)?;
        let count: i64 = store.conn.query_row(COUNT_PROFILES, [], |r| r.get(0))?;
        if count == 0 {
            let token = match store.get_setting(API_TOKEN_KEY)? {
                Some(token) => token,
                None => random_token()?,
            };
            store.conn.execute(
                INSERT_PROFILE,
                params![DEFAULT_PROFILE_ID, seed.name, seed.initials, token],
            )?;
            store.conn.execute(DELETE_SETTING, [API_TOKEN_KEY])?;
        }
        tx.commit()?;
        let primary_store = Arc::new(Mutex::new(store));
        Ok(Library {
            primary_path: primary.to_path_buf(),
            stores: Mutex::new(HashMap::from([(DEFAULT_PROFILE_ID, primary_store.clone())])),
            primary: primary_store,
        })
    }

    /// Primary store, for global settings.
    pub fn primary(&self) -> Arc<Mutex<Store>> {
        self.primary.clone()
    }

    fn registry(&self) -> Result<MutexGuard<'_, Store>> {
        lock(&self.primary)
    }

    fn stores(&self) -> Result<MutexGuard<'_, HashMap<i64, Arc<Mutex<Store>>>>> {
        self.stores.lock().map_err(|_| Msg::StoresPoisoned.err())
    }

    fn get(&self, id: i64) -> Result<Option<Profile>> {
        Ok(self
            .registry()?
            .conn
            .query_row(SELECT_PROFILE, [id], profile_from_row)
            .optional()?)
    }

    fn require(&self, id: i64) -> Result<Profile> {
        self.get(id)?.msg(Msg::ProfileNotFound)
    }

    /// Profiles by ascending id.
    pub fn profiles(&self) -> Result<Vec<Profile>> {
        let registry = self.registry()?;
        let mut stmt = registry.conn.prepare(SELECT_PROFILES)?;
        let rows = stmt.query_map([], profile_from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// By trimmed name, case-insensitively (ASCII, like `COLLATE NOCASE`).
    pub fn find(&self, name: &str) -> Result<Option<Profile>> {
        Ok(self
            .registry()?
            .conn
            .query_row(SELECT_BY_NAME, [name.trim()], profile_from_row)
            .optional()?)
    }

    /// Creates the profile with a new token; its file is not created until it is opened.
    pub fn create(&self, name: &str, initials: &str) -> Result<Profile> {
        let (name, initials) = validate(name, initials)?;
        let token = random_token()?;
        let registry = self.registry()?;
        ensure_name_free(&registry, &name, NO_PROFILE_ID)?;
        registry
            .conn
            .execute(INSERT_NEW_PROFILE, params![name, initials, token])?;
        Ok(Profile {
            id: registry.conn.last_insert_rowid(),
            name,
            initials,
        })
    }

    pub fn rename(&self, id: i64, name: &str, initials: &str) -> Result<Profile> {
        let (name, initials) = validate(name, initials)?;
        let registry = self.registry()?;
        // Under the same lock: "does not exist" is reported before "name taken".
        if registry
            .conn
            .query_row(SELECT_PROFILE, [id], profile_from_row)
            .optional()?
            .is_none()
        {
            return Err(Msg::ProfileNotFound.err());
        }
        ensure_name_free(&registry, &name, id)?;
        if registry
            .conn
            .execute(UPDATE_PROFILE, params![id, name, initials])?
            == 0
        {
            return Err(Msg::ProfileNotFound.err());
        }
        Ok(Profile { id, name, initials })
    }

    /// The profile's store: opened the first time (outside the map lock) and reused.
    pub fn store(&self, id: i64) -> Result<Arc<Mutex<Store>>> {
        self.require(id)?;
        if let Some(store) = self.stores()?.get(&id) {
            return Ok(store.clone());
        }
        let opened = Store::open(&profile_db_path(&self.primary_path, id))?;
        self.register(id, opened)
    }

    /// Like `store`, but `None` if the profile no longer exists (deleted): callers iterating profiles
    /// treat it as an end, not a failure.
    pub fn existing_store(&self, id: i64) -> Result<Option<Arc<Mutex<Store>>>> {
        match self.store(id) {
            Ok(store) => Ok(Some(store)),
            Err(e) if failure_msg(&e) == Some(&Msg::ProfileNotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Puts the freshly opened store in the map, or discards it with its file if the profile no
    /// longer exists.
    fn register(&self, id: i64, opened: Store) -> Result<Arc<Mutex<Store>>> {
        // `delete` removes the row and the map entry while holding this same lock, so here either the
        // row is still there (and `delete`, coming later, will remove this entry and the file) or it is
        // gone and the file we just created is removed here. Order: map, then registry.
        let mut stores = self.stores()?;
        if self.get(id)?.is_none() {
            drop(stores);
            drop(opened);
            remove_db_files(&profile_db_path(&self.primary_path, id))?;
            return Err(Msg::ProfileNotFound.err());
        }
        // If another thread opened it concurrently, the first one wins and this one is discarded.
        Ok(stores
            .entry(id)
            .or_insert_with(|| Arc::new(Mutex::new(opened)))
            .clone())
    }

    /// Active profile; profile 1 if the stored one does not exist or is invalid.
    pub fn active(&self) -> Result<i64> {
        let stored = self.registry()?.get_setting(ACTIVE_PROFILE_KEY)?;
        match stored.and_then(|v| v.parse::<i64>().ok()) {
            Some(id) if self.get(id)?.is_some() => Ok(id),
            _ => Ok(DEFAULT_PROFILE_ID),
        }
    }

    pub fn set_active(&self, id: i64) -> Result<()> {
        self.require(id)?;
        self.registry()?
            .set_setting(ACTIVE_PROFILE_KEY, &id.to_string())
    }

    pub fn active_store(&self) -> Result<Arc<Mutex<Store>>> {
        self.store(self.active()?)
    }

    pub fn token(&self, id: i64) -> Result<String> {
        self.registry()?
            .conn
            .query_row(SELECT_TOKEN, [id], |r| r.get(0))
            .optional()?
            .msg(Msg::ProfileNotFound)
    }

    /// Replaces the profile's token: an extension paired with the old one stops working.
    pub fn regenerate_token(&self, id: i64) -> Result<String> {
        let token = random_token()?;
        if self
            .registry()?
            .conn
            .execute(UPDATE_TOKEN, params![id, token])?
            == 0
        {
            return Err(Msg::ProfileNotFound.err());
        }
        Ok(token)
    }

    /// Profile owning the presented token. Compares in constant time against every row, without
    /// stopping at the first match; an empty token never matches.
    pub fn profile_for_token(&self, presented: &str) -> Result<Option<Profile>> {
        if presented.is_empty() {
            return Ok(None);
        }
        let registry = self.registry()?;
        let mut stmt = registry.conn.prepare(SELECT_TOKENS)?;
        let rows = stmt.query_map([], |r| Ok((profile_from_row(r)?, r.get::<_, String>(3)?)))?;
        let mut found = None;
        for row in rows {
            let (profile, token) = row?;
            if bool::from(token.as_bytes().ct_eq(presented.as_bytes())) {
                found = Some(profile);
            }
        }
        Ok(found)
    }

    /// Removes the row, releases its store and deletes its file with `-wal` and `-shm`. Anyone still
    /// holding the store (the worker) keeps writing to the already unlinked file.
    pub fn delete(&self, id: i64) -> Result<()> {
        if id == DEFAULT_PROFILE_ID {
            return Err(Msg::DefaultProfile.err());
        }
        let was_active = self.active()? == id;
        let released = {
            // Row and map entry under the map lock: see `register`.
            let mut stores = self.stores()?;
            if self.registry()?.conn.execute(DELETE_PROFILE, [id])? == 0 {
                return Err(Msg::ProfileNotFound.err());
            }
            stores.remove(&id)
        };
        drop(released);
        // Before deleting files: if that fails, the row is gone and the active profile must not point to it.
        if was_active {
            self.set_active(DEFAULT_PROFILE_ID)?;
        }
        remove_db_files(&profile_db_path(&self.primary_path, id))
    }

    /// Returns the copy created in the target. Inserts into the target first and then deletes from
    /// the source, never holding both locks: if the delete fails, it is left in both.
    pub fn move_bookmark(&self, id: i64, from: i64, to: i64) -> Result<Bookmark> {
        if from == to {
            return Err(Msg::SameProfile.err());
        }
        let source = self.store(from)?;
        let target_name = self.require(to)?.name;
        let target = self.store(to)?;
        self.move_between(id, &source, to, &target_name, &target)
    }

    fn move_between(
        &self,
        id: i64,
        source: &Mutex<Store>,
        to: i64,
        target_name: &str,
        target: &Mutex<Store>,
    ) -> Result<Bookmark> {
        let (mut copy, favicon) = {
            let source = lock(source)?;
            let copy = source.get(id)?.msg(Msg::BookmarkNotFound)?;
            (copy, source.stored_favicon(id)?)
        };
        // Embeddings are regenerated in the target; no new summary.
        if copy.status == Status::Ready {
            copy.status = Status::PendingEmbed;
        }
        let moved = {
            let target = lock(target)?;
            if target.find_by_url(&copy.url)?.is_some() {
                return Err(Msg::UrlInProfile {
                    profile: target_name.to_string(),
                }
                .err());
            }
            target.insert_copy(&copy, &favicon)?
        };
        // `delete` removes the row before unlinking the file: if the row is still there after inserting,
        // the copy is in a live file (and if the profile is deleted later, it goes with it); otherwise
        // it may have gone to an unlinked file and the original stays. No store is held.
        self.require(to)?;
        lock(source)?.delete(id)?;
        Ok(moved)
    }

    /// Sum of `Store::requeue_if_embedding_changed` over all profiles.
    pub fn requeue_if_embedding_changed(&self, embedding_id: &str) -> Result<usize> {
        let mut total = 0;
        for profile in self.profiles()? {
            let store = self.store(profile.id)?;
            total += lock(&store)?.requeue_if_embedding_changed(embedding_id)?;
        }
        Ok(total)
    }
}

/// Deletes the database and its `-wal` and `-shm`; missing files are not an error.
fn remove_db_files(path: &Path) -> Result<()> {
    for suffix in DB_FILE_SUFFIXES {
        let mut file = path.as_os_str().to_owned();
        file.push(suffix);
        match std::fs::remove_file(&file) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
            _ => {}
        }
    }
    Ok(())
}

/// `Msg::ProfileExists` if another profile (other than `id`) already uses the name.
fn ensure_name_free(registry: &Store, name: &str, id: i64) -> Result<()> {
    let taken: bool = registry
        .conn
        .query_row(NAME_TAKEN, params![name, id], |r| r.get(0))?;
    if taken {
        return Err(Msg::ProfileExists.err());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: ProfileSeed = ProfileSeed {
        name: "Personal",
        initials: "PE",
    };
    use crate::ai::Summary;
    use crate::msg::{BookmarkError, ErrorPart};
    use crate::store::{BookmarkPatch, NewBookmark};
    use crate::vectors::EMBEDDING_ID_KEY;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("marks-profiles-{}-{n}", std::process::id()))
    }

    fn temp_library() -> (Library, PathBuf) {
        let dir = temp_dir();
        let library = Library::open(&dir.join("marks.db"), &SEED).unwrap();
        (library, dir)
    }

    fn cleanup(library: Library, dir: PathBuf) {
        drop(library);
        std::fs::remove_dir_all(dir).unwrap();
    }

    fn new_bookmark(url: &str) -> NewBookmark {
        NewBookmark {
            url: url.into(),
            title: Some("palabraunica".into()),
            notes: String::new(),
            tags: vec![],
            content: None,
            ..Default::default()
        }
    }

    fn err_of<T>(r: Result<T>) -> Msg {
        let e = r.err().expect("se esperaba un error");
        failure_msg(&e).cloned().expect("se esperaba un Failure")
    }

    #[test]
    fn too_new_profile_fails_with_database_too_new() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        drop(lib.store(2).unwrap());
        let path = profile_db_path(&dir.join("marks.db"), 2);
        drop(lib);
        rusqlite::Connection::open(&path)
            .unwrap()
            .execute_batch(
                "PRAGMA user_version = 2; UPDATE meta SET value = '2' WHERE key = 'min_reader';",
            )
            .unwrap();
        let lib = Library::open(&dir.join("marks.db"), &SEED).unwrap();
        assert_eq!(
            err_of(lib.store(2)),
            Msg::DatabaseTooNew {
                path: path.display().to_string()
            }
        );
        cleanup(lib, dir);
    }

    #[test]
    fn first_open_creates_the_default_profile() {
        let (lib, dir) = temp_library();
        assert_eq!(
            lib.profiles().unwrap(),
            vec![Profile {
                id: DEFAULT_PROFILE_ID,
                name: SEED.name.into(),
                initials: SEED.initials.into(),
            }]
        );
        let token = lib.token(1).unwrap();
        assert_eq!(token.len(), 64);
        assert!(
            token
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
        );
        assert_eq!(lib.active().unwrap(), 1);
        cleanup(lib, dir);
    }

    #[test]
    fn first_open_adopts_the_v04_token_and_is_idempotent() {
        let dir = temp_dir();
        let path = dir.join("marks.db");
        Store::open(&path)
            .unwrap()
            .set_setting(API_TOKEN_KEY, "abc")
            .unwrap();
        for _ in 0..2 {
            let lib = Library::open(&path, &SEED).unwrap();
            assert_eq!(lib.token(1).unwrap(), "abc");
            assert_eq!(lib.profiles().unwrap().len(), 1);
            assert_eq!(
                lock(&lib.primary())
                    .unwrap()
                    .get_setting(API_TOKEN_KEY)
                    .unwrap(),
                None
            );
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn profile_db_path_sits_next_to_the_primary() {
        let primary = Path::new("/d/marks.db");
        assert_eq!(
            profile_db_path(primary, 2),
            PathBuf::from("/d/marks-profiles/2.db")
        );
        assert_eq!(profile_db_path(primary, 1), PathBuf::from("/d/marks.db"));
    }

    #[test]
    fn create_validates_name_and_initials() {
        let (lib, dir) = temp_library();
        let p = lib.create("  Trabajo ", "tr").unwrap();
        assert_eq!(
            p,
            Profile {
                id: 2,
                name: "Trabajo".into(),
                initials: "TR".into()
            }
        );
        assert_ne!(lib.token(2).unwrap(), lib.token(1).unwrap());
        assert_eq!(err_of(lib.create("trabajo", "T")), Msg::ProfileExists);
        assert_eq!(
            err_of(lib.create("", "A")),
            Msg::ProfileName {
                max: MAX_NAME_CHARS
            }
        );
        assert_eq!(
            err_of(lib.create("   ", "A")),
            Msg::ProfileName {
                max: MAX_NAME_CHARS
            }
        );
        assert_eq!(
            err_of(lib.create(&"a".repeat(41), "A")),
            Msg::ProfileName {
                max: MAX_NAME_CHARS
            }
        );
        assert_eq!(
            err_of(lib.create("X", "")),
            Msg::ProfileInitials {
                max: MAX_INITIALS_CHARS
            }
        );
        assert_eq!(
            err_of(lib.create("X", "ABC")),
            Msg::ProfileInitials {
                max: MAX_INITIALS_CHARS
            }
        );
        assert_eq!(lib.create(&"a".repeat(40), "A").unwrap().id, 3);
        let nu = lib.create("Ñu", "é🙂").unwrap();
        assert_eq!(nu.initials, "É🙂");
        assert_eq!(lib.find("TRABAJO").unwrap(), Some(p));
        assert_eq!(lib.find("nadie").unwrap(), None);
        assert_eq!(lib.profiles().unwrap().len(), 4);
        cleanup(lib, dir);
    }

    #[test]
    fn rename_changes_name_and_initials() {
        let (lib, dir) = temp_library();
        let casa = lib.rename(1, "Casa", "ca").unwrap();
        assert_eq!(
            casa,
            Profile {
                id: 1,
                name: "Casa".into(),
                initials: "CA".into()
            }
        );
        assert_eq!(lib.profiles().unwrap(), vec![casa]);
        // Changing only the case of its own name is allowed.
        assert_eq!(lib.rename(1, "CASA", "C").unwrap().name, "CASA");
        lib.create("Trabajo", "TR").unwrap();
        assert_eq!(err_of(lib.rename(1, "trabajo", "T")), Msg::ProfileExists);
        assert_eq!(
            err_of(lib.rename(1, "", "T")),
            Msg::ProfileName {
                max: MAX_NAME_CHARS
            }
        );
        assert_eq!(
            err_of(lib.rename(1, "Casa", "ABC")),
            Msg::ProfileInitials {
                max: MAX_INITIALS_CHARS
            }
        );
        assert_eq!(err_of(lib.rename(99, "Otro", "O")), Msg::ProfileNotFound);
        // "Does not exist" is reported before "name taken".
        assert_eq!(err_of(lib.rename(99, "Trabajo", "T")), Msg::ProfileNotFound);
        cleanup(lib, dir);
    }

    #[test]
    fn profiles_are_isolated() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        lock(&lib.store(1).unwrap())
            .unwrap()
            .add(new_bookmark("https://a.com"))
            .unwrap();
        let two = lib.store(2).unwrap();
        let two = lock(&two).unwrap();
        assert!(two.list().unwrap().is_empty());
        assert!(two.search("palabraunica", 10).unwrap().is_empty());
        drop(two);
        assert!(profile_db_path(&dir.join("marks.db"), 2).exists());
        assert!(Arc::ptr_eq(&lib.store(2).unwrap(), &lib.store(2).unwrap()));
        assert_eq!(err_of(lib.store(99)), Msg::ProfileNotFound);
        cleanup(lib, dir);
    }

    #[test]
    fn store_of_a_profile_ignores_the_active_one() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        lib.set_active(2).unwrap();
        assert_eq!(lib.active().unwrap(), 2);
        lock(&lib.store(1).unwrap())
            .unwrap()
            .add(new_bookmark("https://a.com"))
            .unwrap();
        assert_eq!(
            lock(&lib.store(1).unwrap()).unwrap().list().unwrap().len(),
            1
        );
        assert!(
            lock(&lib.active_store().unwrap())
                .unwrap()
                .list()
                .unwrap()
                .is_empty()
        );
        cleanup(lib, dir);
    }

    #[test]
    fn active_falls_back_to_the_default() {
        let (lib, dir) = temp_library();
        for bad in ["99", "x"] {
            lock(&lib.primary())
                .unwrap()
                .set_setting(ACTIVE_PROFILE_KEY, bad)
                .unwrap();
            assert_eq!(lib.active().unwrap(), DEFAULT_PROFILE_ID);
        }
        assert_eq!(err_of(lib.set_active(99)), Msg::ProfileNotFound);
        cleanup(lib, dir);
    }

    #[test]
    fn missing_profile_file_opens_empty() {
        let dir = temp_dir();
        let primary = dir.join("marks.db");
        let lib = Library::open(&primary, &SEED).unwrap();
        let p = lib.create("Trabajo", "TR").unwrap();
        lock(&lib.store(p.id).unwrap())
            .unwrap()
            .add(new_bookmark("https://a.com"))
            .unwrap();
        drop(lib);
        let path = profile_db_path(&primary, p.id);
        assert!(path.exists());
        for file in sidecars(&path) {
            let _ = std::fs::remove_file(file);
        }
        let lib = Library::open(&primary, &SEED).unwrap();
        let store = lib.store(p.id).unwrap();
        assert!(lock(&store).unwrap().list().unwrap().is_empty());
        assert!(path.exists());
        drop(store);
        cleanup(lib, dir);
    }

    /// The database file and its `-wal` and `-shm`.
    fn sidecars(path: &Path) -> Vec<PathBuf> {
        ["", "-wal", "-shm"]
            .iter()
            .map(|suffix| {
                let mut p = path.as_os_str().to_owned();
                p.push(suffix);
                PathBuf::from(p)
            })
            .collect()
    }

    #[test]
    fn delete_removes_the_row_and_the_files() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        let token = lib.token(2).unwrap();
        lock(&lib.store(2).unwrap())
            .unwrap()
            .add(new_bookmark("https://a.com"))
            .unwrap();
        let path = profile_db_path(&dir.join("marks.db"), 2);
        assert!(path.exists());
        lib.delete(2).unwrap();
        assert_eq!(
            lib.profiles()
                .unwrap()
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>(),
            vec![1]
        );
        for file in sidecars(&path) {
            assert!(!file.exists(), "sigue existiendo {}", file.display());
        }
        assert_eq!(err_of(lib.store(2)), Msg::ProfileNotFound);
        for file in sidecars(&path) {
            assert!(!file.exists(), "store(2) recreó {}", file.display());
        }
        assert_eq!(lib.profile_for_token(&token).unwrap(), None);
        assert_eq!(err_of(lib.delete(99)), Msg::ProfileNotFound);
        cleanup(lib, dir);
    }

    #[test]
    fn store_opened_during_a_deletion_is_discarded() {
        // Simulates a `store(2)` that passed the check and opened the file right after `delete(2)`.
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        lib.delete(2).unwrap();
        let path = profile_db_path(&dir.join("marks.db"), 2);
        let late = Store::open(&path).unwrap();
        assert!(path.exists());
        assert_eq!(err_of(lib.register(2, late)), Msg::ProfileNotFound);
        for file in sidecars(&path) {
            assert!(!file.exists(), "quedó {}", file.display());
        }
        assert!(!lib.stores().unwrap().contains_key(&2));
        cleanup(lib, dir);
    }

    #[test]
    fn default_profile_cannot_be_deleted() {
        let (lib, dir) = temp_library();
        assert_eq!(err_of(lib.delete(1)), Msg::DefaultProfile);
        assert_eq!(lib.profiles().unwrap().len(), 1);
        assert!(lock(&lib.store(1).unwrap()).unwrap().list().is_ok());
        assert!(dir.join("marks.db").exists());
        cleanup(lib, dir);
    }

    #[test]
    fn deleting_the_active_profile_activates_the_default() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        lib.set_active(2).unwrap();
        lib.delete(2).unwrap();
        assert_eq!(lib.active().unwrap(), 1);
        assert_eq!(
            lock(&lib.primary())
                .unwrap()
                .get_setting(ACTIVE_PROFILE_KEY)
                .unwrap(),
            Some("1".into())
        );
        cleanup(lib, dir);
    }

    #[test]
    #[cfg(unix)]
    fn failed_file_removal_still_activates_the_default() {
        use std::os::unix::fs::PermissionsExt;
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        lib.set_active(2).unwrap();
        drop(lib.store(2).unwrap());
        let profiles_dir = profile_db_path(&dir.join("marks.db"), 2)
            .parent()
            .unwrap()
            .to_path_buf();
        // Without write permission on the directory, the file cannot be unlinked.
        let read_only = std::fs::Permissions::from_mode(0o555);
        std::fs::set_permissions(&profiles_dir, read_only).unwrap();
        let result = lib.delete(2);
        std::fs::set_permissions(&profiles_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(result.is_err());
        assert_eq!(lib.profiles().unwrap().len(), 1);
        assert_eq!(
            lock(&lib.primary())
                .unwrap()
                .get_setting(ACTIVE_PROFILE_KEY)
                .unwrap(),
            Some("1".into())
        );
        cleanup(lib, dir);
    }

    #[test]
    fn existing_store_ends_quietly_once_the_profile_is_deleted() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        assert!(lib.existing_store(2).unwrap().is_some());
        lib.delete(2).unwrap();
        assert!(lib.existing_store(2).unwrap().is_none());
        assert!(lib.existing_store(99).unwrap().is_none());
        assert!(lib.existing_store(1).unwrap().is_some());
        cleanup(lib, dir);
    }

    #[test]
    fn edit_sent_after_a_switch_lands_in_its_profile() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        let a = lock(&lib.store(1).unwrap())
            .unwrap()
            .add(new_bookmark("https://a.com"))
            .unwrap();
        let b = lock(&lib.store(2).unwrap())
            .unwrap()
            .add(new_bookmark("https://b.com"))
            .unwrap();
        assert_eq!(a.id, b.id);
        // The edit started in profile 1; the switch to 2 arrives before it.
        lib.set_active(2).unwrap();
        let patch = BookmarkPatch {
            title: Some("editado".into()),
            notes: None,
            tags: None,
        };
        lock(&lib.store(1).unwrap())
            .unwrap()
            .update(a.id, patch)
            .unwrap();
        let in_a = lock(&lib.store(1).unwrap()).unwrap().get(a.id).unwrap();
        let in_b = lock(&lib.active_store().unwrap())
            .unwrap()
            .get(b.id)
            .unwrap();
        assert_eq!(in_a.unwrap().title, "editado");
        assert_eq!(in_b.unwrap().title, b.title);
        cleanup(lib, dir);
    }

    #[test]
    fn store_in_use_survives_its_profile_deletion() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        let s = lib.store(2).unwrap();
        let path = profile_db_path(&dir.join("marks.db"), 2);
        assert!(path.exists());
        lib.delete(2).unwrap();
        lock(&s)
            .unwrap()
            .add(new_bookmark("https://a.com"))
            .unwrap();
        assert!(!path.exists());
        drop(s);
        for file in sidecars(&path) {
            assert!(!file.exists(), "reapareció {}", file.display());
        }
        cleanup(lib, dir);
    }

    #[test]
    fn ids_are_not_reused() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        lib.delete(2).unwrap();
        assert_eq!(lib.create("Trabajo", "TR").unwrap().id, 3);
        cleanup(lib, dir);
    }

    /// Bookmark in `profile` with every field filled in and the given status.
    fn full_bookmark(
        lib: &Library,
        profile: i64,
        status: Status,
        error: Option<&BookmarkError>,
    ) -> Bookmark {
        let store = lib.store(profile).unwrap();
        let s = lock(&store).unwrap();
        let b = s
            .add(NewBookmark {
                url: "https://a.com/x".into(),
                title: Some("Título".into()),
                notes: "mis notas".into(),
                tags: vec!["rust".into(), "db".into()],
                content: Some("contenido".into()),
                ..Default::default()
            })
            .unwrap();
        s.apply_summary(
            b.id,
            &Summary {
                title: String::new(),
                summary: "resumen".into(),
                category: "programación".into(),
                tags: vec![],
            },
        )
        .unwrap();
        s.set_status(b.id, status, error).unwrap();
        s.conn
            .execute(
                "UPDATE bookmarks SET created_at = '2020-01-01T00:00:00Z' WHERE id = ?1",
                [b.id],
            )
            .unwrap();
        s.get(b.id).unwrap().unwrap()
    }

    #[test]
    fn move_copies_then_deletes() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        let original = full_bookmark(&lib, 1, Status::Ready, None);
        let moved = lib.move_bookmark(original.id, 1, 2).unwrap();
        let in_target = lock(&lib.store(2).unwrap())
            .unwrap()
            .get(moved.id)
            .unwrap()
            .unwrap();
        assert_eq!(in_target, moved);
        assert_eq!(moved.url, original.url);
        assert_eq!(moved.title, original.title);
        assert_eq!(moved.summary.as_deref(), Some("resumen"));
        assert_eq!(moved.category.as_deref(), Some("programación"));
        assert_eq!(moved.tags, original.tags);
        assert_eq!(moved.notes, original.notes);
        assert_eq!(moved.content, original.content);
        assert_eq!(moved.created_at, "2020-01-01T00:00:00Z");
        assert_eq!(moved.status, Status::PendingEmbed);
        assert_eq!(moved.embedding_id, None);
        assert_eq!(
            lock(&lib.store(1).unwrap())
                .unwrap()
                .get(original.id)
                .unwrap(),
            None
        );
        cleanup(lib, dir);
    }

    #[test]
    fn move_keeps_original_url_and_status() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        let raw = "https://A.com/x?utm_source=t#top";
        let id = {
            let store = lib.store(1).unwrap();
            let s = lock(&store).unwrap();
            let nb = NewBookmark {
                url: raw.into(),
                ..Default::default()
            };
            s.insert_imported(&nb, None).unwrap()
        };
        let moved = lib.move_bookmark(id, 1, 2).unwrap();
        assert_eq!(moved.original_url, raw);
        assert_eq!(moved.status, Status::Imported);
        cleanup(lib, dir);
    }

    #[test]
    fn move_keeps_non_ready_status() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        lib.create("Ocio", "OC").unwrap();
        let pending = full_bookmark(&lib, 1, Status::Pending, None);
        assert_eq!(
            lib.move_bookmark(pending.id, 1, 2).unwrap().status,
            Status::Pending
        );
        cleanup(lib, dir);
    }

    #[test]
    fn insert_copy_keeps_the_error() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        let err = BookmarkError(vec![
            ErrorPart::Msg(Msg::ChatFailed),
            ErrorPart::Raw {
                raw: "se rompió".into(),
            },
        ]);
        let failed = full_bookmark(&lib, 1, Status::Failed, Some(&err));
        let moved = lib.move_bookmark(failed.id, 1, 2).unwrap();
        assert_eq!(moved.status, Status::Failed);
        assert_eq!(moved.error, Some(err));
        cleanup(lib, dir);
    }

    #[test]
    fn move_refuses_a_url_already_in_the_target() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        let original = full_bookmark(&lib, 1, Status::Ready, None);
        lock(&lib.store(2).unwrap())
            .unwrap()
            .add(new_bookmark("https://a.com/x"))
            .unwrap();
        let err = err_of(lib.move_bookmark(original.id, 1, 2));
        assert_eq!(
            err,
            Msg::UrlInProfile {
                profile: "Trabajo".into()
            }
        );
        assert_eq!(
            lock(&lib.store(1).unwrap())
                .unwrap()
                .get(original.id)
                .unwrap(),
            Some(original)
        );
        cleanup(lib, dir);
    }

    #[test]
    fn move_to_the_same_profile_is_refused() {
        let (lib, dir) = temp_library();
        let original = full_bookmark(&lib, 1, Status::Ready, None);
        assert_eq!(
            err_of(lib.move_bookmark(original.id, 1, 1)),
            Msg::SameProfile
        );
        assert_eq!(
            lock(&lib.store(1).unwrap())
                .unwrap()
                .get(original.id)
                .unwrap(),
            Some(original)
        );
        cleanup(lib, dir);
    }

    #[test]
    fn move_into_a_profile_deleted_mid_move_keeps_the_source() {
        // Simulates a `move_bookmark(b, 1, 2)` that got profile 2's store right before `delete(2)`.
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        let original = full_bookmark(&lib, 1, Status::Ready, None);
        let source = lib.store(1).unwrap();
        let target = lib.store(2).unwrap();
        lib.delete(2).unwrap();
        assert_eq!(
            err_of(lib.move_between(original.id, &source, 2, "Trabajo", &target)),
            Msg::ProfileNotFound
        );
        assert_eq!(
            lock(&source).unwrap().get(original.id).unwrap(),
            Some(original)
        );
        drop((source, target));
        cleanup(lib, dir);
    }

    #[test]
    fn move_of_a_missing_bookmark_fails() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        assert_eq!(err_of(lib.move_bookmark(42, 1, 2)), Msg::BookmarkNotFound);
        assert_eq!(err_of(lib.move_bookmark(42, 1, 99)), Msg::ProfileNotFound);
        cleanup(lib, dir);
    }

    #[test]
    fn token_resolves_to_its_profile() {
        let (lib, dir) = temp_library();
        let p = lib.create("Trabajo", "TR").unwrap();
        let old = lib.token(2).unwrap();
        assert_eq!(lib.profile_for_token(&old).unwrap(), Some(p.clone()));
        assert_eq!(lib.profile_for_token("nope").unwrap(), None);
        assert_eq!(lib.profile_for_token("").unwrap(), None);
        let new = lib.regenerate_token(2).unwrap();
        assert_ne!(new, old);
        assert_eq!(lib.token(2).unwrap(), new);
        assert_eq!(lib.profile_for_token(&old).unwrap(), None);
        assert_eq!(lib.profile_for_token(&new).unwrap(), Some(p));
        assert_eq!(err_of(lib.token(99)), Msg::ProfileNotFound);
        assert_eq!(err_of(lib.regenerate_token(99)), Msg::ProfileNotFound);
        cleanup(lib, dir);
    }

    #[test]
    fn requeue_covers_every_profile() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        for (id, urls) in [
            (1, &["https://a.com"][..]),
            (2, &["https://b.com", "https://c.com"]),
        ] {
            let store = lib.store(id).unwrap();
            let s = lock(&store).unwrap();
            s.set_setting(EMBEDDING_ID_KEY, "viejo").unwrap();
            for url in urls {
                let b = s.add(new_bookmark(url)).unwrap();
                s.set_status(b.id, Status::Ready, None).unwrap();
            }
        }
        assert_eq!(lib.requeue_if_embedding_changed("nuevo").unwrap(), 3);
        assert_eq!(lib.requeue_if_embedding_changed("nuevo").unwrap(), 0);
        cleanup(lib, dir);
    }

    #[test]
    fn move_keeps_metadata_error_kind_and_favicon() {
        let (lib, dir) = temp_library();
        lib.create("Trabajo", "TR").unwrap();
        let b = {
            let store = lib.store(1).unwrap();
            let s = lock(&store).unwrap();
            let b = s
                .add(NewBookmark {
                    meta: crate::extract::PageMeta {
                        author: Some("Ana".into()),
                        image: Some("https://a.com/i.png".into()),
                        favicon: Some("https://a.com/f.ico".into()),
                        word_count: Some(7),
                        ..Default::default()
                    },
                    ..new_bookmark("https://a.com")
                })
                .unwrap();
            s.set_blocked(b.id).unwrap();
            let favicon = crate::download::Image {
                mime: "image/x-icon".into(),
                bytes: vec![0, 0, 1, 0],
            };
            s.set_favicon(&b, &Some(favicon)).unwrap();
            s.get(b.id).unwrap().unwrap()
        };
        let favicon = lock(&lib.store(1).unwrap())
            .unwrap()
            .stored_favicon(b.id)
            .unwrap();
        assert!(favicon.is_some());
        let moved = lib.move_bookmark(b.id, 1, 2).unwrap();
        assert_eq!(moved.meta, b.meta);
        assert_eq!(moved.error_kind, b.error_kind);
        assert!(moved.error_kind.is_some());
        assert_eq!(moved.favicon_at, b.favicon_at);
        let target = lib.store(2).unwrap();
        assert_eq!(
            lock(&target).unwrap().stored_favicon(moved.id).unwrap(),
            favicon
        );
        cleanup(lib, dir);
    }
}
