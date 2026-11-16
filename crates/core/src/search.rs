//! Hybrid search: FTS5 + vector KNN merged with Reciprocal Rank Fusion.

use crate::ai::Model;
use crate::store::{Bookmark, Store, lock};
use crate::vectors::{EMBEDDING_ID_KEY, KnnHit};
use anyhow::Result;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// RRF smoothing constant.
pub const RRF_K: f64 = 60.0;
/// Candidates each ranking (FTS and KNN) contributes before merging.
const CANDIDATES: usize = 50;

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub bookmark: Bookmark,
    pub score: f64,
    pub snippet: Option<String>,
}

/// Merges id rankings: each appearance adds `1 / (k + position)`, position from 1.
/// Sorted by descending score; ties by ascending id.
pub fn rrf(rankings: &[Vec<i64>], k: f64) -> Vec<(i64, f64)> {
    let mut scores: HashMap<i64, f64> = HashMap::new();
    for ranking in rankings {
        for (pos, id) in ranking.iter().enumerate() {
            *scores.entry(*id).or_default() += 1.0 / (k + (pos + 1) as f64);
        }
    }
    let mut merged: Vec<(i64, f64)> = scores.into_iter().collect();
    merged.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    merged
}

/// FTS + KNN with RRF. The model is queried outside the lock; if it fails, or the index belongs
/// to another embedding model (incomparable spaces), FTS only.
pub async fn hybrid(
    store: &Mutex<Store>,
    model: Option<&dyn Model>,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchHit>> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    let indexed_id = lock(store)?.get_setting(EMBEDDING_ID_KEY)?;
    let query_vector = match model {
        Some(model) if indexed_id == Some(model.embedding_id()) => embed_query(model, query).await,
        _ => None,
    };
    let guard = lock(store)?;
    rank(&guard, query, query_vector.as_deref(), limit)
}

async fn embed_query(model: &dyn Model, query: &str) -> Option<Vec<f32>> {
    model
        .embed(&[query.to_string()])
        .await
        .ok()
        .and_then(|vectors| vectors.into_iter().next())
}

fn rank(
    store: &Store,
    query: &str,
    query_vector: Option<&[f32]>,
    limit: usize,
) -> Result<Vec<SearchHit>> {
    let fts: Vec<i64> = store
        .search(query, CANDIDATES)?
        .into_iter()
        .map(|b| b.id)
        .collect();
    let knn = query_vector
        .and_then(|v| store.knn(v, CANDIDATES).ok())
        .map(best_chunk_per_bookmark)
        .unwrap_or_default();
    let knn_ids: Vec<i64> = knn.iter().map(|h| h.bookmark_id).collect();
    let mut hits = Vec::new();
    for (id, score) in rrf(&[fts, knn_ids], RRF_K).into_iter().take(limit) {
        let Some(bookmark) = store.get(id)? else {
            continue;
        };
        let snippet = match knn.iter().find(|h| h.bookmark_id == id) {
            Some(hit) => store.chunk_text(hit.chunk_id)?,
            None => None,
        };
        hits.push(SearchHit {
            bookmark,
            score,
            snippet,
        });
    }
    Ok(hits)
}

/// Keeps the first (closest) chunk of each bookmark, preserving distance order.
fn best_chunk_per_bookmark(hits: Vec<KnnHit>) -> Vec<KnnHit> {
    let mut seen = HashSet::new();
    hits.into_iter()
        .filter(|h| seen.insert(h.bookmark_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{FakeModel, SummarizeInput, Summary};
    use crate::store::{NewBookmark, Store};
    use std::sync::Mutex;

    const DIM: usize = 16;
    const LIMIT: usize = 10;

    fn add(s: &Store, url: &str, title: &str) -> i64 {
        s.add(NewBookmark {
            url: url.into(),
            title: Some(title.into()),
            notes: String::new(),
            tags: vec![],
            content: None,
            ..Default::default()
        })
        .unwrap()
        .id
    }

    async fn index(s: &Store, model: &FakeModel, id: i64, chunk: &str) {
        let chunks = vec![chunk.to_string()];
        let vectors = model.embed(&chunks).await.unwrap();
        s.replace_chunks(id, &chunks, &vectors).unwrap();
    }

    /// Tauri requires `Send` futures: this fails to compile if a guard crosses an `.await`.
    fn assert_send<T: Send>(_: &T) {}

    #[test]
    fn rrf_merges_two_rankings() {
        let k = RRF_K;
        let merged = rrf(&[vec![1, 2, 3], vec![3, 1]], k);
        let ids: Vec<i64> = merged.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, vec![1, 3, 2]);
        let expected = [
            1.0 / (k + 1.0) + 1.0 / (k + 2.0),
            1.0 / (k + 3.0) + 1.0 / (k + 1.0),
            1.0 / (k + 2.0),
        ];
        for ((_, score), want) in merged.iter().zip(expected) {
            assert!((score - want).abs() < 1e-12);
        }
    }

    #[tokio::test]
    async fn hybrid_falls_back_to_fts_without_model() {
        let s = Store::open_in_memory().unwrap();
        let id = add(&s, "https://rust.com", "Aprender rust");
        add(&s, "https://otro.com", "Cocina italiana");
        let store = Mutex::new(s);
        let failing = FakeModel {
            dim: DIM,
            fail_embed: true,
            ..FakeModel::default()
        };
        store
            .lock()
            .unwrap()
            .ensure_vectors(&failing.embedding_id(), DIM)
            .unwrap();
        for model in [None, Some(&failing as &dyn Model)] {
            let hits = hybrid(&store, model, "rust", LIMIT).await.unwrap();
            assert_eq!(hits.len(), 1);
            assert_eq!(hits[0].bookmark.id, id);
            assert!(hits[0].snippet.is_none());
        }
        assert!(hybrid(&store, None, "  ", LIMIT).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn hybrid_finds_semantic_match_with_fake_model() {
        let model = FakeModel {
            dim: DIM,
            ..FakeModel::default()
        };
        let s = Store::open_in_memory().unwrap();
        s.ensure_vectors(&model.embedding_id(), DIM).unwrap();
        let a = add(&s, "https://a.com", "Recetas de cocina");
        let b = add(&s, "https://b.com", "Sin coincidencias");
        index(&s, &model, a, "pasta fresca con tomate").await;
        let chunk = "gatos negros durmiendo al sol";
        index(&s, &model, b, chunk).await;
        let store = Mutex::new(s);
        assert!(
            store
                .lock()
                .unwrap()
                .search(chunk, LIMIT)
                .unwrap()
                .is_empty()
        );
        let future = hybrid(&store, Some(&model), chunk, LIMIT);
        assert_send(&future);
        let hits = future.await.unwrap();
        assert_eq!(hits[0].bookmark.id, b);
        assert_eq!(hits[0].snippet.as_deref(), Some(chunk));
    }

    #[tokio::test]
    async fn hybrid_skips_knn_when_embedding_id_differs() {
        let indexer = FakeModel {
            dim: DIM,
            ..FakeModel::default()
        };
        let s = Store::open_in_memory().unwrap();
        s.ensure_vectors(&indexer.embedding_id(), DIM).unwrap();
        let a = add(&s, "https://a.com", "Gatos negros");
        let b = add(&s, "https://b.com", "Sin coincidencias");
        index(&s, &indexer, a, "gatos negros").await;
        index(&s, &indexer, b, "gatos negros durmiendo al sol").await;
        let store = Mutex::new(s);
        let same = hybrid(&store, Some(&indexer), "durmiendo", LIMIT)
            .await
            .unwrap();
        assert_eq!(same[0].bookmark.id, b);
        // Same dimension, different model: KNN would not fail, but it would mix spaces.
        let other = Renamed(indexer);
        let hits = hybrid(&store, Some(&other), "gatos", LIMIT).await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].bookmark.id, a);
        assert!(hits[0].snippet.is_none());
        let hits = hybrid(&store, Some(&other), "durmiendo", LIMIT)
            .await
            .unwrap();
        assert!(hits.is_empty());
    }

    /// `FakeModel` with another `embedding_id` and the same dimension.
    struct Renamed(FakeModel);

    #[async_trait::async_trait]
    impl Model for Renamed {
        async fn summarize(&self, input: SummarizeInput<'_>) -> Result<Summary> {
            self.0.summarize(input).await
        }

        async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            self.0.embed(texts).await
        }

        async fn extract_markdown(&self, input: &str) -> Result<String> {
            self.0.extract_markdown(input).await
        }

        fn embedding_id(&self) -> String {
            format!("otro:{}", self.0.embedding_id())
        }
    }
}
