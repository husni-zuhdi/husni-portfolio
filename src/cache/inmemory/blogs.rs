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
    /// return Option of `Blog` vector. `blogs` filed is always reversed.
    /// if `None`, no blogs within `BlogsParams` was cached
    ///
    /// The read is all-or-nothing: the cache only answers when every indexed
    /// Blog id within `BlogsParams` is present in the cache. An indexed id
    /// that is missing from the cache makes the read return `None` so the
    /// caller falls through to the database and refills. An id without a
    /// cache entry and without an index entry is a gap, not a miss.
    async fn find_blogs(&self, params: BlogsParams) -> Option<Vec<Blog>> {
        let start_seq = params.start.unwrap() + 1;
        let end_seq = params.end.unwrap();
        let tags = params.tags.unwrap_or_default();
        debug!("Finding InMemoryCache {BLOG_KEY_PREFIX}-{start_seq} - {BLOG_KEY_PREFIX}-{end_seq} with tags {:?}", tags);

        let expected_ids: Vec<i64> = {
            let index = self.blogs_index.read().expect("blogs index lock poisoned");
            (start_seq..=end_seq)
                .rev()
                .filter(|id| index.contains(id))
                .collect()
        };

        let mut blogs = Vec::new();
        'cache_id: for id in expected_ids {
            let value = self.find(id).await;
            let Some(value) = value else {
                // Indexed but missing: this is a cache miss.
                // All-or-nothing means the whole read falls through.
                debug!("{BLOG_KEY_PREFIX}-{id} cache miss");
                return None;
            };

            debug!("{BLOG_KEY_PREFIX}-{id} cache hit");

            // Return early when tags is empty
            if tags.is_empty() {
                blogs.push(value);
                continue;
            }

            for tag in remove_whitespace(&tags).split(",") {
                // If tag contained in blog tags, push blog and break `cache_id` loop early
                if value.clone().tags.unwrap().contains(&tag.to_string()) {
                    blogs.push(value);
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
        let key = format!("{BLOG_KEY_PREFIX}-{}", blog.id);
        info!("Inserting {} into InMemoryCache", &key);
        self.blogs_cache.insert(key, blog.clone()).await;
        self.blogs_index
            .write()
            .expect("blogs index lock poisoned")
            .insert(blog.id);
        Some(BlogCommandStatus::CacheInserted)
    }
    /// Invalidate Blog Cache
    /// Invalidate (discard value from the cached key) blog cache by blog id
    /// and drop the blog id from the index so list reads treat it as a gap.
    /// Return Option of `BlogCommandStatus`. If `None`, invalidation failed
    async fn invalidate(&mut self, id: i64) -> Option<BlogCommandStatus> {
        let key = format!("{BLOG_KEY_PREFIX}-{id}");
        info!("Invalidating {} from InMemoryCache", &key);
        self.blogs_index
            .write()
            .expect("blogs index lock poisoned")
            .remove(&id);
        self.blogs_cache.invalidate(&key).await;
        Some(BlogCommandStatus::CacheInvalidated)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::time::{sleep, Duration as SleepDuration};

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

    #[tokio::test]
    async fn test_find_blogs_all_or_nothing_on_partial_cache() {
        // Both Blogs enter the cache and the index; their entries expire while
        // only two of four indexed ids get re-inserted. The list read must
        // fall through (None) instead of answering with the two fresh rows.
        let cache = InMemoryCache::new(3);
        let blogs = vec![
            Blog {
                id: 1,
                name: Some("test".to_string()),
                source: Some(BlogSource::Filesystem),
                filename: Some("test".to_string()),
                body: Some("#Hi".to_string()),
                tags: Some(vec!["tag".to_string()]),
            },
            Blog {
                id: 2,
                name: Some("tast".to_string()),
                source: Some(BlogSource::Filesystem),
                filename: Some("test".to_string()),
                body: Some("#Hi".to_string()),
                tags: Some(vec!["tag".to_string()]),
            },
            Blog {
                id: 3,
                name: Some("tost".to_string()),
                source: Some(BlogSource::Filesystem),
                filename: Some("test".to_string()),
                body: Some("#Hi".to_string()),
                tags: Some(vec!["tag".to_string()]),
            },
            Blog {
                id: 4,
                name: Some("tust".to_string()),
                source: Some(BlogSource::Filesystem),
                filename: Some("test".to_string()),
                body: Some("#Hi".to_string()),
                tags: Some(vec!["tag".to_string()]),
            },
        ];

        for blog in &blogs {
            let insert_status = cache.clone().insert(blog.clone()).await;
            assert_eq!(insert_status.unwrap(), BlogCommandStatus::CacheInserted);
        }

        // Sleep for 4 seconds to pass TTL. The index survives the expiry.
        sleep(SleepDuration::from_secs(4)).await;

        // Refill two of the four ids: ids 1 and 4 get fresh entries, ids 2
        // and 3 stay indexed but expired.
        let refill_fresh = cache
            .clone()
            .find_blogs(BlogsParams {
                start: Some(0),
                end: Some(2),
                tags: None,
            })
            .await;
        assert!(
            refill_fresh.is_none(),
            "partial cache must not answer a list read"
        );
    }
}
