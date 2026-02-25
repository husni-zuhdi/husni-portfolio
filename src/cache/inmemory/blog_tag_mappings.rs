use crate::cache::inmemory::InMemoryCache;
use crate::model::blog_tag_mappings::*;
use crate::repo::blog_tag_mappings::{
    BlogTagMappingCacheOperationRepo as BlogTagMappingOperationRepo, BlogTagMappingDisplayRepo,
};
use async_trait::async_trait;
use tracing::{debug, info};

const BTM_KEY_PREFIX: &str = "btm";

#[async_trait]
impl BlogTagMappingDisplayRepo for InMemoryCache {
    /// Find a BlogTagMapping Caches by blog id
    async fn find_by_blog_id(&self, blog_id: i64) -> Option<BlogTagMappings> {
        debug!("Finding InMemoryCache {BTM_KEY_PREFIX} for blog id {blog_id}");

        let mut maps = Vec::new();
        for (_, v) in self.btms_cache.iter().filter(|(_, v)| v.blog_id == blog_id) {
            debug!("{BTM_KEY_PREFIX}-{blog_id}-{} cache hit", v.tag_id);
            maps.push(v);
        }

        // If Cache is still fresh, return None
        if maps.is_empty() {
            return None;
        }
        Some(BlogTagMappings { maps })
    }
    /// Find a BlogTagMapping Caches by blog id
    async fn find_by_tag_id(&self, tag_id: i64) -> Option<BlogTagMappings> {
        debug!("Finding InMemoryCache {BTM_KEY_PREFIX} for tag id {tag_id}");

        let mut maps = Vec::new();
        for (_, v) in self.btms_cache.iter().filter(|(_, v)| v.tag_id == tag_id) {
            debug!("{BTM_KEY_PREFIX}-{}-{tag_id} cache hit", v.blog_id);
            maps.push(v);
        }

        // If Cache is still fresh, return None
        if maps.is_empty() {
            return None;
        }
        Some(BlogTagMappings { maps })
    }
}

#[async_trait]
impl BlogTagMappingOperationRepo for InMemoryCache {
    /// Insert BlogTagMapping Cache
    /// Take a `BlogTagMapping` object and store it in the `InMemoryCache`
    /// Return Option of `BlogTagMappingCommandStatus`. If `None`, insertion failed
    async fn insert(&mut self, blog_id: i64, tag_id: i64) -> Option<BlogTagMappingCommandStatus> {
        let key = format!("{BTM_KEY_PREFIX}-{blog_id}-{tag_id}");
        info!("Inserting {} into InMemoryCache", &key);
        self.btms_cache
            .insert(key, BlogTagMapping { blog_id, tag_id })
            .await;
        Some(BlogTagMappingCommandStatus::CacheInserted)
    }
    /// Invalidate BlogTagMapping Cache
    /// Invalidate (discard value from the cached key) tag cache by tag id
    /// Return Option of `BlogTagMappingCommandStatus`. If `None`, invalidation failed
    async fn invalidate(
        &mut self,
        blog_id: i64,
        tag_id: i64,
    ) -> Option<BlogTagMappingCommandStatus> {
        let key = format!("{BTM_KEY_PREFIX}-{blog_id}-{tag_id}");
        info!("Invalidating {} from InMemoryCache", &key);
        self.btms_cache.invalidate(&key).await;
        Some(BlogTagMappingCommandStatus::CacheInvalidated)
    }
    /// Invalidate BlogTagMappings Cache by Blog id
    /// Invalidate (discard value from the cached key) btm cache by blog id
    /// Return Option of `BlogTagMappingCommandStatus`. If `None`, invalidation failed
    async fn invalidate_by_blog_id(&mut self, blog_id: i64) -> Option<BlogTagMappingCommandStatus> {
        let key = format!("{BTM_KEY_PREFIX}-{blog_id}-*");
        info!("Invalidating {} from InMemoryCache", &key);

        let btms_opt = self.find_by_blog_id(blog_id).await;
        btms_opt.as_ref()?;

        for btm in btms_opt.unwrap().maps {
            let k = format!("{BTM_KEY_PREFIX}-{blog_id}-{}", btm.tag_id);
            self.btms_cache.invalidate(&k).await;
            debug!("{} invalidated", &k);
        }
        Some(BlogTagMappingCommandStatus::CacheInvalidated)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_insert_find_by_blog_id_and_invalidate_btm() {
        let cache = InMemoryCache::new(3600);
        let test_val = BlogTagMapping {
            blog_id: 1,
            tag_id: 1,
        };

        // Insert cache
        let insert_status = cache
            .clone()
            .insert(test_val.blog_id, test_val.tag_id)
            .await;
        assert!(insert_status.is_some(), "btm-1-1 insertion failed");
        assert_eq!(
            insert_status.unwrap(),
            BlogTagMappingCommandStatus::CacheInserted
        );

        // Find cache
        let result = cache.clone().find_by_blog_id(test_val.blog_id).await;
        assert!(result.is_some(), "btm-1-1 find operation failed");
        assert_eq!(
            result.unwrap(),
            BlogTagMappings {
                maps: vec![test_val.clone()]
            }
        );

        // Invalidate cache
        let invalidate_status = cache
            .clone()
            .invalidate(test_val.blog_id, test_val.tag_id)
            .await;
        assert!(invalidate_status.is_some(), "btm-1-1 invalidation failed");
        assert_eq!(
            invalidate_status.unwrap(),
            BlogTagMappingCommandStatus::CacheInvalidated
        );

        // Find cache after invalidation
        let result = cache.clone().find_by_blog_id(test_val.blog_id).await;
        assert!(result.is_none(), "btm-1-1 find a cache! it shouldn't be!");
    }

    #[tokio::test]
    async fn test_insert_find_by_tag_id_and_invalidate_btm() {
        let cache = InMemoryCache::new(3600);
        let test_val = BlogTagMapping {
            blog_id: 1,
            tag_id: 1,
        };

        // Insert cache
        let insert_status = cache
            .clone()
            .insert(test_val.blog_id, test_val.tag_id)
            .await;
        assert!(insert_status.is_some(), "btm-1-1 insertion failed");
        assert_eq!(
            insert_status.unwrap(),
            BlogTagMappingCommandStatus::CacheInserted
        );

        // Find cache
        let result = cache.clone().find_by_tag_id(test_val.tag_id).await;
        assert!(result.is_some(), "btm-1-1 find operation failed");
        assert_eq!(
            result.unwrap(),
            BlogTagMappings {
                maps: vec![test_val.clone()]
            }
        );

        // Invalidate cache
        let invalidate_status = cache
            .clone()
            .invalidate(test_val.blog_id, test_val.tag_id)
            .await;
        assert!(invalidate_status.is_some(), "btm-1-1 invalidation failed");
        assert_eq!(
            invalidate_status.unwrap(),
            BlogTagMappingCommandStatus::CacheInvalidated
        );

        // Find cache after invalidation
        let result = cache.clone().find_by_blog_id(test_val.blog_id).await;
        assert!(result.is_none(), "btm-1-1 find a cache! it shouldn't be!");
    }

    #[tokio::test]
    async fn test_insert_find_and_invalidate_blog_tag_mappings() {
        let cache = InMemoryCache::new(3600);
        let blog_id = 1;
        let test_values = BlogTagMappings {
            maps: vec![
                BlogTagMapping { blog_id, tag_id: 1 },
                BlogTagMapping { blog_id, tag_id: 2 },
            ],
        };

        for test_val in &test_values.maps {
            let key = format!("{BTM_KEY_PREFIX}-{}-{}", test_val.blog_id, test_val.tag_id);
            // Insert cache
            let insert_status = cache
                .clone()
                .insert(test_val.blog_id, test_val.tag_id)
                .await;
            assert!(
                insert_status.is_some(),
                "{}",
                format!("{key} insertion failed")
            );
            assert_eq!(
                insert_status.unwrap(),
                BlogTagMappingCommandStatus::CacheInserted
            );

            // Find cache by tag id
            let result = cache.clone().find_by_tag_id(test_val.tag_id).await;
            assert!(
                result.is_some(),
                "{}",
                format!("{key} find operation failed")
            );
            assert_eq!(
                result.unwrap(),
                BlogTagMappings {
                    maps: vec![test_val.clone()]
                }
            );
        }

        // Find tag caches
        let blog_tag_mappings_res = cache.clone().find_by_blog_id(blog_id).await;
        assert!(
            blog_tag_mappings_res.is_some(),
            "find_by_blog_id operation failed"
        );
        // Disabled since we don't care the order of the btm
        //assert_eq!(blog_tag_mappings_res.unwrap(), test_values.clone());

        for test_val in &test_values.maps {
            let key = format!("{BTM_KEY_PREFIX}-{}-{}", test_val.blog_id, test_val.tag_id);
            // Invalidate cache
            let invalidate_status = cache
                .clone()
                .invalidate(test_val.blog_id, test_val.tag_id)
                .await;
            assert!(
                invalidate_status.is_some(),
                "{}",
                format!("{key} invalidation failed")
            );
            assert_eq!(
                invalidate_status.unwrap(),
                BlogTagMappingCommandStatus::CacheInvalidated
            );
        }

        // Find tag caches after invalidated
        let inv_blog_tag_mappings_res = cache.clone().find_by_blog_id(blog_id).await;
        assert!(
            inv_blog_tag_mappings_res.is_none(),
            "find_by_blog_id operation should failed after invalidation!"
        );
    }
}
