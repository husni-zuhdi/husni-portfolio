use crate::model::tags::{Tag, TagCommandStatus, Tags, TagsListParams, TagsSearchParams};
use crate::repo::tags::*;
use core::fmt::Debug;
use tracing::{debug, error, info, warn};

impl Debug for dyn TagDisplayRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TagDisplayRepo")
    }
}

impl Debug for dyn TagOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TagOperationRepo")
    }
}

impl Debug for dyn TagCacheOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TagCacheOperationRepo")
    }
}

/// TagUseCase
/// The deep usecase for Tag reads and writes. HTTP handlers and the Blog
/// usecase call only the verbs exposed here; the database repos and the
/// optional in-process cache sit behind this interface.
///
/// Read-through rules:
/// - Detail reads hit the cache first, fall through to the database, and
///   refill the cache on a database hit.
/// - List reads are all-or-nothing: the cache answers only when every Tag
///   expected for the query is present, otherwise the read falls to the
///   database and refills the cache with the returned database rows.
///
/// Write verbs own their cache choreography: `add`, `update`, and `delete`
/// each refresh or drop the Tag cache. Cache disabling is resolved at
/// construction, so the verbs make no per-callsite decision about caching.
///
/// Every verb keeps the repo layer's `Option` convention: `None` means the
/// operation failed. Handlers render an error page from that, and no
/// partial-success distinction is re-derived here.
#[derive(Clone)]
pub struct TagUseCase {
    display_repo: Box<dyn TagDisplayRepo + Send + Sync>,
    operation_repo: Box<dyn TagOperationRepo + Send + Sync>,
    /// Optional in-process cache. `None` means caching is disabled.
    cache: Option<TagCache>,
}

#[derive(Clone)]
struct TagCache {
    display: Box<dyn TagDisplayRepo + Send + Sync>,
    ops: Box<dyn TagCacheOperationRepo + Send + Sync>,
}

impl Debug for TagUseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TagUseCase")
    }
}

impl TagUseCase {
    /// Create a TagUseCase without a cache.
    pub fn new(
        display_repo: Box<dyn TagDisplayRepo + Send + Sync>,
        operation_repo: Box<dyn TagOperationRepo + Send + Sync>,
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
        cache_display: Box<dyn TagDisplayRepo + Send + Sync>,
        cache_ops: Box<dyn TagCacheOperationRepo + Send + Sync>,
    ) -> Self {
        self.cache = Some(TagCache {
            display: cache_display,
            ops: cache_ops,
        });
        self
    }

    /// Read a single Tag through the cache, refilling on a database hit.
    pub async fn find(&mut self, id: i64) -> Option<Tag> {
        if let Some(cache) = self.cache.as_ref() {
            if let Some(tag) = cache.display.find(id).await {
                return Some(tag);
            }
        }

        let db_result = self.display_repo.find(id).await;
        if let Some(tag) = db_result.clone() {
            if let Some(cache) = self.cache.as_mut() {
                let _ = cache.ops.insert(tag).await;
            }
        }
        db_result
    }

    /// Read a list of Tags through the cache with all-or-nothing semantics.
    pub async fn find_tags(&mut self, params: TagsListParams) -> Option<Tags> {
        if let Some(cache) = self.cache.as_ref() {
            let cache_result = cache.display.find_tags(params.clone()).await;
            if cache_result.is_some() {
                return cache_result;
            }
        }

        let db_result = self.display_repo.find_tags(params.clone()).await;
        if let Some(tags) = db_result.clone() {
            if let Some(cache) = self.cache.as_mut() {
                for tag in tags.tags {
                    debug!("Refilling tag cache {}", &tag.id);
                    let _ = cache.ops.insert(tag).await;
                }
            }
        }
        db_result
    }

    /// Search Tags in the database. The database is the only truthful
    /// source for a LIKE search: the tag cache only holds the tags that
    /// list reads and writes have made present, so it cannot prove that
    /// uncached ids do not match the query. Returned rows refill the cache
    /// so later list reads can resolve cache-only.
    pub async fn search_tags(&mut self, params: TagsSearchParams) -> Option<Tags> {
        let db_result = self.display_repo.search_tags(params.clone()).await;
        if let Some(tags) = db_result.clone() {
            if let Some(cache) = self.cache.as_mut() {
                for tag in tags.tags {
                    debug!("Refilling tag cache {}", &tag.id);
                    let _ = cache.ops.insert(tag).await;
                }
            }
        }
        db_result
    }

    /// Next Tag id for the add form.
    pub async fn get_new_id(&mut self) -> Option<i64> {
        self.operation_repo.get_new_id().await
    }

    /// Add a Tag, keeping the cache in step.
    pub async fn add(&mut self, tag: Tag) -> Option<TagCommandStatus> {
        let add_status = self.operation_repo.add(tag.id, tag.clone().name).await?;
        if add_status != TagCommandStatus::Stored {
            error!(
                "Failed to add tag with Id {}: status is not Stored",
                &tag.id
            );
            return None;
        }

        if let Some(cache) = self.cache.as_mut() {
            debug!("Caching tag {}", &tag.id);
            let _ = cache.ops.insert(tag).await;
        }
        Some(TagCommandStatus::Stored)
    }

    /// Update a Tag's name, keeping the cache in step.
    pub async fn update(&mut self, tag: Tag) -> Option<TagCommandStatus> {
        let update_status = self
            .operation_repo
            .update(tag.id, Some(tag.clone().name))
            .await?;
        if update_status != TagCommandStatus::Updated {
            error!(
                "Failed to update tag with Id {}: status is not Updated",
                &tag.id
            );
            return None;
        }

        if let Some(cache) = self.cache.as_mut() {
            debug!("Invalidating tag {} cache before re-inserting", &tag.id);
            let _ = cache.ops.invalidate(tag.id).await;
            let _ = cache.ops.insert(tag).await;
        }
        Some(TagCommandStatus::Updated)
    }

    /// Delete a Tag, dropping its cache entry.
    pub async fn delete(&mut self, id: i64) -> Option<TagCommandStatus> {
        let delete_status = self.operation_repo.delete(id).await?;
        if delete_status != TagCommandStatus::Deleted {
            error!("Failed to delete tag with Id {}: status is not Deleted", id);
            return None;
        }

        if let Some(cache) = self.cache.as_mut() {
            debug!("Invalidating tag {} cache", id);
            let _ = cache.ops.invalidate(id).await;
        }
        Some(TagCommandStatus::Deleted)
    }

    /// Ensure all tag names in `names` exist, creating any missing Tags
    /// (and refreshing the Tag cache along the way), and return the Tags
    /// resolved by id. Shared verb of the Blog write choreography.
    pub async fn ensure_tag_names(&mut self, names: Vec<String>) -> Option<Vec<Tag>> {
        let available = self
            .find_tags(TagsListParams {
                start: Some(0),
                end: Some(1000),
            })
            .await?;

        let mut result = Vec::with_capacity(names.len());
        let mut missing = Vec::new();

        for name in names {
            match available.tags.iter().find(|t| t.name == name) {
                Some(tag) => result.push(tag.clone()),
                None => missing.push(name),
            }
        }

        for name in missing {
            let new_id = self.get_new_id().await?;
            debug!("Creating missing Tag {} with new id {}", &name, &new_id);
            let tag = Tag {
                id: new_id,
                name: name.clone(),
            };
            let add_status = self.add(tag.clone()).await?;
            if add_status != TagCommandStatus::Stored {
                error!("Failed to create new tag {}", name);
                return None;
            }
            result.push(tag);
        }

        Some(result)
    }

    /// Warm the Tag cache from the database. A no-op when the cache is not
    /// attached.
    pub async fn prefill(&mut self) {
        if self.cache.is_none() {
            return;
        }

        let tags_opt = self
            .display_repo
            .find_tags(TagsListParams {
                start: None,
                end: None,
            })
            .await;
        let Some(tags) = tags_opt else {
            warn!("Tags from database are empty");
            return;
        };

        info!("Inserting tags cache");
        for tag in tags.tags {
            let Some(cache) = self.cache.as_mut() else {
                return;
            };
            if cache.ops.insert(tag.clone()).await.is_none() {
                warn!("Failed to insert tag with id {} into cache", tag.id);
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
        let path = std::env::temp_dir().join(format!("husni-tag-usecase-test-{name}.db"));
        let url = format!("file:{}", path.display());
        TursoDatabase::new("sqlite".to_string(), url, None).await
    }

    /// Open a SQLite repo pair on a clean temp file.
    async fn fresh_db_repo(name: &str) -> TursoDatabase {
        let path = std::env::temp_dir().join(format!("husni-tag-usecase-test-{name}.db"));
        let _ = std::fs::remove_file(&path);
        db_repo(name).await
    }

    async fn usecase_no_cache(name: &str) -> TagUseCase {
        let db = fresh_db_repo(name).await;
        TagUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
    }

    async fn usecase_cached(name: &str) -> TagUseCase {
        let db = fresh_db_repo(name).await;
        let cache = InMemoryCache::new(3600);
        TagUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_cache(Box::new(cache.clone()), Box::new(cache))
    }

    fn init_test_tracing() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    }

    #[tokio::test]
    async fn test_find_and_find_tags_resolves_through_cache_and_db() {
        init_test_tracing();
        let mut uc = usecase_cached("cached_find").await;

        uc.add(Tag {
            id: 1,
            name: "rust".to_string(),
        })
        .await;
        uc.add(Tag {
            id: 2,
            name: "go".to_string(),
        })
        .await;

        // The cache answers both reads entirely after the writes.
        let found = uc.find(1).await.expect("tag 1 must be found");
        assert_eq!(found.name, "rust");

        let tags = uc
            .find_tags(TagsListParams {
                start: Some(0),
                end: Some(100),
            })
            .await
            .expect("tags list must resolve");
        assert_eq!(tags.tags.len(), 2, "both tags resolve through the cache");

        // A cache-cold id falls through to the database read contract.
        assert!(uc.find(99).await.is_none(), "unknown Tag id is None");
    }

    #[tokio::test]
    async fn test_search_tags_matches_names_and_refills_the_cache() {
        init_test_tracing();
        let mut uc = usecase_cached("cached_search").await;
        // Seed the database only: a cold cache must fall through to the db.
        let _ = uc
            .operation_repo
            .add(1, "rust".to_string())
            .await
            .expect("seed tag 1");
        let _ = uc
            .operation_repo
            .add(2, "go".to_string())
            .await
            .expect("seed tag 2");

        let tags = uc
            .search_tags(TagsSearchParams {
                start: Some(0),
                end: Some(100),
                query: "rus".to_string(),
            })
            .await
            .expect("search must resolve from the database");
        assert_eq!(tags.tags.len(), 1);
        assert_eq!(tags.tags[0].name, "rust");

        // Search is database-truth: "go" is a database row the previous
        // search refill did not cache, and the cache must not mask it.
        let tags = uc
            .search_tags(TagsSearchParams {
                start: Some(0),
                end: Some(100),
                query: "go".to_string(),
            })
            .await
            .expect("search must resolve from the database");
        assert_eq!(tags.tags.len(), 1);
        assert_eq!(tags.tags[0].name, "go");
    }

    #[tokio::test]
    async fn test_ensure_tag_names_creates_missing_tags() {
        init_test_tracing();
        let mut uc = usecase_no_cache("no_cache_ensure").await;
        uc.add(Tag {
            id: 1,
            name: "rust".to_string(),
        })
        .await;

        let tags = uc
            .ensure_tag_names(vec!["rust".to_string(), "go".to_string()])
            .await
            .expect("ensure must resolve both tags");
        assert_eq!(tags.len(), 2);
        assert!(tags.iter().any(|t| t.name == "rust" && t.id == 1));
        assert!(tags.iter().any(|t| t.name == "go"), "missing tag created");

        // The created Tag id follows the empty-table sequence.
        assert!(tags.iter().any(|t| t.name == "go" && t.id == 2));
    }

    #[tokio::test]
    async fn test_update_and_delete_keep_the_cache_faithful() {
        init_test_tracing();
        let mut uc = usecase_cached("cached_write").await;
        uc.add(Tag {
            id: 1,
            name: "rust".to_string(),
        })
        .await;

        let update_result = uc
            .update(Tag {
                id: 1,
                name: "rust-lang".to_string(),
            })
            .await;
        assert_eq!(update_result, Some(TagCommandStatus::Updated));
        let found = uc.find(1).await.expect("tag 1 must stay readable");
        assert_eq!(found.name, "rust-lang", "cache must hold the new name");

        let delete_result = uc.delete(1).await;
        assert_eq!(delete_result, Some(TagCommandStatus::Deleted));
        assert!(uc.find(1).await.is_none(), "deleted tag leaves the cache");
    }

    #[tokio::test]
    async fn test_prefill_warms_an_empty_database_without_failing() {
        init_test_tracing();
        let mut uc = usecase_cached("cached_prefill").await;
        uc.add(Tag {
            id: 1,
            name: "rust".to_string(),
        })
        .await;
        uc.add(Tag {
            id: 2,
            name: "go".to_string(),
        })
        .await;

        // Another usecase over the same db file but a cold cache.
        let db = db_repo("cached_prefill").await;
        let cache = InMemoryCache::new(3600);
        let mut fresh = TagUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_cache(Box::new(cache.clone()), Box::new(cache));
        fresh.prefill().await;

        let tags = fresh
            .find_tags(TagsListParams {
                start: Some(0),
                end: Some(100),
            })
            .await
            .expect("prefill must make the list readable cache-only");
        assert_eq!(tags.tags.len(), 2);
    }
}
