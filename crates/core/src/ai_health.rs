//! Whether the summary model works: off, ready, or failed (it could not be built, or its last call
//! failed). Session state only: it starts as whatever `built` says and follows each call.

use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use crate::ai::{Model, SummarizeInput, Summary};
use crate::msg::{ErrorPart, parts};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum AiStatus {
    #[default]
    Off,
    Ready {
        model: String,
    },
    /// `model: None` when it could not be built; `Some` when it was, and its last call failed.
    Failed {
        model: Option<String>,
        error: Vec<ErrorPart>,
    },
}

impl AiStatus {
    /// The state right after building the model from settings `enabled` and `model`.
    pub fn after_build(enabled: bool, model: &str, built: Result<(), &anyhow::Error>) -> Self {
        match (enabled, built) {
            (false, _) => Self::Off,
            (true, Ok(())) => Self::Ready {
                model: model.to_string(),
            },
            (true, Err(e)) => Self::Failed {
                model: None,
                error: parts(e),
            },
        }
    }

    /// The state after a call; `None` if calls do not move this state.
    fn after_call(&self, outcome: Result<(), &anyhow::Error>) -> Option<Self> {
        let model = match self {
            Self::Ready { model }
            | Self::Failed {
                model: Some(model), ..
            } => model.clone(),
            Self::Off | Self::Failed { model: None, .. } => return None,
        };
        Some(match outcome {
            Ok(()) => Self::Ready { model },
            Err(e) => Self::Failed {
                model: Some(model),
                error: parts(e),
            },
        })
    }
}

type Notice = Box<dyn Fn() + Send + Sync>;

/// The status with the generation of the model it belongs to, and a notice run on each change.
#[derive(Default)]
pub struct AiHealth {
    state: Mutex<(u64, AiStatus)>,
    notice: OnceLock<Notice>,
}

impl AiHealth {
    /// Sets the status of a freshly built model; returns its generation, for `Watched`.
    pub fn built(&self, status: AiStatus) -> u64 {
        let (generation, changed) = {
            let mut state = self.lock();
            state.0 += 1;
            let changed = state.1 != status;
            state.1 = status;
            (state.0, changed)
        };
        if changed {
            self.notify();
        }
        generation
    }

    /// Notes a call's outcome. Ignored if `generation` is not the current model's.
    pub fn record(&self, generation: u64, outcome: Result<(), &anyhow::Error>) {
        let changed = {
            let mut state = self.lock();
            if state.0 != generation {
                return;
            }
            match state.1.after_call(outcome) {
                Some(next) if next != state.1 => {
                    state.1 = next;
                    true
                }
                _ => false,
            }
        };
        if changed {
            self.notify();
        }
    }

    pub fn status(&self) -> AiStatus {
        self.lock().1.clone()
    }

    /// The notice for each change, run with the lock released. Only the first call sets it.
    pub fn on_change(&self, f: impl Fn() + Send + Sync + 'static) {
        let _ = self.notice.set(Box::new(f));
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, (u64, AiStatus)> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn notify(&self) {
        if let Some(f) = self.notice.get() {
            f();
        }
    }
}

/// A `Model` that notes the outcome of each call on `health`.
pub struct Watched {
    inner: Arc<dyn Model>,
    health: Arc<AiHealth>,
    generation: u64,
}

impl Watched {
    pub fn new(inner: Arc<dyn Model>, health: Arc<AiHealth>, generation: u64) -> Self {
        Self {
            inner,
            health,
            generation,
        }
    }

    fn note<T>(&self, r: anyhow::Result<T>) -> anyhow::Result<T> {
        self.health.record(self.generation, r.as_ref().map(drop));
        r
    }
}

#[async_trait::async_trait]
impl Model for Watched {
    async fn summarize(&self, input: SummarizeInput<'_>) -> anyhow::Result<Summary> {
        self.note(self.inner.summarize(input).await)
    }

    async fn embed(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        self.note(self.inner.embed(texts).await)
    }

    async fn extract_markdown(&self, input: &str) -> anyhow::Result<String> {
        self.note(self.inner.extract_markdown(input).await)
    }

    fn embedding_id(&self) -> String {
        self.inner.embedding_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::FakeModel;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    const MODEL: &str = "qwen3:8b";

    fn ready(model: &str) -> AiStatus {
        AiStatus::Ready {
            model: model.into(),
        }
    }

    #[test]
    fn after_build_decides_the_state() {
        let e = anyhow::anyhow!("boom");
        assert_eq!(AiStatus::after_build(false, MODEL, Ok(())), AiStatus::Off);
        assert_eq!(AiStatus::after_build(false, MODEL, Err(&e)), AiStatus::Off);
        assert_eq!(AiStatus::after_build(true, MODEL, Ok(())), ready(MODEL));
        assert_eq!(
            AiStatus::after_build(true, MODEL, Err(&e)),
            AiStatus::Failed {
                model: None,
                error: parts(&e)
            }
        );
    }

    #[test]
    fn a_failed_call_then_a_good_one() {
        let health = AiHealth::default();
        let g = health.built(ready(MODEL));
        let e = anyhow::anyhow!("down");
        health.record(g, Err(&e));
        assert_eq!(
            health.status(),
            AiStatus::Failed {
                model: Some(MODEL.into()),
                error: parts(&e)
            }
        );
        health.record(g, Ok(()));
        assert_eq!(health.status(), ready(MODEL));
    }

    #[test]
    fn calls_do_not_change_off_or_unbuilt() {
        let e = anyhow::anyhow!("x");
        let unbuilt = AiStatus::Failed {
            model: None,
            error: parts(&anyhow::anyhow!("no model")),
        };
        for status in [AiStatus::Off, unbuilt] {
            let health = AiHealth::default();
            let g = health.built(status.clone());
            health.record(g, Err(&e));
            health.record(g, Ok(()));
            assert_eq!(health.status(), status);
        }
    }

    #[test]
    fn a_call_from_an_old_model_is_ignored() {
        let health = AiHealth::default();
        let old = health.built(ready("a"));
        health.built(ready("b"));
        health.record(old, Err(&anyhow::anyhow!("late")));
        assert_eq!(health.status(), ready("b"));
    }

    #[test]
    fn repeated_failures_notify_once() {
        let health = AiHealth::default();
        let count = Arc::new(AtomicUsize::new(0));
        let seen = count.clone();
        health.on_change(move || {
            seen.fetch_add(1, Ordering::SeqCst);
        });
        let g = health.built(ready(MODEL));
        assert_eq!(count.load(Ordering::SeqCst), 1);
        health.record(g, Err(&anyhow::anyhow!("same")));
        assert_eq!(count.load(Ordering::SeqCst), 2);
        health.record(g, Err(&anyhow::anyhow!("same")));
        assert_eq!(count.load(Ordering::SeqCst), 2);
        health.record(g, Ok(()));
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn the_notice_runs_without_the_lock() {
        let health = Arc::new(AiHealth::default());
        let inner = health.clone();
        let seen = Arc::new(Mutex::new(None));
        let out = seen.clone();
        health.on_change(move || {
            *out.lock().unwrap() = Some(inner.status());
        });
        health.built(ready(MODEL));
        assert_eq!(*seen.lock().unwrap(), Some(ready(MODEL)));
    }

    /// Fails all three calls while `fail` is set.
    struct Flaky {
        fake: FakeModel,
        fail: AtomicBool,
    }

    #[async_trait::async_trait]
    impl Model for Flaky {
        async fn summarize(&self, input: SummarizeInput<'_>) -> anyhow::Result<Summary> {
            anyhow::ensure!(!self.fail.load(Ordering::SeqCst), "summarize");
            self.fake.summarize(input).await
        }

        async fn embed(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
            anyhow::ensure!(!self.fail.load(Ordering::SeqCst), "embed");
            self.fake.embed(texts).await
        }

        async fn extract_markdown(&self, input: &str) -> anyhow::Result<String> {
            anyhow::ensure!(!self.fail.load(Ordering::SeqCst), "extract");
            self.fake.extract_markdown(input).await
        }

        fn embedding_id(&self) -> String {
            self.fake.embedding_id()
        }
    }

    #[tokio::test]
    async fn watched_records_each_call() {
        let flaky = Arc::new(Flaky {
            fake: FakeModel::default(),
            fail: AtomicBool::new(false),
        });
        let health = Arc::new(AiHealth::default());
        let g = health.built(ready(MODEL));
        let watched = Watched::new(flaky.clone(), health.clone(), g);
        let input = SummarizeInput {
            url: "https://a.com",
            title: "A",
            markdown: "md",
            existing_categories: &[],
            existing_tags: &[],
        };
        let failed = |health: &AiHealth| matches!(health.status(), AiStatus::Failed { .. });

        flaky.fail.store(true, Ordering::SeqCst);
        assert!(watched.summarize(input).await.is_err());
        assert!(failed(&health));
        flaky.fail.store(false, Ordering::SeqCst);
        assert!(watched.summarize(input).await.is_ok());
        assert_eq!(health.status(), ready(MODEL));

        flaky.fail.store(true, Ordering::SeqCst);
        assert!(watched.embed(&["x".into()]).await.is_err());
        assert!(failed(&health));
        flaky.fail.store(false, Ordering::SeqCst);
        assert!(watched.embed(&["x".into()]).await.is_ok());
        assert_eq!(health.status(), ready(MODEL));

        flaky.fail.store(true, Ordering::SeqCst);
        assert!(watched.extract_markdown("x").await.is_err());
        assert!(failed(&health));
        flaky.fail.store(false, Ordering::SeqCst);
        assert!(watched.extract_markdown("x").await.is_ok());
        assert_eq!(health.status(), ready(MODEL));

        assert_eq!(watched.embedding_id(), flaky.fake.embedding_id());
    }
}
