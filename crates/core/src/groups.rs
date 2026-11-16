//! Tag groups: manual grouping of tags in the sidebar.

use crate::msg::Msg;
use crate::store::Store;
use anyhow::Result;
use rusqlite::{ErrorCode, params};
use serde::Serialize;

const SELECT_GROUPS: &str = "SELECT id, name FROM tag_groups ORDER BY id";
const SELECT_MEMBERS: &str = "SELECT group_id, tag FROM tag_group_members ORDER BY tag";
const INSERT_GROUP: &str = "INSERT INTO tag_groups (name) VALUES (?1)";
const RENAME_GROUP: &str = "UPDATE tag_groups SET name = ?2 WHERE id = ?1";
const DELETE_GROUP: &str = "DELETE FROM tag_groups WHERE id = ?1";
const SET_MEMBER: &str = "INSERT INTO tag_group_members (tag, group_id) VALUES (?1, ?2)
     ON CONFLICT(tag) DO UPDATE SET group_id = excluded.group_id";
const DELETE_MEMBER: &str = "DELETE FROM tag_group_members WHERE tag = ?1";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TagGroup {
    pub id: i64,
    pub name: String,
    pub tags: Vec<String>,
}

/// Trimmed name; empty is an error.
fn clean_name(name: &str) -> Result<&str> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Msg::GroupNameEmpty.err());
    }
    Ok(name)
}

/// Maps an SQLite constraint violation to the given message; anything else passes through.
fn constraint_as(e: rusqlite::Error, message: Msg) -> anyhow::Error {
    match e.sqlite_error_code() {
        Some(ErrorCode::ConstraintViolation) => message.err(),
        _ => e.into(),
    }
}

impl Store {
    /// Fails if another group already has that name, case-insensitively. SQLite's `NOCASE` only folds
    /// ASCII ("Diseño" and "DISEÑO" look different to it), so the comparison happens here.
    fn ensure_name_free(&self, name: &str, except: Option<i64>) -> Result<()> {
        let wanted = name.to_lowercase();
        let mut stmt = self.conn.prepare(SELECT_GROUPS)?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        for row in rows {
            let (id, existing) = row?;
            if Some(id) != except && existing.to_lowercase() == wanted {
                return Err(Msg::GroupExists.err());
            }
        }
        Ok(())
    }

    /// Groups in creation order, each with its tags sorted alphabetically.
    pub fn tag_groups(&self) -> Result<Vec<TagGroup>> {
        let mut groups: Vec<TagGroup> = self
            .conn
            .prepare(SELECT_GROUPS)?
            .query_map([], |r| {
                Ok(TagGroup {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    tags: vec![],
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        let mut stmt = self.conn.prepare(SELECT_MEMBERS)?;
        let members = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        for member in members {
            let (group_id, tag) = member?;
            if let Some(group) = groups.iter_mut().find(|g| g.id == group_id) {
                group.tags.push(tag);
            }
        }
        Ok(groups)
    }

    pub fn create_tag_group(&self, name: &str) -> Result<TagGroup> {
        let name = clean_name(name)?;
        self.ensure_name_free(name, None)?;
        self.conn
            .execute(INSERT_GROUP, [name])
            .map_err(|e| constraint_as(e, Msg::GroupExists))?;
        Ok(TagGroup {
            id: self.conn.last_insert_rowid(),
            name: name.to_string(),
            tags: vec![],
        })
    }

    pub fn rename_tag_group(&self, id: i64, name: &str) -> Result<()> {
        let name = clean_name(name)?;
        self.ensure_name_free(name, Some(id))?;
        let changed = self
            .conn
            .execute(RENAME_GROUP, params![id, name])
            .map_err(|e| constraint_as(e, Msg::GroupExists))?;
        if changed == 0 {
            return Err(Msg::GroupNotFound.err());
        }
        Ok(())
    }

    /// `false` if the group did not exist. Its tags are left ungrouped (cascade delete).
    pub fn delete_tag_group(&self, id: i64) -> Result<bool> {
        Ok(self.conn.execute(DELETE_GROUP, [id])? > 0)
    }

    /// Moves the tag to the group; `None` removes it from its group.
    pub fn set_tag_group(&self, tag: &str, group_id: Option<i64>) -> Result<()> {
        match group_id {
            Some(id) => self
                .conn
                .execute(SET_MEMBER, params![tag, id])
                .map_err(|e| constraint_as(e, Msg::GroupNotFound))?,
            None => self.conn.execute(DELETE_MEMBER, [tag])?,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_lists_and_assigns() {
        let s = Store::open_in_memory().unwrap();
        let datos = s.create_tag_group("  Datos ").unwrap();
        assert_eq!(datos.name, "Datos");
        let front = s.create_tag_group("Frontend").unwrap();
        s.set_tag_group("sqlite", Some(datos.id)).unwrap();
        s.set_tag_group("fts5", Some(datos.id)).unwrap();
        s.set_tag_group("sqlite", Some(front.id)).unwrap(); // moved, not duplicated
        let groups = s.tag_groups().unwrap();
        assert_eq!(groups[0].tags, ["fts5"]);
        assert_eq!(groups[1].tags, ["sqlite"]);
        s.set_tag_group("sqlite", None).unwrap();
        assert!(s.tag_groups().unwrap()[1].tags.is_empty());
    }

    #[test]
    fn rejects_blank_and_duplicate_names() {
        let s = Store::open_in_memory().unwrap();
        let datos = s.create_tag_group("Datos").unwrap();
        let other = s.create_tag_group("Otro").unwrap();
        let err = |r: anyhow::Error| crate::msg::failure_msg(&r).cloned();
        assert_eq!(
            err(s.create_tag_group("   ").unwrap_err()),
            Some(Msg::GroupNameEmpty)
        );
        assert_eq!(
            err(s.create_tag_group(" datos ").unwrap_err()),
            Some(Msg::GroupExists)
        );
        assert_eq!(
            err(s.rename_tag_group(other.id, "DATOS").unwrap_err()),
            Some(Msg::GroupExists)
        );
        assert_eq!(
            err(s.rename_tag_group(other.id, "").unwrap_err()),
            Some(Msg::GroupNameEmpty)
        );
        assert_eq!(
            err(s.rename_tag_group(999, "X").unwrap_err()),
            Some(Msg::GroupNotFound)
        );
        s.rename_tag_group(datos.id, "datos").unwrap(); // same group, only the case changes
        let groups = s.tag_groups().unwrap();
        assert_eq!(groups[0].name, "datos");
        assert_eq!(groups[1].name, "Otro");
    }

    #[test]
    fn duplicate_names_ignore_non_ascii_case() {
        let s = Store::open_in_memory().unwrap();
        let design = s.create_tag_group("Diseño").unwrap();
        let other = s.create_tag_group("Árbol").unwrap();
        let err = |r: anyhow::Error| crate::msg::failure_msg(&r).cloned();
        assert_eq!(
            err(s.create_tag_group("DISEÑO").unwrap_err()),
            Some(Msg::GroupExists)
        );
        assert_eq!(
            err(s.rename_tag_group(design.id, "árbol").unwrap_err()),
            Some(Msg::GroupExists)
        );
        s.rename_tag_group(other.id, "ÁRBOL").unwrap(); // the group itself may change case
        assert_eq!(s.tag_groups().unwrap().len(), 2);
    }

    #[test]
    fn deleting_a_group_frees_its_tags() {
        let s = Store::open_in_memory().unwrap();
        let g = s.create_tag_group("Datos").unwrap();
        s.set_tag_group("sqlite", Some(g.id)).unwrap();
        assert!(s.delete_tag_group(g.id).unwrap());
        assert!(!s.delete_tag_group(g.id).unwrap());
        assert!(s.tag_groups().unwrap().is_empty());
        let again = s.create_tag_group("Datos").unwrap();
        s.set_tag_group("otra", Some(again.id)).unwrap();
        // The previous membership was cascade-deleted: `sqlite` does not come back.
        assert_eq!(s.tag_groups().unwrap()[0].tags, ["otra"]);
    }

    #[test]
    fn assigning_to_a_missing_group_fails() {
        let s = Store::open_in_memory().unwrap();
        let err = s.set_tag_group("sqlite", Some(7)).unwrap_err();
        assert_eq!(crate::msg::failure_msg(&err), Some(&Msg::GroupNotFound));
    }
}
