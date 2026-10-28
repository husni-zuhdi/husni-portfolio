use crate::model::blogs::{Blog, BlogCommandStatus, BlogsParams};
use crate::model::tags::Tag;
use crate::repo::blogs::*;
use crate::usecase::blog_tag_mappings::BlogTagMappingUseCase;
use crate::usecase::tags::TagUseCase;
use core::fmt::Debug;
use tracing::{debug, error, info, warn};

impl Debug for dyn BlogDisplayRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogDisplayRepo")
    }
}

impl Debug for dyn BlogOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogOperationRepo")
    }
}

impl Debug for dyn BlogCacheOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogCacheOperationRepo")
    }
}

/// BlogUseCase
/// The deep usecase for Blog reads and writes. Handlers call only the verbs
/// exposed here; the database repos and the optional in-process cache sit
/// behind this interface.
///
/// Read-through rules:
/// - Detail reads hit the cache first, fall through to the database, and
///   refill the cache on a database hit.
/// - List reads are all-or-nothing: the cache answers only when every Blog
///   expected for the query is present, otherwise the read falls to the
///   database and refills the cache with the returned database rows.
///
/// Write verbs own their cache choreography: `add`, `update`, and `delete`
/// each refresh or drop the Blog cache, keep the Blog tag mappings in sync,
/// and create missing Tags as they go. Cache disabling is resolved at
/// construction, so the verbs make no per-callsite decision about caching.
///
/// Every verb keeps the repo layer's `Option` convention: `None` means the
/// operation or one of its correlated steps failed. Handlers render an error
/// page from that, and no partial-success distinction is re-derived there.
#[derive(Clone)]
pub struct BlogUseCase {
    display_repo: Box<dyn BlogDisplayRepo + Send + Sync>,
    operation_repo: Box<dyn BlogOperationRepo + Send + Sync>,
    /// Optional in-process cache. `None` means caching is disabled.
    cache: Option<BlogCache>,
    /// The Tag usecase the Blog write choreography needs to keep Blog tags
    /// in sync.
    tags: Option<TagUseCase>,
    /// The Blog tag usecase the Blog write choreography needs.
    btms: Option<BlogTagMappingUseCase>,
}

#[derive(Clone)]
struct BlogCache {
    display: Box<dyn BlogDisplayRepo + Send + Sync>,
    ops: Box<dyn BlogCacheOperationRepo + Send + Sync>,
}

impl Debug for BlogUseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BlogUseCase")
    }
}

impl BlogUseCase {
    /// Create a BlogUseCase without a cache.
    pub fn new(
        display_repo: Box<dyn BlogDisplayRepo + Send + Sync>,
        operation_repo: Box<dyn BlogOperationRepo + Send + Sync>,
    ) -> Self {
        Self {
            display_repo,
            operation_repo,
            cache: None,
            tags: None,
            btms: None,
        }
    }

    /// Attach the in-process cache adapters.
    pub fn with_cache(
        mut self,
        cache_display: Box<dyn BlogDisplayRepo + Send + Sync>,
        cache_ops: Box<dyn BlogCacheOperationRepo + Send + Sync>,
    ) -> Self {
        self.cache = Some(BlogCache {
            display: cache_display,
            ops: cache_ops,
        });
        self
    }

    /// Attach the Tag usecase the Blog write choreography needs to keep
    /// Blog tags in sync.
    pub fn with_tag_usecase(mut self, tags: TagUseCase) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Attach the Blog tag usecase the Blog write choreography needs.
    pub fn with_btm_usecase(mut self, btms: BlogTagMappingUseCase) -> Self {
        self.btms = Some(btms);
        self
    }

    /// Read a single Blog through the cache, refilling on a database hit.
    pub async fn find(&mut self, id: i64) -> Option<Blog> {
        if let Some(cache) = self.cache.as_ref() {
            if let Some(blog) = cache.display.find(id).await {
                return Some(blog);
            }
        }

        let db_result = self.display_repo.find(id).await;
        if let Some(blog) = db_result.clone() {
            if let Some(cache) = self.cache.as_mut() {
                let _ = cache.ops.insert(blog).await;
            }
        }
        db_result
    }

    /// Read a list of Blogs through the cache with all-or-nothing semantics.
    pub async fn find_blogs(&mut self, params: BlogsParams) -> Option<Vec<Blog>> {
        if let Some(cache) = self.cache.as_ref() {
            let cache_result = cache.display.find_blogs(params.clone()).await;
            if cache_result.is_some() {
                return cache_result;
            }
        }

        let db_result = self.display_repo.find_blogs(params.clone()).await;
        if let Some(blogs) = db_result.clone() {
            if let Some(cache) = self.cache.as_mut() {
                for blog in blogs {
                    debug!("Refilling blog cache {}", &blog.id);
                    let _ = cache.ops.insert(blog).await;
                }
            }
        }
        db_result
    }

    /// Next Blog id for the add form.
    pub async fn get_new_id(&mut self) -> Option<i64> {
        self.operation_repo.get_new_id().await
    }

    /// Add a Blog with its Blog tags. Any failing step after the Blog is
    /// stored keeps the stored Blog and makes the verb return `None`.
    pub async fn add(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        let blog_id = blog.id;
        let add_status = self.operation_repo.add(blog.clone()).await?;
        if add_status != BlogCommandStatus::Stored {
            error!(
                "Failed to add blog with Id {}: status is not Stored",
                blog_id
            );
            return None;
        }
        if let Some(cache) = self.cache.as_mut() {
            debug!("Caching blog {}", &blog_id);
            let _ = cache.ops.insert(blog.clone()).await;
        }

        let selected_tags = self
            .ensure_tag_names(blog.tags.clone().unwrap_or_default())
            .await?;
        debug!("Selected Tags: {:?}", selected_tags);

        let Some(btms) = self.btms.as_mut() else {
            error!("Failed to add Blog tag mappings: Blog tag usecase is not attached");
            return None;
        };
        for tag in &selected_tags {
            if btms.add(blog_id, tag.id).await.is_none() {
                error!(
                    "Failed to add Blog tag mapping for Blog id {} and Tag id {}",
                    blog_id, tag.id
                );
                return None;
            }
        }
        Some(BlogCommandStatus::Stored)
    }

    /// Update a Blog with its Blog tags. Any failing step after the Blog is
    /// updated keeps the updated Blog and makes the verb return `None`.
    pub async fn update(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        let blog_id = blog.id;
        let update_status = self.operation_repo.update(blog.clone()).await?;
        if update_status != BlogCommandStatus::Updated {
            error!(
                "Failed to update blog with Id {}: status is not Updated",
                blog_id
            );
            return None;
        }
        if let Some(cache) = self.cache.as_mut() {
            debug!("Invalidating blog {} cache before re-inserting", &blog_id);
            let _ = cache.ops.invalidate(blog_id).await;
            let _ = cache.ops.insert(blog.clone()).await;
        }

        let selected_tags = self
            .ensure_tag_names(blog.tags.clone().unwrap_or_default())
            .await?;
        let selected_tag_ids: Vec<i64> = selected_tags.iter().map(|t| t.id).collect();
        debug!("Selected Tag IDs {:?}", &selected_tag_ids);

        self.sync_blog_tag_mappings(blog_id, selected_tag_ids)
            .await?;
        Some(BlogCommandStatus::Updated)
    }

    /// Delete a Blog and its Blog tag mappings, dropping the Blog cache.
    pub async fn delete(&mut self, id: i64) -> Option<BlogCommandStatus> {
        let delete_status = self.operation_repo.delete(id).await;
        if delete_status.is_none() || delete_status.unwrap() != BlogCommandStatus::Deleted {
            error!("Failed to delete blog with Id {}", id);
            return None;
        }
        if let Some(cache) = self.cache.as_mut() {
            debug!("Invalidating blog {} cache", id);
            let _ = cache.ops.invalidate(id).await;
        }

        let Some(btms) = self.btms.as_mut() else {
            error!("Failed to delete Blog tag mappings: Blog tag usecase is not attached");
            return None;
        };
        if btms.delete_by_blog_id(id).await.is_none() {
            error!("Failed to delete Blog tag mappings for blog id {}", id);
            return None;
        }
        Some(BlogCommandStatus::Deleted)
    }

    /// Warm the Blog and Blog tag caches from the database. A no-op when the
    /// cache is not attached.
    pub async fn prefill(&mut self) {
        if self.cache.is_none() {
            return;
        }

        let blogs_opt = self
            .display_repo
            .find_blogs(BlogsParams {
                start: None,
                end: None,
                tags: None,
            })
            .await;
        let Some(blogs) = blogs_opt else {
            warn!("Blogs from database are empty");
            return;
        };
        let blog_ids: Vec<i64> = blogs.iter().map(|b| b.id).collect();

        info!("Inserting blogs cache");
        for blog in &blogs {
            let Some(cache) = self.cache.as_mut() else {
                return;
            };
            let _ = cache.ops.insert(blog.clone()).await;
        }

        let Some(btms) = self.btms.as_mut() else {
            warn!("Blog tag usecase is not attached. Skipping Blog tag cache prefill");
            return;
        };
        btms.prefill_for_blog_ids(blog_ids).await;
    }

    /// Ensure all tag names exist, delegating to the Tag usecase.
    async fn ensure_tag_names(&mut self, names: Vec<String>) -> Option<Vec<Tag>> {
        let Some(tags) = self.tags.as_mut() else {
            error!("Failed to ensure Tags: Tag usecase is not attached");
            return None;
        };
        tags.ensure_tag_names(names).await
    }

    /// Sync the Blog tag mappings of a Blog to the selected Tag ids:
    /// deletes mappings not selected anymore and adds the newly selected
    /// ones. The Blog tag usecase refreshes the cache for every change.
    async fn sync_blog_tag_mappings(
        &mut self,
        blog_id: i64,
        selected_tag_ids: Vec<i64>,
    ) -> Option<()> {
        let Some(btms) = self.btms.as_mut() else {
            error!("Failed to sync Blog tag mappings: Blog tag usecase is not attached");
            return None;
        };

        let existing_maps = btms.find_by_blog_id(blog_id).await?.maps;

        // Find tags not present in request but present in mapping.
        // Delete those mappings.
        for delete_map in existing_maps
            .iter()
            .filter(|m| !selected_tag_ids.contains(&m.tag_id))
        {
            info!(
                "Deleting Blog tag mapping for Blog id {} and Tag id {}",
                blog_id, delete_map.tag_id
            );
            if btms
                .delete_by_blog_id_and_tag_id(blog_id, delete_map.tag_id)
                .await
                .is_none()
            {
                info!(
                    "Failed to delete Blog tag mapping with Blog id {} and Tag id {}.",
                    blog_id, delete_map.tag_id
                );
                return None;
            }
        }

        // Find tags not present in mapping but present in request.
        // Add those mappings.
        for add_tag_id in selected_tag_ids
            .iter()
            .filter(|tag_id| !existing_maps.iter().any(|m| m.tag_id == **tag_id))
        {
            info!(
                "Adding Blog tag mapping for Blog id {} and Tag id {}",
                &blog_id, &add_tag_id
            );
            if btms.add(blog_id, *add_tag_id).await.is_none() {
                error!(
                    "Failed to add Blog tag mapping for Blog id {} and Tag id {}",
                    &blog_id, &add_tag_id
                );
                return None;
            }
        }

        Some(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cache::inmemory::InMemoryCache;
    use crate::database::turso::TursoDatabase;
    use crate::model::blog_tag_mappings::{BlogTagMapping, BlogTagMappings};
    use crate::model::blogs::BlogSource;
    use crate::repo::blog_tag_mappings::BlogTagMappingDisplayRepo;

    /// Open a SQLite repo pair against a unique temp file. Re-opening the
    /// same name does not wipe it: only `fresh_db_repo` starts clean.
    async fn db_repo(name: &str) -> TursoDatabase {
        let path = std::env::temp_dir().join(format!("husni-blog-usecase-test-{name}.db"));
        let url = format!("file:{}", path.display());
        TursoDatabase::new("sqlite".to_string(), url, None).await
    }

    /// Open a SQLite repo pair on a clean temp file.
    async fn fresh_db_repo(name: &str) -> TursoDatabase {
        let path = std::env::temp_dir().join(format!("husni-blog-usecase-test-{name}.db"));
        let _ = std::fs::remove_file(&path);
        db_repo(name).await
    }

    async fn usecase_no_cache(name: &str) -> BlogUseCase {
        let db = fresh_db_repo(name).await;
        // Mirror production wiring: Tag and Blog tag usecases are always
        // attached for the write choreography; only the cache is optional.
        let tags = TagUseCase::new(Box::new(db.clone()), Box::new(db.clone()));
        let btms = BlogTagMappingUseCase::new(Box::new(db.clone()), Box::new(db.clone()));
        BlogUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_tag_usecase(tags)
            .with_btm_usecase(btms)
    }

    async fn usecase_cached(name: &str) -> BlogUseCase {
        let db = fresh_db_repo(name).await;
        let cache = InMemoryCache::new(3600);
        let tags = TagUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_cache(Box::new(cache.clone()), Box::new(cache.clone()));
        let btms = BlogTagMappingUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_cache(Box::new(cache.clone()));
        BlogUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_cache(Box::new(cache.clone()), Box::new(cache))
            .with_tag_usecase(tags)
            .with_btm_usecase(btms)
    }

    fn sample_blog(id: i64, name: &str, tags: Vec<&str>) -> Blog {
        Blog {
            id,
            name: Some(name.to_string()),
            source: Some(BlogSource::Filesystem),
            filename: Some(format!("{name}.md")),
            body: Some("# Title".to_string()),
            tags: Some(tags.iter().map(|t| t.to_string()).collect()),
        }
    }

    fn init_test_tracing() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    }

    async fn mappings_of(db: &TursoDatabase, blog_id: i64) -> Vec<BlogTagMapping> {
        let btm_display: Box<dyn BlogTagMappingDisplayRepo + Send + Sync> = Box::new(db.clone());
        let btms = btm_display.find_by_blog_id(blog_id).await;
        btms.map(|result: BlogTagMappings| result.maps)
            .unwrap_or(vec![])
    }

    #[tokio::test]
    async fn test_find_and_find_blogs_without_cache() {
        init_test_tracing();
        let mut uc = usecase_no_cache("no_cache_find").await;
        let add_result = uc.add(sample_blog(1, "First", vec!["rust"])).await;
        assert_eq!(add_result, Some(BlogCommandStatus::Stored));

        let created_tag_ids = mappings_of(&db_repo("no_cache_find").await, 1).await;
        assert_eq!(created_tag_ids.len(), 1, "one Blog tag mapping expected");

        let found = uc.find(1).await;
        assert_eq!(found, Some(sample_blog(1, "First", vec!["rust"])));

        let list = uc
            .find_blogs(BlogsParams {
                start: Some(0),
                end: Some(100),
                tags: Some(String::new()),
            })
            .await
            .expect("list read should find the Blog");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, 1);

        assert!(uc.find(99).await.is_none(), "unknown Blog id is None");
    }

    #[tokio::test]
    async fn test_add_creates_missing_tags_and_mappings() {
        let mut uc = usecase_no_cache("no_cache_add_tags").await;
        let add_result = uc.add(sample_blog(1, "First", vec!["rust", "go"])).await;
        assert_eq!(add_result, Some(BlogCommandStatus::Stored));

        let mappings = mappings_of(&db_repo("no_cache_add_tags").await, 1).await;
        // "rust" and "go" get Tag ids 1 and 2 on the empty Tag table.
        let tag_ids: Vec<i64> = mappings.iter().map(|m| m.tag_id).collect();
        assert!(tag_ids.contains(&1) && tag_ids.contains(&2));
    }

    #[tokio::test]
    async fn test_update_syncs_blog_tag_mappings() {
        let mut uc = usecase_no_cache("no_cache_update").await;
        uc.add(sample_blog(1, "First", vec!["rust"])).await;
        let update_result = uc
            .update(sample_blog(1, "First updated", vec!["rust", "go"]))
            .await;
        assert_eq!(update_result, Some(BlogCommandStatus::Updated));

        let mappings = mappings_of(&db_repo("no_cache_update").await, 1).await;
        assert_eq!(mappings.len(), 2, "Blog tag mappings should hold both tags");

        let update_result = uc.update(sample_blog(1, "First updated", vec!["go"])).await;
        assert_eq!(update_result, Some(BlogCommandStatus::Updated));
        let mappings = mappings_of(&db_repo("no_cache_update").await, 1).await;
        assert_eq!(
            mappings.len(),
            1,
            "the removed Tag's mapping must be deleted"
        );
    }

    #[tokio::test]
    async fn test_delete_removes_blog_and_mappings() {
        let mut uc = usecase_no_cache("no_cache_delete").await;
        uc.add(sample_blog(1, "First", vec!["rust"])).await;
        uc.add(sample_blog(2, "Second", vec!["go"])).await;

        let delete_result = uc.delete(1).await;
        assert_eq!(delete_result, Some(BlogCommandStatus::Deleted));

        assert!(uc.find(1).await.is_none(), "deleted Blog is gone");
        let mappings = mappings_of(&db_repo("no_cache_delete").await, 1).await;
        assert!(mappings.is_empty(), "Blog tag mappings are deleted");
        assert!(uc.find(2).await.is_some(), "other Blog stays");
    }

    #[tokio::test]
    async fn test_cached_reads_stay_correct_across_writes() {
        let mut uc = usecase_cached("cached_flow").await;
        uc.prefill().await;

        // The database is empty: the list read resolves to an empty list,
        // mirroring the database read contract.
        let empty_list = uc
            .find_blogs(BlogsParams {
                start: Some(0),
                end: Some(100),
                tags: Some(String::new()),
            })
            .await;
        assert!(empty_list.is_some() && empty_list.unwrap().is_empty());

        uc.add(sample_blog(1, "First", vec!["rust"])).await;
        uc.add(sample_blog(2, "Second", vec!["go"])).await;

        let list = uc
            .find_blogs(BlogsParams {
                start: Some(0),
                end: Some(100),
                tags: Some(String::new()),
            })
            .await
            .expect("list read resolves after adds");
        assert_eq!(list.len(), 2);

        // The cache path answers the list with the fresh state.
        assert!(uc.find(2).await.is_some());

        uc.delete(2).await;
        let list = uc
            .find_blogs(BlogsParams {
                start: Some(0),
                end: Some(100),
                tags: Some(String::new()),
            })
            .await
            .expect("list read resolves after delete");
        assert_eq!(
            list.iter().map(|b| b.id).collect::<Vec<i64>>(),
            vec![1],
            "the deleted Blog leaves the list in the cache too"
        );
    }
}
