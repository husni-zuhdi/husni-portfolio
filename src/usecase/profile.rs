use crate::model::profile::{
    MoveDirection, ProfileCommandStatus, ProfileMeta, ProfilePage, ProfileSection, ProfileSections,
};
use crate::repo::profile::*;
use core::fmt::Debug;
use tracing::{debug, error, info, warn};

impl Debug for dyn ProfileDisplayRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProfileDisplayRepo")
    }
}

impl Debug for dyn ProfileOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProfileOperationRepo")
    }
}

impl Debug for dyn ProfileCacheOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProfileCacheOperationRepo")
    }
}

/// ProfileUseCase
/// The deep usecase for the Profile page (sections and meta) reads and
/// writes. HTTP handlers call only the verbs exposed here; the database
/// repos and the optional in-process cache sit behind this interface.
///
/// The Profile page is cached as one unit (`ProfilePage` = sections + meta):
/// - Reads hit the cached page first and fall through to the database,
///   rebuilding the cached page on a database hit.
/// - Every write verb drops and rebuilds the cached page after the database
///   change. Cache disabling is resolved at construction, so the verbs make
///   no per-callsite decision about caching.
///
/// Every verb keeps the repo layer's `Option` convention: `None` means the
/// operation failed (a protected or missing section, for example). Handlers
/// render an error page from that, and no partial-success distinction is
/// re-derived here.
#[derive(Clone)]
pub struct ProfileUseCase {
    display_repo: Box<dyn ProfileDisplayRepo + Send + Sync>,
    operation_repo: Box<dyn ProfileOperationRepo + Send + Sync>,
    /// Optional in-process cache. `None` means caching is disabled.
    cache: Option<ProfileCache>,
}

#[derive(Clone)]
struct ProfileCache {
    display: Box<dyn ProfileDisplayRepo + Send + Sync>,
    ops: Box<dyn ProfileCacheOperationRepo + Send + Sync>,
}

impl Debug for ProfileUseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProfileUseCase")
    }
}

impl ProfileUseCase {
    /// Create a ProfileUseCase without a cache.
    pub fn new(
        display_repo: Box<dyn ProfileDisplayRepo + Send + Sync>,
        operation_repo: Box<dyn ProfileOperationRepo + Send + Sync>,
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
        cache_display: Box<dyn ProfileDisplayRepo + Send + Sync>,
        cache_ops: Box<dyn ProfileCacheOperationRepo + Send + Sync>,
    ) -> Self {
        self.cache = Some(ProfileCache {
            display: cache_display,
            ops: cache_ops,
        });
        self
    }

    /// Read all Profile Sections through the cache, rebuilding the cached
    /// page on a database hit.
    pub async fn find_sections(&mut self) -> Option<ProfileSections> {
        if let Some(cache) = self.cache.as_ref() {
            if let Some(sections) = cache.display.find_sections().await {
                return Some(sections);
            }
        }

        let sections = self.display_repo.find_sections().await?;
        self.rebuild_page_cache().await;
        Some(sections)
    }

    /// Read a single Profile Section through the cache, rebuilding the
    /// cached page on a database hit.
    pub async fn find_section(&mut self, id: i64) -> Option<ProfileSection> {
        if let Some(cache) = self.cache.as_ref() {
            if let Some(section) = cache.display.find_section(id).await {
                return Some(section);
            }
        }

        let section = self.display_repo.find_section(id).await?;
        self.rebuild_page_cache().await;
        Some(section)
    }

    /// Read the Profile Meta through the cache. Falls through to the
    /// database without touching the cache: meta is refreshed as part of
    /// the page rebuilds on reads and writes.
    pub async fn find_meta(&mut self) -> Option<ProfileMeta> {
        if let Some(cache) = self.cache.as_ref() {
            if let Some(meta) = cache.display.find_meta().await {
                return Some(meta);
            }
        }
        self.display_repo.find_meta().await
    }

    /// Next Profile Section id for the add form.
    pub async fn get_new_id(&mut self) -> Option<i64> {
        self.operation_repo.get_new_id().await
    }

    /// Add a Profile Section and refresh the page cache.
    pub async fn add(&mut self, section: ProfileSection) -> Option<ProfileCommandStatus> {
        let add_status = self.operation_repo.add(section.clone()).await?;
        if add_status != ProfileCommandStatus::Stored {
            error!(
                "Failed to add Profile Section {}: status is not Stored",
                section.id
            );
            return None;
        }
        self.rebuild_page_cache().await;
        Some(ProfileCommandStatus::Stored)
    }

    /// Update a Profile Section and refresh the page cache.
    pub async fn update(&mut self, section: ProfileSection) -> Option<ProfileCommandStatus> {
        let update_status = self.operation_repo.update(section.clone()).await?;
        if update_status != ProfileCommandStatus::Updated {
            error!(
                "Failed to update Profile Section {}: status is not Updated",
                section.id
            );
            return None;
        }
        self.rebuild_page_cache().await;
        Some(ProfileCommandStatus::Updated)
    }

    /// Delete a Profile Section and refresh the page cache. `None` means
    /// the section is missing or the protected page title section.
    pub async fn delete(&mut self, id: i64) -> Option<ProfileCommandStatus> {
        let delete_status = self.operation_repo.delete(id).await?;
        if delete_status != ProfileCommandStatus::Deleted {
            error!(
                "Failed to delete Profile Section {}: status is not Deleted",
                id
            );
            return None;
        }
        self.rebuild_page_cache().await;
        Some(ProfileCommandStatus::Deleted)
    }

    /// Move a Profile Section up or down and refresh the page cache.
    pub async fn move_section(
        &mut self,
        id: i64,
        direction: MoveDirection,
    ) -> Option<ProfileCommandStatus> {
        let move_status = self.operation_repo.move_section(id, direction).await?;
        if move_status != ProfileCommandStatus::Moved {
            info!("Failed to move Profile Section {}.", id);
            return None;
        }
        self.rebuild_page_cache().await;
        Some(ProfileCommandStatus::Moved)
    }

    /// Update the Profile Meta and refresh the page cache.
    pub async fn update_meta(&mut self, meta: ProfileMeta) -> Option<ProfileCommandStatus> {
        let update_status = self.operation_repo.update_meta(meta.clone()).await?;
        if update_status != ProfileCommandStatus::Updated {
            error!("Failed to update Profile Meta: status is not Updated");
            return None;
        }
        self.rebuild_page_cache().await;
        Some(ProfileCommandStatus::Updated)
    }

    /// Warm the Profile page cache from the database. A no-op when the
    /// cache is not attached.
    pub async fn prefill(&mut self) {
        if self.cache.is_none() {
            return;
        }

        let sections_opt = self.display_repo.find_sections().await;
        let meta_opt = self.display_repo.find_meta().await;
        match (sections_opt, meta_opt) {
            (Some(sections), Some(meta)) => {
                info!("Inserting profile page cache");
                self.insert_page(ProfilePage { sections, meta }).await;
            }
            (sections_opt, meta_opt) => {
                warn!(
                    "Profile sections or meta from database are empty (sections: {}, meta: {})",
                    sections_opt.is_none(),
                    meta_opt.is_none()
                );
            }
        }
    }

    /// Refresh the cached page after any change: drop the cached page,
    /// fetch sections and meta from the database, and store the fresh
    /// page. A no-op when the cache is not attached.
    async fn rebuild_page_cache(&mut self) {
        if let Some(cache) = self.cache.as_mut() {
            let _ = cache.ops.invalidate_page().await;
        }

        let sections_opt = self.display_repo.find_sections().await;
        let meta_opt = self.display_repo.find_meta().await;
        if let (Some(sections), Some(meta)) = (sections_opt, meta_opt) {
            debug!("Re-caching profile page");
            self.insert_page(ProfilePage { sections, meta }).await;
        } else {
            warn!("Failed to rebuild profile page cache. Database data are empty.");
        }
    }

    /// Store a built `ProfilePage` in the cache. Callers must have checked
    /// that the cache is attached.
    async fn insert_page(&mut self, page: ProfilePage) {
        if let Some(cache) = self.cache.as_mut() {
            if cache.ops.insert_page(page).await.is_none() {
                warn!("Failed to insert profile page into cache");
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
        let path = std::env::temp_dir().join(format!("husni-profile-usecase-test-{name}.db"));
        let url = format!("file:{}", path.display());
        TursoDatabase::new("sqlite".to_string(), url, None).await
    }

    /// Open a SQLite repo pair on a clean temp file.
    async fn fresh_db_repo(name: &str) -> TursoDatabase {
        let path = std::env::temp_dir().join(format!("husni-profile-usecase-test-{name}.db"));
        let _ = std::fs::remove_file(&path);
        db_repo(name).await
    }

    async fn usecase_no_cache(name: &str) -> ProfileUseCase {
        let db = fresh_db_repo(name).await;
        ProfileUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
    }

    async fn usecase_cached(name: &str) -> ProfileUseCase {
        let db = fresh_db_repo(name).await;
        let cache = InMemoryCache::new(3600);
        ProfileUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_cache(Box::new(cache.clone()), Box::new(cache))
    }

    fn sample_section(id: i64, title: &str) -> ProfileSection {
        ProfileSection {
            id,
            // Position is determined by the repo on insertion
            position: 0,
            title: title.to_string(),
            body: "Some body".to_string(),
            layout: crate::model::profile::Layout::Stacked,
        }
    }

    fn init_test_tracing() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    }

    // The SQLite test databases are seeded with 7 profile sections
    // (ids 1..=7, position 1 = protected page title section) and the
    // profile meta row. New sections must therefore use fresh ids.

    #[tokio::test]
    async fn test_add_update_delete_sections_without_cache() {
        init_test_tracing();
        let mut uc = usecase_no_cache("no_cache_flow").await;

        let add_result = uc.add(sample_section(10, "First")).await;
        assert_eq!(add_result, Some(ProfileCommandStatus::Stored));

        let found = uc.find_section(10).await.expect("section 10 must be found");
        assert_eq!(found.title, "First");
        assert_eq!(found.position, 8, "the repo appends after the 7 seeds");

        let update_result = uc.update(sample_section(10, "First updated")).await;
        assert_eq!(update_result, Some(ProfileCommandStatus::Updated));
        assert_eq!(
            uc.find_section(10).await.unwrap().title,
            "First updated",
            "the read must hold the new title"
        );

        let move_result = uc.move_section(10, MoveDirection::Up).await;
        assert_eq!(move_result, Some(ProfileCommandStatus::Moved));

        let delete_result = uc.delete(10).await;
        assert_eq!(delete_result, Some(ProfileCommandStatus::Deleted));
        assert!(
            uc.find_section(10).await.is_none(),
            "deleted section is gone"
        );
    }

    #[tokio::test]
    async fn test_protected_title_section_cannot_be_deleted() {
        init_test_tracing();
        let mut uc = usecase_cached("cached_protected").await;

        // The lowest-position section (seed id 1) is the protected page
        // title section.
        let delete_result = uc.delete(1).await;
        assert!(delete_result.is_none(), "protected section must stay");
        assert!(
            uc.find_section(1).await.is_some(),
            "protected section survives"
        );

        uc.add(sample_section(10, "Overview")).await;
        uc.delete(10).await;
        assert!(
            uc.find_section(10).await.is_none(),
            "unprotected section is gone"
        );
    }

    #[tokio::test]
    async fn test_update_meta_and_reads_stay_consistent() {
        init_test_tracing();
        let mut uc = usecase_cached("cached_meta").await;

        let meta = ProfileMeta {
            id: 1,
            description: "Husni Zuhdi Profile".to_string(),
        };
        let update_result = uc.update_meta(meta.clone()).await;
        assert_eq!(update_result, Some(ProfileCommandStatus::Updated));

        let sections_cache = uc.find_sections().await.unwrap();
        assert_eq!(sections_cache.sections.len(), 7);
        assert_eq!(uc.find_meta().await, Some(meta));
    }

    #[tokio::test]
    async fn test_reads_rebuild_the_page_cache() {
        init_test_tracing();
        let name = "cached_reads_rebuild";
        let mut writer = usecase_no_cache(name).await;
        writer.add(sample_section(10, "Overview")).await;

        // Another usecase over the same db file but a cold cache.
        let db = db_repo(name).await;
        let cache = InMemoryCache::new(3600);
        let mut fresh = ProfileUseCase::new(Box::new(db.clone()), Box::new(db.clone()))
            .with_cache(Box::new(cache.clone()), Box::new(cache));
        fresh.prefill().await;
        assert_eq!(
            fresh.find_sections().await.unwrap().sections.len(),
            8,
            "the list must resolve cache-only after prefill"
        );

        // A single read keeps the seeded truth reachable.
        let sections = fresh.find_section(1).await.expect("section must be found");
        assert_eq!(sections.position, 1);
    }
}
