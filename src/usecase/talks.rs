use crate::model::talks::{Talk, TalkCommandStatus, Talks, TalksParams};
use crate::repo::talks::*;
use core::fmt::Debug;
use tracing::{debug, error, info, warn};

impl Debug for dyn TalkDisplayRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TalkDisplayRepo")
    }
}

impl Debug for dyn TalkOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TalkOperationRepo")
    }
}

impl Debug for dyn TalkCacheOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TalkCacheOperationRepo")
    }
}

/// TalkUseCase
/// The deep usecase for Talk reads and writes. HTTP handlers call only the
/// verbs exposed here; the database repos and the optional in-process cache
/// sit behind this interface.
///
/// Read-through rules:
/// - Detail reads hit the cache first, fall through to the database, and
///   refill the cache on a database hit.
/// - List reads are all-or-nothing: the cache answers only when every Talk
///   expected for the query is present, otherwise the read falls to the
///   database and refills the cache with the returned database rows.
///
/// Write verbs own their cache choreography: `add`, `update`, and `delete`
/// each refresh or drop the Talk cache. Cache disabling is resolved at
/// construction, so the verbs make no per-callsite decision about caching.
///
/// Every verb keeps the repo layer's `Option` convention: `None` means the
/// operation failed. Handlers render an error page from that, and no
/// partial-success distinction is re-derived here.
#[derive(Clone)]
pub struct TalkUseCase {
    display_repo: Box<dyn TalkDisplayRepo + Send + Sync>,
    operation_repo: Box<dyn TalkOperationRepo + Send + Sync>,
    /// Optional in-process cache. `None` means caching is disabled.
    cache: Option<TalkCache>,
}

#[derive(Clone)]
struct TalkCache {
    display: Box<dyn TalkDisplayRepo + Send + Sync>,
    ops: Box<dyn TalkCacheOperationRepo + Send + Sync>,
}

impl Debug for TalkUseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TalkUseCase")
    }
}

impl TalkUseCase {
    /// Create a TalkUseCase without a cache.
    pub fn new(
        display_repo: Box<dyn TalkDisplayRepo + Send + Sync>,
        operation_repo: Box<dyn TalkOperationRepo + Send + Sync>,
    ) -> Self {
        Self {
            display_repo,
            operation_repo,
            cache: None,
        }
    }

    /// Attach the in-process cache adapters.
    pub fn with_cache(
        mut self,
        cache_display: Box<dyn TalkDisplayRepo + Send + Sync>,
        cache_ops: Box<dyn TalkCacheOperationRepo + Send + Sync>,
    ) -> Self {
        self.cache = Some(TalkCache {
            display: cache_display,
            ops: cache_ops,
        });
        self
    }

    /// Read a single Talk through the cache, refilling on a database hit.
    pub async fn find(&mut self, id: i64) -> Option<Talk> {
        if let Some(cache) = self.cache.as_ref() {
            if let Some(talk) = cache.display.find(id).await {
                return Some(talk);
            }
        }

        let db_result = self.display_repo.find(id).await;
        if let Some(talk) = db_result.clone() {
            if let Some(cache) = self.cache.as_mut() {
                let _ = cache.ops.insert(talk).await;
            }
        }
        db_result
    }

    /// Read a list of Talks through the cache with all-or-nothing semantics.
    pub async fn find_talks(&mut self, params: TalksParams) -> Option<Talks> {
        if let Some(cache) = self.cache.as_ref() {
            let cache_result = cache.display.find_talks(params.clone()).await;
            if cache_result.is_some() {
                return cache_result;
            }
        }

        let db_result = self.display_repo.find_talks(params.clone()).await;
        if let Some(talks) = db_result.clone() {
            if let Some(cache) = self.cache.as_mut() {
                for talk in talks.talks {
                    debug!("Refilling talk cache {}", &talk.id);
                    let _ = cache.ops.insert(talk).await;
                }
            }
        }
        db_result
    }

    /// Next Talk id for the add form.
    pub async fn get_new_id(&mut self) -> Option<i64> {
        self.operation_repo.get_new_id().await
    }

    /// Add a Talk, keeping the cache in step.
    pub async fn add(&mut self, talk: Talk) -> Option<TalkCommandStatus> {
        let add_status = self
            .operation_repo
            .add(
                talk.id,
                talk.clone().name,
                talk.clone().date,
                talk.clone().media_link,
                talk.clone().org_name,
                talk.clone().org_link,
            )
            .await?;
        if add_status != TalkCommandStatus::Stored {
            error!(
                "Failed to add talk with Id {}: status is not Stored",
                &talk.id
            );
            return None;
        }

        if let Some(cache) = self.cache.as_mut() {
            debug!("Caching talk {}", &talk.id);
            let _ = cache.ops.insert(talk).await;
        }
        Some(TalkCommandStatus::Stored)
    }

    /// Update a Talk, keeping the cache in step.
    pub async fn update(&mut self, talk: Talk) -> Option<TalkCommandStatus> {
        let update_status = self
            .operation_repo
            .update(
                talk.id,
                Some(talk.clone().name),
                Some(talk.clone().date),
                talk.clone().media_link,
                talk.clone().org_name,
                talk.clone().org_link,
            )
            .await?;
        if update_status != TalkCommandStatus::Updated {
            error!(
                "Failed to update talk with Id {}: status is not Updated",
                &talk.id
            );
            return None;
        }

        if let Some(cache) = self.cache.as_mut() {
            debug!("Invalidating talk {} cache before re-inserting", &talk.id);
            let _ = cache.ops.invalidate(talk.id).await;
            let _ = cache.ops.insert(talk).await;
        }
        Some(TalkCommandStatus::Updated)
    }

    /// Delete a Talk, dropping its cache entry.
    pub async fn delete(&mut self, id: i64) -> Option<TalkCommandStatus> {
        let delete_status = self.operation_repo.delete(id).await?;
        if delete_status != TalkCommandStatus::Deleted {
            error!(
                "Failed to delete talk with Id {}: status is not Deleted",
                id
            );
            return None;
        }

        if let Some(cache) = self.cache.as_mut() {
            debug!("Invalidating talk {} cache", id);
            let _ = cache.ops.invalidate(id).await;
        }
        Some(TalkCommandStatus::Deleted)
    }

    /// Warm the Talk cache from the database. A no-op when the cache is
    /// not attached.
    pub async fn prefill(&mut self) {
        if self.cache.is_none() {
            return;
        }

        let talks_opt = self
            .display_repo
            .find_talks(TalksParams {
                start: None,
                end: None,
            })
            .await;
        let Some(talks) = talks_opt else {
            warn!("Talks from database are empty");
            return;
        };

        info!("Inserting talks cache");
        for talk in talks.talks {
            let Some(cache) = self.cache.as_mut() else {
                return;
            };
            if cache.ops.insert(talk.clone()).await.is_none() {
                warn!("Failed to insert talk with id {} into cache", talk.id);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cache::inmemory::InMemoryCache;
    use crate::database::turso::TursoDatabase;

    /// Open a SQLite repo pair against a unique temp file. Re-opening the
    /// same name does not wipe it: only `fresh_db_repo` starts clean.
    async fn db_repo(name: &str) -> TursoDatabase {
        let path = std::env::temp_dir().join(format!("husni-talk-usecase-test-{name}.db"));
        let url = format!("file:{}", path.display());
        TursoDatabase::new("sqlite".to_string(), url, None).await
    }

    /// Open a SQLite repo pair on a clean temp file.
    async fn fresh_db_repo(name: &str) -> TursoDatabase {
        let path = std::env::temp_dir().join(format!("husni-talk-usecase-test-{name}.db"));
        let _ = std::fs::remove_file(&path);
        db_repo(name).await
    }

    async fn usecase_no_cache(name: &str) -> TalkUseCase {
        let db = fresh_db_repo(name).await;
        TalkUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
    }

    async fn usecase_cached(name: &str) -> TalkUseCase {
        let db = fresh_db_repo(name).await;
        let cache = InMemoryCache::new(3600);
        TalkUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_cache(Box::new(cache.clone()), Box::new(cache))
    }

    fn sample_talk(id: i64, name: &str) -> Talk {
        Talk {
            id,
            name: name.to_string(),
            date: "2024-01-01".to_string(),
            media_link: Some("https://example.com".to_string()),
            org_name: Some("Org".to_string()),
            org_link: Some("https://example.com/org".to_string()),
        }
    }

    fn init_test_tracing() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    }

    #[tokio::test]
    async fn test_add_find_find_talks_and_delete_without_cache() {
        init_test_tracing();
        let mut uc = usecase_no_cache("no_cache_flow").await;

        let add_result = uc.add(sample_talk(1, "First")).await;
        assert_eq!(add_result, Some(TalkCommandStatus::Stored));

        let found = uc.find(1).await.expect("talk 1 must be found");
        assert_eq!(found.name, "First");

        let talks = uc
            .find_talks(TalksParams {
                start: Some(0),
                end: Some(100),
            })
            .await
            .expect("talks list must resolve");
        assert_eq!(talks.talks.len(), 1);
        assert_eq!(talks.talks[0].id, 1);

        assert!(uc.find(99).await.is_none(), "unknown Talk id is None");

        let delete_result = uc.delete(1).await;
        assert_eq!(delete_result, Some(TalkCommandStatus::Deleted));
        assert!(uc.find(1).await.is_none(), "deleted talk is gone");
    }

    #[tokio::test]
    async fn test_update_keeps_the_cache_faithful() {
        init_test_tracing();
        let mut uc = usecase_cached("cached_update").await;
        uc.add(sample_talk(1, "First")).await;

        let mut updated = sample_talk(1, "First updated");
        updated.media_link = None;
        let update_result = uc.update(updated).await;
        assert_eq!(update_result, Some(TalkCommandStatus::Updated));

        let found = uc.find(1).await.expect("talk 1 must stay readable");
        assert_eq!(found.name, "First updated", "cache must hold the new name");
        assert!(found.media_link.is_none());
    }

    #[tokio::test]
    async fn test_prefill_warms_the_database_rows() {
        init_test_tracing();
        let name = "cached_prefill";
        let mut writer = usecase_no_cache(name).await;
        writer.add(sample_talk(1, "First")).await;
        writer.add(sample_talk(2, "Second")).await;

        // Another usecase over the same db file but a cold cache.
        let db = db_repo(name).await;
        let cache = InMemoryCache::new(3600);
        let mut fresh = TalkUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_cache(Box::new(cache.clone()), Box::new(cache));
        fresh.prefill().await;

        let talks = fresh
            .find_talks(TalksParams {
                start: Some(0),
                end: Some(100),
            })
            .await
            .expect("prefill must make the list readable cache-only");
        assert_eq!(talks.talks.len(), 2);
    }
}
