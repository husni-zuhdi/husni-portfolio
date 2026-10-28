use crate::model::blog_tag_mappings::{BlogTagMappingCommandStatus, BlogTagMappings};
use crate::repo::blog_tag_mappings::*;
use core::fmt::Debug;
use tracing::{debug, error, info, warn};

impl Debug for dyn BlogTagMappingDisplayRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogTagMappingDisplayRepo")
    }
}

impl Debug for dyn BlogTagMappingOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogTagMappingOperationRepo")
    }
}

impl Debug for dyn BlogTagMappingCacheOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogTagMappingCacheOperationRepo")
    }
}

/// BlogTagMappingUseCase
/// The deep usecase for Blog tag mapping reads and writes. It is consumed by
/// the Blog usecase's write choreography; no HTTP handler calls it directly.
///
/// Read verbs resolve to the database directly: the Blog tag mapping truth
/// must not come from a cache that the Blog writes race with. The write verbs
/// own the cache choreography: every mapping change refreshes or drops the
/// matching cached entries. Cache disabling is resolved at construction.
///
/// Every verb keeps the repo layer's `Option` convention: `None` means the
/// operation failed, and no partial-success distinction is re-derived here.
#[derive(Clone)]
pub struct BlogTagMappingUseCase {
    display_repo: Box<dyn BlogTagMappingDisplayRepo + Send + Sync>,
    operation_repo: Box<dyn BlogTagMappingOperationRepo + Send + Sync>,
    /// Optional in-process cache. `None` means caching is disabled.
    cache: Option<BtmCache>,
}

#[derive(Clone)]
struct BtmCache {
    ops: Box<dyn BlogTagMappingCacheOperationRepo + Send + Sync>,
}

impl Debug for BlogTagMappingUseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BlogTagMappingUseCase")
    }
}

impl BlogTagMappingUseCase {
    /// Create a BlogTagMappingUseCase without a cache.
    pub fn new(
        display_repo: Box<dyn BlogTagMappingDisplayRepo + Send + Sync>,
        operation_repo: Box<dyn BlogTagMappingOperationRepo + Send + Sync>,
    ) -> Self {
        Self {
            display_repo,
            operation_repo,
            cache: None,
        }
    }

    /// Attach the in-process cache operations.
    pub fn with_cache(
        mut self,
        cache_ops: Box<dyn BlogTagMappingCacheOperationRepo + Send + Sync>,
    ) -> Self {
        self.cache = Some(BtmCache { ops: cache_ops });
        self
    }

    /// Read the Blog tag mappings of a Blog from the database.
    pub async fn find_by_blog_id(&mut self, blog_id: i64) -> Option<BlogTagMappings> {
        self.display_repo.find_by_blog_id(blog_id).await
    }

    /// Read the Blog tag mappings of a Tag from the database.
    pub async fn find_by_tag_id(&mut self, tag_id: i64) -> Option<BlogTagMappings> {
        self.display_repo.find_by_tag_id(tag_id).await
    }

    /// Add one Blog tag mapping, keeping the cache in step.
    pub async fn add(&mut self, blog_id: i64, tag_id: i64) -> Option<BlogTagMappingCommandStatus> {
        let add_status = self.operation_repo.add(blog_id, tag_id).await?;
        if add_status != BlogTagMappingCommandStatus::Stored {
            error!(
                "Failed to add Blog tag mapping for blog id {} and tag id {}: status is not Stored",
                blog_id, tag_id
            );
            return None;
        }

        if let Some(cache) = self.cache.as_mut() {
            debug!(
                "Caching Blog tag mapping for blog id {} and tag id {}",
                blog_id, tag_id
            );
            let _ = cache.ops.insert(blog_id, tag_id).await;
        }
        Some(BlogTagMappingCommandStatus::Stored)
    }

    /// Delete every mapping of a Blog, dropping the cached entries.
    pub async fn delete_by_blog_id(&mut self, blog_id: i64) -> Option<BlogTagMappingCommandStatus> {
        let delete_status = self.operation_repo.delete_by_blog_id(blog_id).await?;
        if delete_status != BlogTagMappingCommandStatus::Deleted {
            error!(
                "Failed to delete Blog tag mappings for blog id {}: status is not Deleted",
                blog_id
            );
            return None;
        }

        if let Some(cache) = self.cache.as_mut() {
            debug!(
                "Invalidating Blog tag mapping cache for blog id {}",
                blog_id
            );
            let invalidated = cache.ops.invalidate_by_blog_id(blog_id).await;
            if invalidated.is_none() {
                warn!(
                    "Failed to invalidate Blog tag mappings for blog id {}",
                    blog_id
                );
            }
        }
        Some(BlogTagMappingCommandStatus::Deleted)
    }

    /// Delete one mapping of a Blog, dropping its cached entry.
    pub async fn delete_by_blog_id_and_tag_id(
        &mut self,
        blog_id: i64,
        tag_id: i64,
    ) -> Option<BlogTagMappingCommandStatus> {
        let delete_status = self
            .operation_repo
            .delete_by_blog_id_and_tag_id(blog_id, tag_id)
            .await?;
        if delete_status != BlogTagMappingCommandStatus::Deleted {
            error!(
                "Failed to delete Blog tag mapping for blog id {} and tag id {}: status is not Deleted",
                blog_id, tag_id
            );
            return None;
        }

        if let Some(cache) = self.cache.as_mut() {
            debug!(
                "Invalidating Blog tag mapping cache for blog id {} and tag id {}",
                blog_id, tag_id
            );
            let _ = cache.ops.invalidate(blog_id, tag_id).await;
        }
        Some(BlogTagMappingCommandStatus::Deleted)
    }

    /// Warm the Blog tag mapping cache for the given Blogs from the
    /// database. A no-op when the cache is not attached.
    pub async fn prefill_for_blog_ids(&mut self, blog_ids: Vec<i64>) {
        if self.cache.is_none() {
            return;
        }

        for id in blog_ids {
            let Some(btms_result) = self.display_repo.find_by_blog_id(id).await else {
                warn!("Blog tag mappings for blog id {} in database are empty", id);
                continue;
            };
            info!("Inserting Blog tag mappings for blog id {} into cache", id);
            let Some(cache) = self.cache.as_mut() else {
                break;
            };
            for val in btms_result.maps {
                let insert_opt = cache.ops.insert(val.blog_id, val.tag_id).await;
                if insert_opt.is_none() {
                    warn!(
                        "Failed to insert Blog tag mapping with blog id {} and tag id {} into cache",
                        val.blog_id, val.tag_id
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cache::inmemory::InMemoryCache;
    use crate::database::turso::TursoDatabase;
    use crate::model::blog_tag_mappings::BlogTagMapping;

    /// Open a SQLite repo pair against a unique temp file. Re-opening the
    /// same name does not wipe it: only `fresh_db_repo` starts clean.
    async fn db_repo(name: &str) -> TursoDatabase {
        let path = std::env::temp_dir().join(format!("husni-btm-usecase-test-{name}.db"));
        let url = format!("file:{}", path.display());
        TursoDatabase::new("sqlite".to_string(), url, None).await
    }

    /// Open a SQLite repo pair on a clean temp file.
    async fn fresh_db_repo(name: &str) -> TursoDatabase {
        let path = std::env::temp_dir().join(format!("husni-btm-usecase-test-{name}.db"));
        let _ = std::fs::remove_file(&path);
        db_repo(name).await
    }

    async fn usecase_no_cache(name: &str) -> BlogTagMappingUseCase {
        let db = fresh_db_repo(name).await;
        BlogTagMappingUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
    }

    async fn usecase_cached(name: &str) -> (BlogTagMappingUseCase, InMemoryCache) {
        let db = fresh_db_repo(name).await;
        let cache = InMemoryCache::new(3600);
        let uc = BlogTagMappingUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_cache(Box::new(cache.clone()));
        (uc, cache)
    }

    fn init_test_tracing() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    }

    #[tokio::test]
    async fn test_add_and_find_reads_the_database() {
        init_test_tracing();
        let mut uc = usecase_no_cache("no_cache_read").await;

        let add_result = uc.add(1, 2).await;
        assert_eq!(add_result, Some(BlogTagMappingCommandStatus::Stored));

        let mappings = uc.find_by_blog_id(1).await.expect("mapping must be found");
        assert_eq!(
            mappings,
            BlogTagMappings {
                maps: vec![BlogTagMapping {
                    blog_id: 1,
                    tag_id: 2
                }]
            }
        );
    }

    #[tokio::test]
    async fn test_add_keeps_cache_in_step() {
        init_test_tracing();
        let (mut uc, cache) = usecase_cached("cached_add").await;
        uc.add(1, 2).await;

        let cached = cache
            .find_by_blog_id(1)
            .await
            .expect("the mapping must be cached after add");
        assert_eq!(
            cached.maps,
            vec![BlogTagMapping {
                blog_id: 1,
                tag_id: 2
            }]
        );
    }

    #[tokio::test]
    async fn test_delete_by_blog_id_and_tag_id_removes_single_mapping() {
        init_test_tracing();
        let (mut uc, cache) = usecase_cached("cached_delete_pair").await;
        uc.add(1, 2).await;
        uc.add(1, 3).await;

        let delete_result = uc.delete_by_blog_id_and_tag_id(1, 2).await;
        assert_eq!(delete_result, Some(BlogTagMappingCommandStatus::Deleted));

        let cached = cache.find_by_blog_id(1).await.unwrap();
        assert_eq!(cached.maps.len(), 1);
        assert_eq!(cached.maps[0].tag_id, 3);
    }

    #[tokio::test]
    async fn test_delete_by_blog_id_clears_every_mapping() {
        init_test_tracing();
        let mut uc = usecase_no_cache("no_cache_delete_all").await;
        uc.add(1, 2).await;
        uc.add(1, 3).await;

        let delete_result = uc.delete_by_blog_id(1).await;
        assert_eq!(delete_result, Some(BlogTagMappingCommandStatus::Deleted));

        let mappings = uc.find_by_blog_id(1).await.unwrap();
        assert!(mappings.maps.is_empty(), "no mapping must remain");
    }

    #[tokio::test]
    async fn test_prefill_warms_the_cache_from_the_database() {
        init_test_tracing();
        let db_repo_name = "cached_prefill";
        let mut write_uc = usecase_no_cache(db_repo_name).await;
        write_uc.add(1, 2).await;
        write_uc.add(1, 3).await;

        let db = db_repo(db_repo_name).await;
        let cache = InMemoryCache::new(3600);
        let mut prefilled = BlogTagMappingUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_cache(Box::new(cache.clone()));
        prefilled.prefill_for_blog_ids(vec![1]).await;

        let cached = cache
            .find_by_blog_id(1)
            .await
            .expect("prefill must warm the cache for blog id 1");
        assert_eq!(cached.maps.len(), 2);
    }
}
