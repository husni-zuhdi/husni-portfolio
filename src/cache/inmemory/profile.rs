use crate::cache::inmemory::InMemoryCache;
use crate::model::profile::*;
use crate::repo::profile::{ProfileCacheOperationRepo, ProfileDisplayRepo};
use async_trait::async_trait;
use tracing::{debug, info};

const PROFILE_PAGE_KEY: &str = "profile-page";

#[async_trait]
impl ProfileDisplayRepo for InMemoryCache {
    /// Find Profile Sections Cache
    /// Return Option of `ProfileSections` ordered by position.
    /// If `None`, no profile page was cached
    async fn find_sections(&self) -> Option<ProfileSections> {
        debug!("Finding InMemoryCache {PROFILE_PAGE_KEY} sections");
        let page = self.profile_cache.get(PROFILE_PAGE_KEY).await?;
        Some(page.sections)
    }
    /// Find a Profile Section Cache
    /// Take section id and return Option of `ProfileSection`.
    /// If `None`, the section was not cached
    async fn find_section(&self, id: i64) -> Option<ProfileSection> {
        debug!("Finding InMemoryCache {PROFILE_PAGE_KEY} section {id}");
        let page = self.profile_cache.get(PROFILE_PAGE_KEY).await?;
        page.sections
            .sections
            .iter()
            .find(|section| section.id == id)
            .cloned()
    }
    /// Find Profile Meta Cache
    /// Return Option of `ProfileMeta`.
    /// If `None`, no profile page was cached
    async fn find_meta(&self) -> Option<ProfileMeta> {
        debug!("Finding InMemoryCache {PROFILE_PAGE_KEY} meta");
        let page = self.profile_cache.get(PROFILE_PAGE_KEY).await?;
        Some(page.meta)
    }
}

#[async_trait]
impl ProfileCacheOperationRepo for InMemoryCache {
    /// Insert the whole Profile Page Cache (sections + meta)
    /// Return Option of `ProfileCommandStatus`. If `None`, insertion failed
    async fn insert_page(&mut self, page: ProfilePage) -> Option<ProfileCommandStatus> {
        info!("Inserting {PROFILE_PAGE_KEY} into InMemoryCache");
        self.profile_cache
            .insert(PROFILE_PAGE_KEY.to_string(), page)
            .await;
        Some(ProfileCommandStatus::CacheInserted)
    }
    /// Invalidate the whole Profile Page Cache
    /// Return Option of `ProfileCommandStatus`. If `None`, invalidation failed
    async fn invalidate_page(&mut self) -> Option<ProfileCommandStatus> {
        info!("Invalidating {PROFILE_PAGE_KEY} from InMemoryCache");
        self.profile_cache.invalidate(PROFILE_PAGE_KEY).await;
        Some(ProfileCommandStatus::CacheInvalidated)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn sample_page() -> ProfilePage {
        ProfilePage {
            sections: ProfileSections {
                sections: vec![
                    ProfileSection {
                        id: 1,
                        position: 1,
                        title: String::new(),
                        body: "# Husni Naufal Zuhdi - Site Reliability Engineer".to_string(),
                        layout: Layout::Stacked,
                    },
                    ProfileSection {
                        id: 2,
                        position: 2,
                        title: "Overview".to_string(),
                        body: "Overview body".to_string(),
                        layout: Layout::Cards,
                    },
                ],
            },
            meta: ProfileMeta {
                id: 1,
                description: "Husni Zuhdi Profile".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn test_insert_find_and_invalidate_profile_page() {
        let cache = InMemoryCache::new(3600);
        let page = sample_page();

        // No page cached yet
        let sections = cache.clone().find_sections().await;
        assert!(sections.is_none(), "profile page should not be cached");

        // Insert page cache
        let insert_status = cache.clone().insert_page(page.clone()).await;
        assert!(insert_status.is_some(), "profile page insertion failed");
        assert_eq!(insert_status.unwrap(), ProfileCommandStatus::CacheInserted);

        // Find sections cache
        let sections = cache.clone().find_sections().await;
        assert!(sections.is_some(), "profile sections find operation failed");
        assert_eq!(sections.unwrap(), page.sections);

        // Find a single section cache
        let section = cache.clone().find_section(2).await;
        assert!(section.is_some(), "profile section find operation failed");
        assert_eq!(section.unwrap(), page.sections.sections[1]);
        let missing = cache.clone().find_section(99).await;
        assert!(missing.is_none(), "find should miss unknown section id");

        // Find meta cache
        let meta = cache.clone().find_meta().await;
        assert!(meta.is_some(), "profile meta find operation failed");
        assert_eq!(meta.unwrap(), page.meta);

        // Invalidate page cache
        let invalidate_status = cache.clone().invalidate_page().await;
        assert!(
            invalidate_status.is_some(),
            "profile page invalidation failed"
        );
        assert_eq!(
            invalidate_status.unwrap(),
            ProfileCommandStatus::CacheInvalidated
        );

        // No page cached after invalidation
        let sections = cache.clone().find_sections().await;
        assert!(sections.is_none(), "profile page should not be cached");
        let meta = cache.clone().find_meta().await;
        assert!(meta.is_none(), "profile meta should not be cached");
    }
}
