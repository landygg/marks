//! Chunk vector index with sqlite-vec (`vec_chunks.rowid = chunks.id`).

use crate::msg::{Msg, MsgExt};
use crate::store::{Store, TOUCH};
use anyhow::Result;
use rusqlite::{OptionalExtension, Transaction, TransactionBehavior, params};
use zerocopy::IntoBytes;

/// Setting with the id of the embedding model the index was built with.
pub const EMBEDDING_ID_KEY: &str = "embedding_id";
/// Setting with the dimension of the vectors in `vec_chunks`.
pub const EMBEDDING_DIM_KEY: &str = "embedding_dim";

const VEC_TABLE_EXISTS: &str =
    "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'vec_chunks')";
const DROP_VEC_TABLE: &str = "DROP TABLE IF EXISTS vec_chunks";
const DELETE_ALL_CHUNKS: &str = "DELETE FROM chunks";
const DELETE_SETTING: &str = "DELETE FROM settings WHERE key = ?1";
const DELETE_BOOKMARK_VECTORS: &str =
    "DELETE FROM vec_chunks WHERE rowid IN (SELECT id FROM chunks WHERE bookmark_id = ?1)";
const DELETE_BOOKMARK_CHUNKS: &str = "DELETE FROM chunks WHERE bookmark_id = ?1";
const INSERT_CHUNK: &str = "INSERT INTO chunks (bookmark_id, ord, text) VALUES (?1, ?2, ?3)";
const INSERT_VECTOR: &str = "INSERT INTO vec_chunks (rowid, embedding) VALUES (?1, ?2)";
const SELECT_CHUNK_TEXT: &str = "SELECT text FROM chunks WHERE id = ?1";
/// The `LIMIT` goes inside so sqlite-vec runs the KNN before the join.
const KNN: &str = "SELECT c.bookmark_id, c.id, v.distance
     FROM (SELECT rowid, distance FROM vec_chunks
           WHERE embedding MATCH ?1 ORDER BY distance LIMIT ?2) v
     JOIN chunks c ON c.id = v.rowid
     ORDER BY v.distance";

#[derive(Debug, Clone, PartialEq)]
pub struct KnnHit {
    pub bookmark_id: i64,
    pub chunk_id: i64,
    pub distance: f64,
}

impl Store {
    /// Prepares `vec_chunks` for the given model. If the stored model or dimension differ, deletes
    /// vectors and chunks, requeues `ready` bookmarks as `pending_embed` and returns `true`.
    /// With no stored dimension (after `requeue_if_embedding_changed`) it only creates the table.
    pub fn ensure_vectors(&self, embedding_id: &str, dim: usize) -> Result<bool> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let stored_id = self.get_setting(EMBEDDING_ID_KEY)?;
        let stored_dim = self.stored_dim()?;
        let changed = stored_id.is_some()
            && (stored_id.as_deref() != Some(embedding_id) || stored_dim.is_some_and(|d| d != dim));
        if changed {
            self.reset_vectors()?;
        }
        self.set_setting(EMBEDDING_ID_KEY, embedding_id)?;
        self.set_setting(EMBEDDING_DIM_KEY, &dim.to_string())?;
        self.conn.execute_batch(&format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS vec_chunks USING vec0(embedding float[{dim}])"
        ))?;
        tx.commit()?;
        Ok(changed)
    }

    /// If the index belongs to another model, drops it and requeues `ready` bookmarks without waiting
    /// for the first embedding (the dimension is not known yet). Returns how many were requeued.
    pub fn requeue_if_embedding_changed(&self, embedding_id: &str) -> Result<usize> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let stored_id = self.get_setting(EMBEDDING_ID_KEY)?;
        if stored_id.is_none_or(|id| id == embedding_id) {
            return Ok(0);
        }
        let moved = self.reset_vectors()?;
        self.set_setting(EMBEDDING_ID_KEY, embedding_id)?;
        self.conn.execute(DELETE_SETTING, [EMBEDDING_DIM_KEY])?;
        tx.commit()?;
        Ok(moved)
    }

    /// Deletes the bookmark's vectors (not its chunks). No transaction of its own.
    pub(crate) fn delete_bookmark_vectors(&self, bookmark_id: i64) -> Result<()> {
        if self.vec_table_exists()? {
            self.conn.execute(DELETE_BOOKMARK_VECTORS, [bookmark_id])?;
        }
        Ok(())
    }

    /// Replaces the bookmark's chunks and vectors and assigns it the index's model.
    pub fn replace_chunks(
        &self,
        bookmark_id: i64,
        chunks: &[String],
        vectors: &[Vec<f32>],
    ) -> Result<()> {
        if chunks.len() != vectors.len() {
            return Err(Msg::VectorsLenMismatch.err());
        }
        // Reads happen inside the transaction: another process could switch models.
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let dim = self.stored_dim()?.msg(Msg::VectorsNoIndex)?;
        if vectors.iter().any(|v| v.len() != dim) {
            return Err(Msg::VectorsDimMismatch.err());
        }
        let embedding_id = self
            .get_setting(EMBEDDING_ID_KEY)?
            .msg(Msg::VectorsNoIndex)?;
        tx.execute(DELETE_BOOKMARK_VECTORS, [bookmark_id])?;
        tx.execute(DELETE_BOOKMARK_CHUNKS, [bookmark_id])?;
        for (ord, (text, vector)) in chunks.iter().zip(vectors).enumerate() {
            tx.execute(INSERT_CHUNK, params![bookmark_id, ord as i64, text])?;
            tx.execute(
                INSERT_VECTOR,
                params![tx.last_insert_rowid(), vector.as_bytes()],
            )?;
        }
        tx.execute(
            &format!("UPDATE bookmarks SET embedding_id = ?1, {TOUCH} WHERE id = ?2"),
            params![embedding_id, bookmark_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// The `k` chunks closest to `query`. Empty if the index does not exist yet.
    pub fn knn(&self, query: &[f32], k: usize) -> Result<Vec<KnnHit>> {
        if k == 0 || !self.vec_table_exists()? {
            return Ok(Vec::new());
        }
        if self.stored_dim()? != Some(query.len()) {
            return Err(Msg::VectorsDimMismatch.err());
        }
        let mut stmt = self.conn.prepare(KNN)?;
        let rows = stmt.query_map(params![query.as_bytes(), k as i64], |r| {
            Ok(KnnHit {
                bookmark_id: r.get(0)?,
                chunk_id: r.get(1)?,
                distance: r.get(2)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn chunk_text(&self, chunk_id: i64) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(SELECT_CHUNK_TEXT, [chunk_id], |r| r.get(0))
            .optional()?)
    }

    fn stored_dim(&self) -> Result<Option<usize>> {
        self.get_setting(EMBEDDING_DIM_KEY)?
            .map(|d| d.parse().msg(Msg::VectorsBadDim))
            .transpose()
    }

    fn vec_table_exists(&self) -> Result<bool> {
        Ok(self.conn.query_row(VEC_TABLE_EXISTS, [], |r| r.get(0))?)
    }

    /// Deletes vectors and chunks and requeues `ready` bookmarks; returns how many. No transaction of its own.
    fn reset_vectors(&self) -> Result<usize> {
        self.conn.execute_batch(DROP_VEC_TABLE)?;
        self.conn.execute(DELETE_ALL_CHUNKS, [])?;
        self.mark_ready_pending_embed()
    }
}

#[cfg(test)]
mod tests {
    use super::{EMBEDDING_DIM_KEY, EMBEDDING_ID_KEY};
    use crate::store::{NewBookmark, Status, Store};

    const MODEL_A: &str = "ollama/a";
    const MODEL_B: &str = "ollama/b";
    const DIM: usize = 3;

    fn ready_bookmark(s: &Store, url: &str) -> i64 {
        let b = s
            .add(NewBookmark {
                url: url.into(),
                title: None,
                notes: String::new(),
                tags: vec![],
                content: None,
                ..Default::default()
            })
            .unwrap();
        s.set_status(b.id, Status::Ready, None).unwrap();
        b.id
    }

    fn texts(items: &[&str]) -> Vec<String> {
        items.iter().map(|t| t.to_string()).collect()
    }

    fn rows(s: &Store, table: &str) -> i64 {
        s.conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    fn indexed_store() -> (Store, i64, i64) {
        let s = Store::open_in_memory().unwrap();
        assert!(!s.ensure_vectors(MODEL_A, DIM).unwrap());
        let a = ready_bookmark(&s, "https://a.com");
        let b = ready_bookmark(&s, "https://b.com");
        s.replace_chunks(
            a,
            &texts(&["a0", "a1"]),
            &[vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]],
        )
        .unwrap();
        s.replace_chunks(b, &texts(&["b0"]), &[vec![0.0, 0.0, 1.0]])
            .unwrap();
        (s, a, b)
    }

    #[test]
    fn knn_returns_nearest_first() {
        let (s, a, b) = indexed_store();
        let hits = s.knn(&[0.0, 0.1, 0.9], 2).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].bookmark_id, b);
        assert_eq!(hits[1].bookmark_id, a);
        assert!(hits[0].distance <= hits[1].distance);
        assert_eq!(
            s.chunk_text(hits[0].chunk_id).unwrap().as_deref(),
            Some("b0")
        );
        assert_eq!(
            s.chunk_text(hits[1].chunk_id).unwrap().as_deref(),
            Some("a1")
        );
        assert!(s.knn(&[1.0, 0.0], 2).is_err());
        assert!(s.knn(&[0.0, 0.1, 0.9], 0).unwrap().is_empty());
    }

    #[test]
    fn replace_chunks_overwrites_previous() {
        let (s, a, _) = indexed_store();
        s.replace_chunks(a, &texts(&["nuevo"]), &[vec![0.0, 0.0, 1.0]])
            .unwrap();
        let hits = s.knn(&[0.0, 0.0, 1.0], 10).unwrap();
        let of_a: Vec<_> = hits.iter().filter(|h| h.bookmark_id == a).collect();
        assert_eq!(of_a.len(), 1);
        assert_eq!(rows(&s, "vec_chunks"), 2);
        assert_eq!(
            s.chunk_text(of_a[0].chunk_id).unwrap().as_deref(),
            Some("nuevo")
        );
        assert_eq!(
            s.get(a).unwrap().unwrap().embedding_id.as_deref(),
            Some(MODEL_A)
        );
        assert!(s.replace_chunks(a, &texts(&["x"]), &[]).is_err());
        assert!(s.replace_chunks(a, &texts(&["x"]), &[vec![1.0]]).is_err());
    }

    #[test]
    fn changing_embedding_id_resets_even_with_same_dim() {
        let (s, a, b) = indexed_store();
        assert!(s.ensure_vectors(MODEL_B, DIM).unwrap());
        assert!(s.knn(&[1.0, 0.0, 0.0], 10).unwrap().is_empty());
        assert_eq!(rows(&s, "chunks"), 0);
        for id in [a, b] {
            assert_eq!(s.get(id).unwrap().unwrap().status, Status::PendingEmbed);
        }
        assert_eq!(
            s.get_setting(EMBEDDING_ID_KEY).unwrap().as_deref(),
            Some(MODEL_B)
        );
    }

    #[test]
    fn same_embedding_id_keeps_vectors() {
        let (s, a, _) = indexed_store();
        assert!(!s.ensure_vectors(MODEL_A, DIM).unwrap());
        assert_eq!(s.knn(&[1.0, 0.0, 0.0], 1).unwrap()[0].bookmark_id, a);
        assert_eq!(s.get(a).unwrap().unwrap().status, Status::Ready);
    }

    #[test]
    fn deleting_a_bookmark_removes_its_vectors_and_frees_rowids() {
        let (s, a, b) = indexed_store();
        assert!(s.delete(b).unwrap());
        let c = ready_bookmark(&s, "https://c.com");
        s.replace_chunks(
            c,
            &texts(&["c0", "c1"]),
            &[vec![0.0, 0.0, 1.0], vec![0.0, 1.0, 1.0]],
        )
        .unwrap();
        let hits = s.knn(&[0.0, 0.0, 1.0], 10).unwrap();
        assert!(!hits.is_empty());
        assert!(
            hits.iter()
                .all(|h| h.bookmark_id == a || h.bookmark_id == c)
        );
        assert_eq!(rows(&s, "vec_chunks"), rows(&s, "chunks"));
    }

    #[test]
    fn delete_without_vector_index_still_works() {
        let s = Store::open_in_memory().unwrap();
        let a = ready_bookmark(&s, "https://a.com");
        assert!(s.delete(a).unwrap());
    }

    #[test]
    fn requeue_if_embedding_changed_resets_index_without_dim() {
        let (s, a, b) = indexed_store();
        assert_eq!(s.requeue_if_embedding_changed(MODEL_A).unwrap(), 0);
        assert_eq!(s.get(a).unwrap().unwrap().status, Status::Ready);

        assert_eq!(s.requeue_if_embedding_changed(MODEL_B).unwrap(), 2);
        assert_eq!(rows(&s, "chunks"), 0);
        assert!(s.knn(&[1.0, 0.0, 0.0], 10).unwrap().is_empty());
        for id in [a, b] {
            assert_eq!(s.get(id).unwrap().unwrap().status, Status::PendingEmbed);
        }
        assert_eq!(
            s.get_setting(EMBEDDING_ID_KEY).unwrap().as_deref(),
            Some(MODEL_B)
        );
        assert_eq!(s.get_setting(EMBEDDING_DIM_KEY).unwrap(), None);

        // With the id already stored and no dimension, ensure_vectors creates the table without requeuing.
        assert!(!s.ensure_vectors(MODEL_B, DIM).unwrap());
        s.replace_chunks(a, &texts(&["a0"]), &[vec![1.0, 0.0, 0.0]])
            .unwrap();
        assert_eq!(s.knn(&[1.0, 0.0, 0.0], 1).unwrap()[0].bookmark_id, a);
        assert_eq!(s.get(b).unwrap().unwrap().status, Status::PendingEmbed);
    }

    #[test]
    fn requeue_if_embedding_changed_ignores_fresh_store() {
        let s = Store::open_in_memory().unwrap();
        ready_bookmark(&s, "https://a.com");
        assert_eq!(s.requeue_if_embedding_changed(MODEL_A).unwrap(), 0);
        assert_eq!(s.get_setting(EMBEDDING_ID_KEY).unwrap(), None);
    }
}
