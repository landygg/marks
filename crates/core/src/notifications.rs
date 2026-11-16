//! In-memory notification log: groups by profile, severity and text, with a cap.
//! No I/O and no clock: the caller passes the time.

use crate::msg::{BookmarkError, ErrorPart, Msg};
use crate::store::{Bookmark, Status, Store};
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const MAX_NOTIFICATIONS: usize = 200;
pub const MAX_MESSAGE_CHARS: usize = 1_000;
pub const MAX_DETAIL_CHARS: usize = 300;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// What the notification points to; decides its actions in the UI.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Target {
    Bookmark { id: i64 },
    Ai,
    Extension,
    Updates,
}

/// What gets notified.
#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct NewNotification {
    /// `None`: global, visible in every profile.
    pub profile: Option<i64>,
    pub severity: Severity,
    pub message: String,
    /// Second line: title, domain or operation.
    pub detail: Option<String>,
    pub target: Option<Target>,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Notification {
    pub id: u64,
    pub profile: Option<i64>,
    pub severity: Severity,
    pub message: String,
    pub detail: Option<String>,
    pub target: Option<Target>,
    pub count: u32,
    /// Milliseconds since the UNIX epoch.
    pub last_at: u64,
    pub read: bool,
}

impl Notification {
    fn visible_in(&self, profile: i64) -> bool {
        self.profile.is_none_or(|p| p == profile)
    }
}

#[derive(Default)]
pub struct Notifications {
    /// Most recent first.
    entries: Vec<Notification>,
    next_id: u64,
    /// Message for a notification that arrives empty; the log's creator provides it (core does not translate).
    empty_message: String,
}

impl Notifications {
    pub fn new(empty_message: String) -> Self {
        Self {
            empty_message,
            ..Self::default()
        }
    }

    /// Adds the notification or, if an equal one exists (profile, severity and text), adds to that one
    /// and moves it to the top as unread.
    pub fn push(&mut self, new: NewNotification, now_ms: u64) -> Notification {
        self.push_with(new, now_ms, true)
    }

    /// Like `push`, but a repeat does not mark it unread again: for background failures that recur
    /// on every worker pass and should not ring the bell each time.
    pub fn push_quiet(&mut self, new: NewNotification, now_ms: u64) -> Notification {
        self.push_with(new, now_ms, false)
    }

    fn push_with(&mut self, new: NewNotification, now_ms: u64, unread_again: bool) -> Notification {
        let message =
            clean(&new.message, MAX_MESSAGE_CHARS).unwrap_or_else(|| self.empty_message.clone());
        let detail = new
            .detail
            .as_deref()
            .and_then(|d| clean(d, MAX_DETAIL_CHARS));
        let same = self.entries.iter().position(|n| {
            n.profile == new.profile && n.severity == new.severity && n.message == message
        });
        let entry = match same {
            Some(i) => {
                let mut n = self.entries.remove(i);
                n.count = n.count.saturating_add(1);
                n.last_at = now_ms;
                n.read = n.read && !unread_again;
                n.detail = detail;
                n.target = new.target;
                n
            }
            None => {
                self.next_id += 1;
                Notification {
                    id: self.next_id,
                    profile: new.profile,
                    severity: new.severity,
                    message,
                    detail,
                    target: new.target,
                    count: 1,
                    last_at: now_ms,
                    read: false,
                }
            }
        };
        self.entries.insert(0, entry.clone());
        if self.entries.len() > MAX_NOTIFICATIONS {
            // Drop the oldest profile one: global ones (AI, API) are few and shown everywhere.
            let oldest = self.entries.iter().rposition(|n| n.profile.is_some());
            self.entries.remove(oldest.unwrap_or(MAX_NOTIFICATIONS));
        }
        entry
    }

    /// The profile's notifications plus the global ones, most recent first.
    pub fn list(&self, profile: i64) -> Vec<Notification> {
        self.entries
            .iter()
            .filter(|n| n.visible_in(profile))
            .cloned()
            .collect()
    }

    pub fn mark_read(&mut self, profile: i64) {
        for n in self.entries.iter_mut().filter(|n| n.visible_in(profile)) {
            n.read = true;
        }
    }

    /// `true` if it existed.
    pub fn dismiss(&mut self, id: u64) -> bool {
        let before = self.entries.len();
        self.entries.retain(|n| n.id != id);
        self.entries.len() < before
    }

    /// Removes the profile's notifications and the global ones, which show in every profile.
    pub fn clear(&mut self, profile: i64) {
        self.entries.retain(|n| !n.visible_in(profile));
    }
}

/// If bookmark `id` ended up `failed`, the notification for its failure, with the error passed
/// through `render` (core does not translate). The caller takes the store.
pub fn bookmark_failure(
    store: &Store,
    profile: i64,
    id: i64,
    render: impl Fn(&BookmarkError) -> String,
) -> Result<Option<NewNotification>> {
    Ok(store
        .get(id)?
        .and_then(|b| failure_notice(&b, profile, render)))
}

fn failure_notice(
    b: &Bookmark,
    profile: i64,
    render: impl Fn(&BookmarkError) -> String,
) -> Option<NewNotification> {
    (b.status == Status::Failed).then(|| NewNotification {
        profile: Some(profile),
        severity: Severity::Error,
        message: match &b.error {
            Some(e) => render(e),
            None => render(&BookmarkError(vec![ErrorPart::Msg(Msg::NoContent)])),
        },
        detail: Some(
            if b.title.trim().is_empty() {
                &b.url
            } else {
                &b.title
            }
            .clone(),
        ),
        target: Some(Target::Bookmark { id: b.id }),
    })
}

/// Collapses runs of whitespace, trims and caps by characters; `None` if empty.
fn clean(text: &str, max: usize) -> Option<String> {
    let text: String = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max)
        .collect();
    let text = text.trim_end();
    (!text.is_empty()).then(|| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::NewBookmark;

    fn new(profile: Option<i64>, severity: Severity, message: &str) -> NewNotification {
        NewNotification {
            profile,
            severity,
            message: message.to_string(),
            detail: None,
            target: None,
        }
    }

    fn error(profile: i64, message: &str) -> NewNotification {
        new(Some(profile), Severity::Error, message)
    }

    #[test]
    fn push_adds_unread_at_the_head_with_new_ids() {
        let mut log = Notifications::default();
        log.push(error(1, "a"), 10);
        log.push(error(1, "b"), 20);
        let list = log.list(1);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].message, "b");
        assert_eq!((list[0].id, list[1].id), (2, 1));
        assert!(list.iter().all(|n| n.count == 1 && !n.read));
    }

    #[test]
    fn push_groups_by_profile_severity_and_message() {
        let mut log = Notifications::default();
        let first = log.push(error(1, "x"), 10);
        let mut again = error(1, "x");
        again.detail = Some("segundo".into());
        again.target = Some(Target::Bookmark { id: 7 });
        log.push(again, 20);
        let list = log.list(1);
        assert_eq!(list.len(), 1);
        let n = &list[0];
        assert_eq!((n.id, n.count, n.last_at), (first.id, 2, 20));
        assert_eq!(n.detail.as_deref(), Some("segundo"));
        assert_eq!(n.target, Some(Target::Bookmark { id: 7 }));
    }

    #[test]
    fn grouping_unreads_and_moves_to_the_head() {
        let mut log = Notifications::default();
        log.push(error(1, "A"), 10);
        log.push(error(1, "B"), 20);
        log.mark_read(1);
        log.push(error(1, "A"), 30);
        let list = log.list(1);
        assert_eq!(list[0].message, "A");
        assert!(!list[0].read);
        assert_eq!(list[1].message, "B");
        assert!(list[1].read);
    }

    #[test]
    fn different_profile_or_severity_does_not_group() {
        let mut log = Notifications::default();
        log.push(new(Some(1), Severity::Error, "x"), 1);
        log.push(new(Some(2), Severity::Error, "x"), 2);
        log.push(new(Some(1), Severity::Warning, "x"), 3);
        log.push(new(None, Severity::Error, "x"), 4);
        // Profile 1: its Error, its Warning and the global one; profile 2: its own and the global one.
        assert_eq!((log.list(1).len(), log.list(2).len()), (3, 2));
        assert!(log.list(1).iter().chain(&log.list(2)).all(|n| n.count == 1));
    }

    #[test]
    fn list_shows_the_profile_and_globals_only() {
        let mut log = Notifications::default();
        log.push(error(1, "uno"), 1);
        log.push(error(2, "dos"), 2);
        log.push(new(None, Severity::Warning, "global"), 3);
        let profiles = |p| log.list(p).iter().map(|n| n.profile).collect::<Vec<_>>();
        assert_eq!(profiles(1), [None, Some(1)]);
        assert_eq!(profiles(2), [None, Some(2)]);
    }

    #[test]
    fn push_caps_at_max_dropping_the_oldest() {
        let mut log = Notifications::default();
        for i in 0..=MAX_NOTIFICATIONS {
            log.push(error(1, &format!("m{i}")), i as u64);
        }
        let list = log.list(1);
        assert_eq!(list.len(), MAX_NOTIFICATIONS);
        assert!(list.iter().all(|n| n.message != "m0"));
    }

    #[test]
    fn the_cap_drops_profile_entries_before_global_ones() {
        let mut log = Notifications::default();
        log.push(new(None, Severity::Warning, "global"), 0);
        for i in 0..MAX_NOTIFICATIONS {
            log.push(error(1, &format!("m{i}")), 1 + i as u64);
        }
        let list = log.list(1);
        assert_eq!(list.len(), MAX_NOTIFICATIONS);
        assert!(list.iter().any(|n| n.message == "global"));
        assert!(list.iter().all(|n| n.message != "m0"));
    }

    #[test]
    fn a_quiet_repeat_counts_but_stays_read() {
        let mut log = Notifications::default();
        log.push_quiet(error(1, "almacén"), 1);
        log.mark_read(1);
        let again = log.push_quiet(error(1, "almacén"), 2);
        assert_eq!((again.count, again.read, again.last_at), (2, true, 2));
        // A new failure is still news.
        assert!(!log.push_quiet(error(1, "otro"), 3).read);
        // An ordinary repeat marks it unread, as before.
        assert!(!log.push(error(1, "almacén"), 4).read);
    }

    #[test]
    fn push_bounds_and_cleans_the_text() {
        let mut log = Notifications::new("vacía".into());
        assert_eq!(log.push(error(1, "  a \n  b "), 1).message, "a b");
        let long = log.push(error(1, &"ñ".repeat(1_500)), 2);
        assert_eq!(long.message.chars().count(), MAX_MESSAGE_CHARS);
        assert_eq!(log.push(error(1, "   "), 3).message, "vacía");
        let mut with_detail = error(1, "d");
        with_detail.detail = Some("x".repeat(400));
        let detail = log.push(with_detail, 4).detail.unwrap_or_default();
        assert_eq!(detail.chars().count(), MAX_DETAIL_CHARS);
        let mut blank_detail = error(1, "e");
        blank_detail.detail = Some("  ".into());
        assert_eq!(log.push(blank_detail, 5).detail, None);
    }

    #[test]
    fn mark_read_and_clear_touch_only_what_is_visible() {
        let mut log = Notifications::default();
        log.push(error(1, "uno"), 1);
        log.push(error(2, "dos"), 2);
        log.push(new(None, Severity::Info, "global"), 3);
        log.mark_read(1);
        assert!(log.list(1).iter().all(|n| n.read));
        let two = log.list(2);
        assert!(two.iter().any(|n| n.profile == Some(2) && !n.read));
        log.clear(1);
        assert!(log.list(1).is_empty());
        let two = log.list(2);
        assert_eq!(two.len(), 1);
        assert_eq!(two[0].profile, Some(2));
    }

    #[test]
    fn dismiss_removes_by_id() {
        let mut log = Notifications::default();
        let n = log.push(error(1, "x"), 1);
        assert!(log.dismiss(n.id));
        assert!(!log.dismiss(n.id));
        assert!(log.list(1).is_empty());
    }

    #[test]
    fn bookmark_failure_only_for_failed() {
        let store = Store::open_in_memory().unwrap();
        let add = |url: &str, title: Option<&str>| {
            store
                .add(NewBookmark {
                    url: url.into(),
                    title: title.map(Into::into),
                    ..Default::default()
                })
                .unwrap()
        };

        let failed = add("https://example.com/a", Some("T"));
        let render = |e: &BookmarkError| e.to_string();
        store
            .set_failed(
                failed.id,
                &BookmarkError(vec![ErrorPart::Raw { raw: "boom".into() }]),
            )
            .unwrap();
        let n = bookmark_failure(&store, 4, failed.id, render)
            .unwrap()
            .unwrap();
        assert_eq!(n.severity, Severity::Error);
        assert_eq!(n.message, "boom");
        assert_eq!(n.detail.as_deref(), Some("T"));
        assert_eq!(n.target, Some(Target::Bookmark { id: failed.id }));
        assert_eq!(n.profile, Some(4));

        let ready = add("https://example.com/b", Some("R"));
        store.set_status(ready.id, Status::Ready, None).unwrap();
        assert_eq!(bookmark_failure(&store, 4, ready.id, render).unwrap(), None);
        assert_eq!(bookmark_failure(&store, 4, 999, render).unwrap(), None);

        // The store never keeps an empty title; the rule is tested on the bookmark as read.
        let bare = add("https://example.com/c", None);
        store.set_status(bare.id, Status::Failed, None).unwrap();
        let mut bare = store.get(bare.id).unwrap().unwrap();
        bare.title = String::new();
        let n = failure_notice(&bare, 4, render).unwrap();
        assert_eq!(n.message, Msg::NoContent.to_string());
        assert_eq!(n.detail.as_deref(), Some(bare.url.as_str()));
    }

    #[test]
    fn serde_shapes_match_the_frontend() {
        use serde_json::{json, to_value};
        assert_eq!(
            to_value(Target::Bookmark { id: 3 }).unwrap(),
            json!({"kind": "bookmark", "id": 3})
        );
        assert_eq!(to_value(Target::Ai).unwrap(), json!({"kind": "ai"}));
        assert_eq!(to_value(Severity::Warning).unwrap(), json!("warning"));
        let back: NewNotification = serde_json::from_value(json!({
            "profile": null, "severity": "info", "message": "m", "detail": null,
            "target": {"kind": "extension"}
        }))
        .unwrap();
        assert_eq!(back.target, Some(Target::Extension));
    }

    #[test]
    fn updates_target_serializes_as_kind() {
        assert_eq!(
            serde_json::to_value(Target::Updates).unwrap(),
            serde_json::json!({ "kind": "updates" })
        );
    }
}
