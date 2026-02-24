use crate::cache::inmemory::InMemoryCache;
use crate::model::blogs::*;
use crate::repo::blogs::{BlogCacheOperationRepo as BlogOperationRepo, BlogDisplayRepo};
use crate::utils::remove_whitespace;
use async_trait::async_trait;
use tracing::{debug, info};

const BLOG_KEY_PREFIX: &str = "blog";

#[async_trait]
impl BlogDisplayRepo for InMemoryCache {
    /// Find a Blog Cache
    /// Take blog id and return Option of `Blog`. If `None`, no blog was cached
    async fn find(&self, id: i64) -> Option<Blog> {
        debug!("Finding InMemoryCache {BLOG_KEY_PREFIX}-{id}");
        let key = format!("{BLOG_KEY_PREFIX}-{id}");
        self.blogs_cache.get(&key).await
    }
    /// Find Blogs Caches
    /// Take `BlogsParams` that contain `start`, `end`, and `tags` then
    /// return Option of `BlogMetadata` vector. `blogs` filed is always reversed.
    /// if `None`, no blogs within `BlogsParams` was cached
    async fn find_blogs(&self, params: BlogsParams) -> Option<Vec<Blog>> {
        let start_seq = params.start.unwrap() + 1;
        let end_seq = params.end.unwrap();
        let tags = params.tags.unwrap();
        debug!("Finding InMemoryCache {BLOG_KEY_PREFIX}-{start_seq} - {BLOG_KEY_PREFIX}-{end_seq} with tags {:?}", tags);

        let mut blogs = Vec::new();
        // rev() method to reverse Blog order
        // TODO: Observe the effect of Blog order reversal
        // I forgot why I need to reverse the order in Talk cache
        'cache_id: for id in (start_seq..=end_seq).rev() {
            let value = self.find(id).await;
            if value.is_none() {
                debug!("{BLOG_KEY_PREFIX}-{id} cache miss");
                continue;
            }

            // Return early when tags is empty
            if tags.is_empty() {
                debug!("{BLOG_KEY_PREFIX}-{id} cache hit");
                blogs.push(value.unwrap());
                continue;
            }

            for tag in remove_whitespace(&tags).split(",") {
                // If tag contained in blog tags, push blog and break `cache_id` loop early
                if value
                    .clone()
                    .unwrap()
                    .tags
                    .unwrap()
                    .contains(&tag.to_string())
                {
                    debug!("{BLOG_KEY_PREFIX}-{id} cache hit");
                    blogs.push(value.unwrap());
                    continue 'cache_id;
                }
            }
        }

        // If Cache is still fresh, return None
        if blogs.is_empty() {
            return None;
        }
        Some(blogs)
    }
}

#[async_trait]
impl BlogOperationRepo for InMemoryCache {
    /// Insert Blog Cache
    /// Take a `Blog` object and store it in the `InMemoryCache`
    /// Return Option of `BlogCommandStatus`. If `None`, insertion failed
    async fn insert(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        let key = format!("{BLOG_KEY_PREFIX}-{}", &blog.id);
        info!("Inserting {} into InMemoryCache", &key);
        self.blogs_cache.insert(key, blog).await;
        Some(BlogCommandStatus::CacheInserted)
    }
    /// Invalidate Blog Cache
    /// Invalidate (discard value from the cached key) blog cache by blog id
    /// Return Option of `BlogCommandStatus`. If `None`, invalidation failed
    async fn invalidate(&mut self, id: i64) -> Option<BlogCommandStatus> {
        let key = format!("{BLOG_KEY_PREFIX}-{id}");
        info!("Invalidating {} from InMemoryCache", &key);
        self.blogs_cache.invalidate(&key).await;
        Some(BlogCommandStatus::CacheInvalidated)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_insert_find_and_invalidate_blog() {
        let cache = InMemoryCache::new(3600);
        let test_val = Blog {
            id: 1,
            name: Some("test".to_string()),
            source: Some(BlogSource::Filesystem),
            filename: Some("test".to_string()),
            body: Some("#Hi".to_string()),
            tags: Some(vec!["test".to_string(), "tast".to_string()]),
        };

        // Insert cache
        let insert_status = cache.clone().insert(test_val.clone()).await;
        assert!(insert_status.is_some(), "blog-1 insertion failed");
        assert_eq!(insert_status.unwrap(), BlogCommandStatus::CacheInserted);

        // Find cache
        let result = cache.clone().find(test_val.id).await;
        assert!(result.is_some(), "blog-1 find operation failed");
        assert_eq!(result.unwrap(), test_val.clone());

        // Invalidate cache
        let invalidate_status = cache.clone().invalidate(test_val.id).await;
        assert!(invalidate_status.is_some(), "blog-1 invalidation failed");
        assert_eq!(
            invalidate_status.unwrap(),
            BlogCommandStatus::CacheInvalidated
        );

        // Find cache after invalidation
        let result = cache.clone().find(test_val.id).await;
        assert!(result.is_none(), "blog-1 find a cache! it shouldn't be!");
    }

    #[tokio::test]
    async fn test_insert_find_and_invalidate_blogs() {
        let cache = InMemoryCache::new(3600);
        let test_values = vec![
            Blog {
                id: 2,
                name: Some("tost".to_string()),
                source: Some(BlogSource::Filesystem),
                filename: Some("test".to_string()),
                body: Some("#Hello".to_string()),
                tags: Some(vec!["test".to_string(), "tast".to_string()]),
            },
            Blog {
                id: 1,
                name: Some("test".to_string()),
                source: Some(BlogSource::Filesystem),
                filename: Some("test".to_string()),
                body: Some("#Hi".to_string()),
                tags: Some(vec!["test".to_string(), "tast".to_string()]),
            },
        ];

        for test_val in &test_values {
            let id = test_val.id;
            // Insert cache
            let insert_status = cache.clone().insert(test_val.clone()).await;
            assert!(
                insert_status.is_some(),
                "{}",
                format!("{BLOG_KEY_PREFIX}-{id} insertion failed")
            );
            assert_eq!(insert_status.unwrap(), BlogCommandStatus::CacheInserted);

            // Find cache
            let result = cache.clone().find(id).await;
            assert!(
                result.is_some(),
                "{}",
                format!("{BLOG_KEY_PREFIX}-{id} find operation failed")
            );
            assert_eq!(result.unwrap(), test_val.clone());
        }

        // Find blog caches
        let blogs_res = cache
            .clone()
            .find_blogs(BlogsParams {
                start: Some(0),
                end: Some(2),
                tags: None,
            })
            .await;
        assert!(blogs_res.is_some(), "find_blogs operation failed");
        assert_eq!(blogs_res.unwrap(), test_values);

        for test_val in &test_values {
            let id = test_val.id;
            // Invalidate cache
            let invalidate_status = cache.clone().invalidate(id).await;
            assert!(
                invalidate_status.is_some(),
                "{}",
                format!("{BLOG_KEY_PREFIX}-{id} invalidation failed")
            );
            assert_eq!(
                invalidate_status.unwrap(),
                BlogCommandStatus::CacheInvalidated
            );
        }

        // Find blog caches after invalidated
        let blogs_res = cache
            .clone()
            .find_blogs(BlogsParams {
                start: Some(0),
                end: Some(2),
                tags: None,
            })
            .await;
        assert!(blogs_res.is_none(), "find_blogs operation should failed!");
    }
}
