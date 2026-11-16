//! Imports a browser's `bookmarks.html` into a profile (preview, then apply), with folders as
//! groups and tags, and exports a profile to one. The format lives in `netscape`.

use crate::capture::ALLOWED_SCHEMES;
use crate::msg::{Msg, MsgExt};
use crate::netscape::{self, Entry, MARKS_FOLDER, Marker, Node};
use crate::store::{ExportRow, NewBookmark, Store, normalize_tags};
use anyhow::Result;
use rusqlite::{Transaction, TransactionBehavior};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Read;
use std::path::Path;
use url::Url;

/// Cap on the file to import, so an arbitrary one is not loaded into memory.
pub const MAX_FILE_MB: u64 = 50;
pub const MAX_FILE_BYTES: u64 = MAX_FILE_MB * 1024 * 1024;

/// Import counts. `existing` includes URLs repeated within the file itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct ImportSummary {
    pub new: usize,
    pub existing: usize,
    pub unsupported: usize,
    pub invalid: usize,
}

const BOM: char = '\u{feff}';

/// An importable item (http or https), new or already seen.
struct Item {
    /// Normalized, for deduplication.
    url: String,
    /// As it appears in the file.
    href: String,
    title: String,
    add_date: Option<i64>,
    tags: Vec<String>,
    /// `(tag, group)` pairs, from the folder path only.
    groups: Vec<(String, String)>,
}

/// What was read and counted, with nothing written; `apply` writes it.
pub struct Preview {
    pub summary: ImportSummary,
    items: Vec<Item>,
}

/// The whole file as text, without BOM. Fails if it exceeds `MAX_FILE_BYTES` or is not UTF-8.
pub fn read_file(path: &Path) -> Result<String> {
    let failed = || Msg::ImportRead {
        path: path.display().to_string(),
    };
    let file = std::fs::File::open(path).msg(failed())?;
    if file.metadata().msg(failed())?.len() > MAX_FILE_BYTES {
        return Err(Msg::FileTooLarge {
            max_mb: MAX_FILE_MB,
        }
        .err());
    }
    // Cap the read too: a special file may report size 0 and never end.
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .msg(failed())?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(Msg::FileTooLarge {
            max_mb: MAX_FILE_MB,
        }
        .err());
    }
    let mut text = String::from_utf8(bytes).map_err(|_| Msg::NotUtf8.err())?;
    if text.starts_with(BOM) {
        text.drain(..BOM.len_utf8());
    }
    Ok(text)
}

/// Tags from a folder path, each with its group if any. `parents` are the first names of paths
/// two or more levels deep, lowercased, spelled as they first appeared.
fn path_tags(path: &[String], parents: &HashMap<String, String>) -> Vec<(String, Option<String>)> {
    match path {
        [] => vec![],
        [only] => vec![(only.clone(), parents.get(&only.to_lowercase()).cloned())],
        [group, rest @ ..] => rest
            .iter()
            .map(|name| (name.clone(), Some(group.clone())))
            .collect(),
    }
}

fn importable(href: &str) -> bool {
    Url::parse(href).is_ok_and(|u| ALLOWED_SCHEMES.contains(&u.scheme()))
}

/// Classifies and counts each item without writing: new, already in the database or earlier in
/// the file, unsupported scheme, or invalid URL.
pub fn preview(store: &Store, entries: Vec<Entry>, reading_list_tag: &str) -> Result<Preview> {
    let mut parents: HashMap<String, String> = HashMap::new();
    // Only subfolders with something importable count: bookmarklets yield no group.
    for e in entries
        .iter()
        .filter(|e| e.path.len() >= 2 && importable(&e.href))
    {
        parents
            .entry(e.path[0].to_lowercase())
            .or_insert_with(|| e.path[0].clone());
    }
    let mut summary = ImportSummary::default();
    let mut seen = HashSet::new();
    let mut items = Vec::new();
    for e in entries {
        let Ok(parsed) = Url::parse(&e.href) else {
            summary.invalid += 1;
            continue;
        };
        if !ALLOWED_SCHEMES.contains(&parsed.scheme()) {
            summary.unsupported += 1;
            continue;
        }
        let url = crate::normalize_url(&e.href)?;
        if seen.insert(url.clone()) && store.find_by_url(&url)?.is_none() {
            summary.new += 1;
        } else {
            summary.existing += 1;
        }
        let mut tags = Vec::new();
        let mut groups = Vec::new();
        for (name, group) in path_tags(&e.path, &parents) {
            for tag in normalize_tags(&[name]) {
                if let Some(group) = &group {
                    groups.push((tag.clone(), group.clone()));
                }
                tags.push(tag);
            }
        }
        if e.reading_list {
            tags.push(reading_list_tag.to_string());
        }
        tags.extend(e.tags);
        items.push(Item {
            url,
            href: e.href,
            title: e.title,
            add_date: e.add_date,
            tags: normalize_tags(&tags),
            groups,
        });
    }
    Ok(Preview { summary, items })
}

/// Writes the preview in one transaction and recounts new and existing items, in case the
/// database changed since the preview. Existing bookmarks only get their missing tags. Only tags
/// that did not exist get a group, and the file's first assignment wins.
pub fn apply(store: &Store, preview: &Preview) -> Result<ImportSummary> {
    let tx = Transaction::new_unchecked(&store.conn, TransactionBehavior::Immediate)?;
    let mut before: HashSet<String> = store.tags()?.into_iter().collect();
    let mut group_ids = HashMap::new();
    for g in store.tag_groups()? {
        before.extend(g.tags);
        group_ids.insert(g.name.to_lowercase(), g.id);
    }
    let mut assigned = HashSet::new();
    let mut summary = ImportSummary {
        unsupported: preview.summary.unsupported,
        invalid: preview.summary.invalid,
        ..ImportSummary::default()
    };
    for item in &preview.items {
        match store.find_by_url(&item.url)? {
            None => {
                let new = NewBookmark {
                    url: item.href.clone(),
                    title: Some(item.title.clone()),
                    tags: item.tags.clone(),
                    ..NewBookmark::default()
                };
                store.insert_imported(&new, item.add_date)?;
                summary.new += 1;
            }
            Some(existing) => {
                if item.tags.iter().any(|t| !existing.tags.contains(t)) {
                    let mut tags = existing.tags;
                    tags.extend(item.tags.iter().cloned());
                    store.set_tags_only(existing.id, &tags)?;
                }
                summary.existing += 1;
            }
        }
        for (tag, group) in &item.groups {
            if before.contains(tag) || assigned.contains(tag) {
                continue;
            }
            let key = group.to_lowercase();
            let id = match group_ids.get(&key) {
                Some(id) => *id,
                None => {
                    let id = store.create_tag_group(group)?.id;
                    group_ids.insert(key, id);
                    id
                }
            };
            store.set_tag_group(tag, Some(id))?;
            assigned.insert(tag.clone());
        }
    }
    tx.commit()?;
    Ok(summary)
}

fn folder(name: &str, marker: Option<Marker>, children: Vec<Node>) -> Node {
    Node::Folder {
        name: name.to_string(),
        marker,
        children,
    }
}

/// `marks/<group>/<tag>/`, ungrouped tags in `marks/<tag>/` and untagged bookmarks in
/// `marks/<untagged_folder>/`. Groups in creation order, tags in byte order (accented letters
/// sort after `z`) and bookmarks oldest first. No empty folders.
pub fn export_html(store: &Store, untagged_folder: &str) -> Result<String> {
    let rows = store.export_rows()?;
    let link = |r: &ExportRow| Node::Link {
        href: r.url.clone(),
        title: r.title.clone(),
        add_date: r.add_date,
        tags: r.tags.clone(),
    };
    let mut by_tag: BTreeMap<&str, Vec<Node>> = BTreeMap::new();
    let mut untagged = Vec::new();
    for r in &rows {
        if r.tags.is_empty() {
            untagged.push(link(r));
        }
        for tag in &r.tags {
            by_tag.entry(tag).or_default().push(link(r));
        }
    }
    let mut root = Vec::new();
    for g in store.tag_groups()? {
        let tags: Vec<Node> = g
            .tags
            .iter()
            .filter_map(|t| {
                by_tag
                    .remove(t.as_str())
                    .map(|links| folder(t, None, links))
            })
            .collect();
        if !tags.is_empty() {
            root.push(folder(&g.name, None, tags));
        }
    }
    root.extend(by_tag.into_iter().map(|(t, links)| folder(t, None, links)));
    if !untagged.is_empty() {
        root.push(folder(untagged_folder, Some(Marker::Untagged), untagged));
    }
    let nodes = if root.is_empty() {
        vec![]
    } else {
        vec![folder(MARKS_FOLDER, Some(Marker::Root), root)]
    };
    Ok(netscape::render(&nodes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNTAGGED: &str = "Sin etiqueta";
    const READING_LIST_TAG: &str = "lista de lectura";
    use crate::netscape;
    use crate::normalize_url;
    use crate::store::{Bookmark, NewBookmark, Status};
    use std::collections::HashSet;

    const CHROME: &str = include_str!("../fixtures/bookmarks-chrome.html");
    const SAFARI: &str = include_str!("../fixtures/bookmarks-safari.html");
    const READING_LIST: &str = include_str!("../fixtures/readinglist-safari.html");

    fn entry(href: &str, path: &[&str]) -> Entry {
        Entry {
            href: href.to_string(),
            title: String::new(),
            add_date: None,
            path: path.iter().map(|s| s.to_string()).collect(),
            reading_list: false,
            tags: vec![],
        }
    }

    fn import(s: &Store, entries: Vec<Entry>) -> ImportSummary {
        apply(s, &preview(s, entries, READING_LIST_TAG).unwrap()).unwrap()
    }

    fn import_html(s: &Store, html: &str) -> ImportSummary {
        import(s, netscape::parse(html, &[UNTAGGED.to_string()]))
    }

    fn summary(new: usize, existing: usize, unsupported: usize, invalid: usize) -> ImportSummary {
        ImportSummary {
            new,
            existing,
            unsupported,
            invalid,
        }
    }

    fn bookmark(s: &Store, url: &str) -> Bookmark {
        s.find_by_url(&normalize_url(url).unwrap())
            .unwrap()
            .unwrap()
    }

    fn tags_of(s: &Store, url: &str) -> Vec<String> {
        bookmark(s, url).tags
    }

    fn groups(s: &Store) -> Vec<(String, Vec<String>)> {
        s.tag_groups()
            .unwrap()
            .into_iter()
            .map(|g| (g.name, g.tags))
            .collect()
    }

    fn group(name: &str, tags: &[&str]) -> (String, Vec<String>) {
        (
            name.to_string(),
            tags.iter().map(|t| t.to_string()).collect(),
        )
    }

    fn new(url: &str, tags: &[&str]) -> NewBookmark {
        NewBookmark {
            url: url.to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn read_file_rejects_large_and_non_utf8() {
        let dir = std::env::temp_dir().join(format!("marks-transfer-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let big = dir.join("big.html");
        // Sparse: does not write 50 MB to disk.
        std::fs::File::create(&big)
            .unwrap()
            .set_len(MAX_FILE_BYTES + 1)
            .unwrap();
        assert_eq!(
            crate::msg::failure_msg(&read_file(&big).unwrap_err()),
            Some(&Msg::FileTooLarge {
                max_mb: MAX_FILE_MB
            })
        );
        let bad = dir.join("bad.html");
        std::fs::write(&bad, [0xff, 0xfe, 0x00]).unwrap();
        assert_eq!(
            crate::msg::failure_msg(&read_file(&bad).unwrap_err()),
            Some(&Msg::NotUtf8)
        );
        let bom = dir.join("bom.html");
        std::fs::write(&bom, "\u{feff}<DL><p>").unwrap();
        assert_eq!(read_file(&bom).unwrap(), "<DL><p>");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn read_file_names_a_missing_file() {
        let path = std::env::temp_dir().join(format!("marks-missing-{}.html", std::process::id()));
        let parts = crate::msg::parts(&read_file(&path).unwrap_err());
        assert_eq!(
            parts.first(),
            Some(&crate::msg::ErrorPart::Msg(Msg::ImportRead {
                path: path.display().to_string()
            }))
        );
        assert_eq!(parts.len(), 2, "{parts:?}");
    }

    #[test]
    fn preview_counts_without_writing() {
        let s = Store::open_in_memory().unwrap();
        s.add(new("https://ya.com/", &[])).unwrap();
        let total = s.stats().unwrap().total;
        let p = preview(
            &s,
            vec![
                entry("https://a.com/", &[]),
                entry("https://b.com/", &["G", "x"]),
                entry("https://ya.com/", &[]),
                entry("https://a.com/", &[]),
                entry("javascript:alert(1)", &[]),
                entry("no es url", &[]),
            ],
            READING_LIST_TAG,
        )
        .unwrap();
        assert_eq!(p.summary, summary(2, 2, 1, 1));
        assert_eq!(s.stats().unwrap().total, total);
        assert!(s.tag_groups().unwrap().is_empty());
    }

    #[test]
    fn folders_become_groups_and_tags() {
        let s = Store::open_in_memory().unwrap();
        let paths: [&[&str]; 5] = [
            &[],
            &["Solo"],
            &["Dev"],
            &["Dev", "Rust"],
            &["Lang", "Rust", "Async"],
        ];
        let entries = paths
            .iter()
            .enumerate()
            .map(|(i, p)| entry(&format!("https://e.com/{i}"), p))
            .collect();
        assert_eq!(import(&s, entries), summary(5, 0, 0, 0));
        let expected: [&[&str]; 5] = [&[], &["solo"], &["dev"], &["rust"], &["rust", "async"]];
        for (i, tags) in expected.iter().enumerate() {
            assert_eq!(tags_of(&s, &format!("https://e.com/{i}")), *tags);
        }
        assert_eq!(
            groups(&s),
            [group("Dev", &["dev", "rust"]), group("Lang", &["async"])]
        );
    }

    #[test]
    fn top_level_folder_with_subfolders_groups_its_own_tag() {
        let s = Store::open_in_memory().unwrap();
        import(
            &s,
            vec![
                entry("https://e.com/0", &["dev"]),
                entry("https://e.com/1", &["DEV", "Rust"]),
            ],
        );
        assert_eq!(groups(&s), [group("DEV", &["dev", "rust"])]);
    }

    #[test]
    fn subfolders_without_importable_links_make_no_group() {
        let s = Store::open_in_memory().unwrap();
        import(
            &s,
            vec![
                entry("https://e.com/0", &["Dev"]),
                entry("javascript:alert(1)", &["Dev", "Bookmarklets"]),
                entry("no es url", &["Dev", "Rotos"]),
            ],
        );
        assert_eq!(tags_of(&s, "https://e.com/0"), ["dev"]);
        assert!(s.tag_groups().unwrap().is_empty());
    }

    #[test]
    fn existing_tags_keep_their_group_and_no_empty_groups() {
        let s = Store::open_in_memory().unwrap();
        s.add(new("https://go.dev/", &["go"])).unwrap();
        let mio = s.create_tag_group("Mío").unwrap();
        s.set_tag_group("rust", Some(mio.id)).unwrap();
        import(
            &s,
            vec![
                entry("https://e.com/1", &["Dev", "rust"]),
                entry("https://e.com/2", &["Dev", "go"]),
            ],
        );
        assert_eq!(groups(&s), [group("Mío", &["rust"])]);
        assert_eq!(tags_of(&s, "https://e.com/2"), ["go"]);
    }

    #[test]
    fn group_names_match_without_case() {
        let s = Store::open_in_memory().unwrap();
        s.create_tag_group("dev").unwrap();
        import(&s, vec![entry("https://e.com/1", &["DEV", "nuevo"])]);
        assert_eq!(groups(&s), [group("dev", &["nuevo"])]);
    }

    #[test]
    fn repeated_urls_merge_tags_and_touch_nothing_else() {
        let s = Store::open_in_memory().unwrap();
        let b = s
            .add(NewBookmark {
                title: Some("Mío".into()),
                notes: "nota".into(),
                ..new("https://x.com/p", &["a"])
            })
            .unwrap();
        s.set_status(b.id, Status::Ready, None).unwrap();
        // A fixed old date: if it were touched, it would surely change.
        s.conn
            .execute(
                "UPDATE bookmarks SET updated_at = '2001-01-01T00:00:00Z' WHERE id = ?1",
                [b.id],
            )
            .unwrap();
        let before = s.get(b.id).unwrap().unwrap();
        let mut first = entry("https://x.com/p?utm_source=n", &["B"]);
        first.title = "Otro título".into();
        first.add_date = Some(1_700_000_000);
        let got = import(&s, vec![first, entry("https://x.com/p", &["C"])]);
        assert_eq!(got, summary(0, 2, 0, 0));
        let after = s.get(b.id).unwrap().unwrap();
        assert_eq!(after.tags, ["a", "b", "c"]);
        assert_eq!(after.title, before.title);
        assert_eq!(after.notes, before.notes);
        assert_eq!(after.status, Status::Ready);
        assert_eq!(after.original_url, before.original_url);
        assert_eq!(after.created_at, before.created_at);
        assert_eq!(after.updated_at, "2001-01-01T00:00:00Z");
    }

    #[test]
    fn variants_of_one_url_import_once() {
        let s = Store::open_in_memory().unwrap();
        let got = import(
            &s,
            vec![
                entry("https://x.com/p#uno", &[]),
                entry("https://x.com/p?utm_source=z", &[]),
            ],
        );
        assert_eq!(got, summary(1, 1, 0, 0));
        assert_eq!(
            bookmark(&s, "https://x.com/p").original_url,
            "https://x.com/p#uno"
        );
    }

    #[test]
    fn reading_list_uses_the_given_tag() {
        let s = Store::open_in_memory().unwrap();
        let mut e = entry("https://e.com/", &[]);
        e.reading_list = true;
        apply(&s, &preview(&s, vec![e], "reading list").unwrap()).unwrap();
        assert_eq!(tags_of(&s, "https://e.com/"), ["reading list"]);
    }

    #[test]
    fn untagged_folder_is_recognised_in_every_locale_without_marker() {
        let source = sample();
        let html = export_html(&source, "Untagged")
            .unwrap()
            .replace(" MARKS=\"root\"", "")
            .replace(" MARKS=\"untagged\"", "");
        assert!(!html.contains("MARKS="));
        let names = ["Sin etiqueta".to_string(), "Untagged".to_string()];
        let other = Store::open_in_memory().unwrap();
        let p = preview(&other, netscape::parse(&html, &names), READING_LIST_TAG).unwrap();
        apply(&other, &p).unwrap();
        assert_eq!(snapshot(&other), snapshot(&source));
        assert!(!other.tags().unwrap().iter().any(|t| t == "untagged"));
    }

    #[test]
    fn reading_list_gets_its_tag() {
        let s = Store::open_in_memory().unwrap();
        let mut e = entry("https://e.com/", &[]);
        e.reading_list = true;
        import(&s, vec![e]);
        assert_eq!(tags_of(&s, "https://e.com/"), [READING_LIST_TAG]);
        assert!(s.tag_groups().unwrap().is_empty());
    }

    #[test]
    fn tags_attribute_adds_tags_without_group() {
        let s = Store::open_in_memory().unwrap();
        let mut e = entry("https://e.com/", &["G", "y"]);
        e.tags = vec!["x".into()];
        import(&s, vec![e]);
        assert_eq!(tags_of(&s, "https://e.com/"), ["y", "x"]);
        assert_eq!(groups(&s), [group("G", &["y"])]);
    }

    #[test]
    fn folder_names_with_commas_become_one_tag() {
        let s = Store::open_in_memory().unwrap();
        import(&s, vec![entry("https://e.com/", &["Viajes, 2024"])]);
        assert_eq!(tags_of(&s, "https://e.com/"), ["viajes 2024"]);
    }

    #[test]
    fn imported_keep_date_status_and_original_url() {
        let s = Store::open_in_memory().unwrap();
        let href = "https://Example.com/a?utm_source=x#frag";
        let mut e = entry(href, &[]);
        e.add_date = Some(1_700_000_000);
        import(&s, vec![e]);
        let b = bookmark(&s, href);
        assert_eq!(b.status, Status::Imported);
        assert_eq!(b.created_at, "2023-11-14T22:13:20Z");
        assert_eq!(b.original_url, href);
        assert_eq!(b.url, "https://example.com/a");
    }

    #[test]
    fn random_html_imports_nothing() {
        let s = Store::open_in_memory().unwrap();
        let got = import_html(&s, "<html><body><p>hola</p></body></html>");
        assert_eq!(got, ImportSummary::default());
    }

    #[test]
    fn imports_ten_thousand_entries() {
        let s = Store::open_in_memory().unwrap();
        let entries = (0..10_000)
            .map(|i| entry(&format!("https://e.com/{i}"), &[&format!("f{}", i % 100)]))
            .collect();
        assert_eq!(import(&s, entries).new, 10_000);
        assert_eq!(s.tags().unwrap().len(), 100);
    }

    /// Group `Dev` (`go`, `rust` and `vacía`, with no bookmarks), ungrouped `web`, and bookmarks in
    /// `imported`, `pending` and `failed`: one with `go`, one with `rust` and `web`, and one untagged.
    fn sample() -> Store {
        let s = Store::open_in_memory().unwrap();
        let dev = s.create_tag_group("Dev").unwrap();
        for tag in ["rust", "go", "vacía"] {
            s.set_tag_group(tag, Some(dev.id)).unwrap();
        }
        let go = NewBookmark {
            title: Some("Go & <amigos> \"ya\"".into()),
            ..new("https://go.dev/#/play", &["go"])
        };
        s.insert_imported(&go, Some(1_600_000_000)).unwrap();
        s.add(new("https://rust-lang.org/?utm_source=x", &["rust", "web"]))
            .unwrap();
        let none = s.add(new("https://sin.com/", &[])).unwrap();
        s.set_failed(none.id, &crate::msg::BookmarkError::raw("error"))
            .unwrap();
        s
    }

    #[test]
    fn export_puts_groups_tags_and_untagged_in_folders() {
        let html = export_html(&sample(), UNTAGGED).unwrap();
        let got: Vec<(String, Vec<String>)> = netscape::parse(&html, &[UNTAGGED.to_string()])
            .into_iter()
            .map(|e| (e.href, e.path))
            .collect();
        let row = |href: &str, path: &[&str]| {
            (
                href.to_string(),
                path.iter().map(|p| p.to_string()).collect::<Vec<_>>(),
            )
        };
        assert_eq!(
            got,
            [
                row("https://go.dev/#/play", &["Dev", "go"]),
                row("https://rust-lang.org/?utm_source=x", &["Dev", "rust"]),
                row("https://rust-lang.org/?utm_source=x", &["web"]),
                row("https://sin.com/", &[]),
            ]
        );
        assert_eq!(
            netscape::parse(&html, &[UNTAGGED.to_string()])[0].title,
            "Go & <amigos> \"ya\""
        );
        // marks, Dev, go, rust, web and Sin etiqueta: none empty (not even `vacía`).
        assert_eq!(html.matches("<H3").count(), 6);
    }

    /// Original URL, sorted tags and date of each bookmark, and the groups with the tags that have
    /// bookmarks (a tag without bookmarks is not exported).
    type Snapshot = (
        Vec<(String, Vec<String>, String)>,
        Vec<(String, Vec<String>)>,
    );

    fn snapshot(s: &Store) -> Snapshot {
        let mut rows: Vec<_> = s
            .list()
            .unwrap()
            .into_iter()
            .map(|b| {
                let mut tags = b.tags;
                tags.sort();
                (b.original_url, tags, b.created_at)
            })
            .collect();
        rows.sort();
        let used: HashSet<String> = s.tags().unwrap().into_iter().collect();
        let groups = groups(s)
            .into_iter()
            .map(|(name, tags)| {
                (
                    name,
                    tags.into_iter().filter(|t| used.contains(t)).collect(),
                )
            })
            .collect();
        (rows, groups)
    }

    #[test]
    fn export_round_trips_into_an_empty_profile() {
        let s = sample();
        let html = export_html(&s, UNTAGGED).unwrap();
        // As after a round trip through a browser: no markers, inside its "Imported" folder.
        let stripped = html
            .replace(" MARKS=\"root\"", "")
            .replace(" MARKS=\"untagged\"", "");
        assert_ne!(stripped, html);
        let wrapped = format!("<DT><H3>Importado</H3><DL><p>{stripped}</DL><p>");
        for text in [&html, &stripped, &wrapped] {
            let other = Store::open_in_memory().unwrap();
            assert_eq!(import_html(&other, text), summary(3, 1, 0, 0));
            assert_eq!(snapshot(&other), snapshot(&s));
        }
    }

    #[test]
    fn chrome_fixture_imports() {
        let s = Store::open_in_memory().unwrap();
        assert_eq!(import_html(&s, CHROME), summary(3, 0, 0, 0));
        assert!(s.tags().unwrap().is_empty());
        assert_eq!(
            bookmark(&s, "https://agent.example.dev/").created_at,
            "2026-01-01T00:00:00Z"
        );
    }

    #[test]
    fn safari_fixture_imports() {
        let s = Store::open_in_memory().unwrap();
        assert_eq!(import_html(&s, SAFARI), summary(66, 0, 0, 0));
        assert_eq!(
            groups(&s),
            [
                group("Dev", &["dev", "python", "rust"]),
                group("Clients", &["acme"]),
                group("Courses", &["genai", "starter"]),
            ]
        );
        let grouped: HashSet<String> = groups(&s).into_iter().flat_map(|(_, t)| t).collect();
        let loose: Vec<String> = s
            .tags()
            .unwrap()
            .into_iter()
            .filter(|t| !grouped.contains(t))
            .collect();
        assert_eq!(loose, ["ai", "devops", "personal", "ui"]);
        // Favourites is the only root with bookmarks: the empty-path ones are its 12.
        let roots: Vec<Entry> = netscape::parse(SAFARI, &[UNTAGGED.to_string()])
            .into_iter()
            .filter(|e| e.path.is_empty())
            .collect();
        assert_eq!(roots.len(), 12);
        for e in &roots {
            assert!(tags_of(&s, &e.href).is_empty(), "{}", e.href);
        }
        assert_eq!(
            tags_of(&s, "https://academy.example.com/learn/foundations/lessons"),
            ["genai", "starter"]
        );
        let fragment = bookmark(&s, "https://site07.example/#xxxxxxxx");
        assert_eq!(fragment.original_url, "https://site07.example/#xxxxxxxx");
        assert_eq!(fragment.url, "https://site07.example/");
    }

    #[test]
    fn chrome_then_safari_merges_the_shared_url() {
        let s = Store::open_in_memory().unwrap();
        import_html(&s, CHROME);
        assert_eq!(import_html(&s, SAFARI), summary(65, 1, 0, 0));
        assert_eq!(tags_of(&s, "https://design-notes.example/"), ["ai"]);
    }

    #[test]
    fn safari_reading_list_imports() {
        let s = Store::open_in_memory().unwrap();
        assert_eq!(import_html(&s, READING_LIST), summary(2, 0, 0, 0));
        for b in s.list().unwrap() {
            assert_eq!(b.tags, [READING_LIST_TAG]);
        }
        assert!(s.tag_groups().unwrap().is_empty());
    }
}
