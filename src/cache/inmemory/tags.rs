use crate::cache::inmemory::InMemoryCache;
use crate::model::tags::*;
use crate::repo::tags::{TagCacheOperationRepo as TagOperationRepo, TagDisplayRepo};
use async_trait::async_trait;
use tracing::{debug, info};

const TAG_KEY_PREFIX: &str = "tag";

#[async_trait]
impl TagDisplayRepo for InMemoryCache {
    /// Find a Tag Cache
    /// Take tag id and return Option of `Tag`. If `None`, no tag was cached
    async fn find(&self, id: i64) -> Option<Tag> {
        debug!("Finding InMemoryCache {TAG_KEY_PREFIX}-{id}");
        let key = format!("{TAG_KEY_PREFIX}-{id}");
        self.tags_cache.get(&key).await
    }
    /// Find a Tag Caches
    /// Take `TagListParams` that contain `start` and `end` then
    /// return Option of `Tags`. `tags` filed is always reversed.
    /// if `None`, no tags within `TagParams` was cached
    async fn find_tags(&self, params: TagsListParams) -> Option<Tags> {
        let start_seq = params.start.unwrap() + 1;
        let end_seq = params.end.unwrap();
        debug!("Finding InMemoryCache {TAG_KEY_PREFIX}-{start_seq} - {TAG_KEY_PREFIX}-{end_seq}");

        let mut tags = Vec::new();
        // rev() method to reverse Tag order
        // TODO: Observe the effect of Tag order reversal
        // I forgot why I need to reverse the order in Talk cache
        for id in (start_seq..=end_seq).rev() {
            let value = self.find(id).await;
            if value.is_none() {
                debug!("{TAG_KEY_PREFIX}-{id} cache miss");
                continue;
            }

            debug!("{TAG_KEY_PREFIX}-{id} cache hit");
            tags.push(value.unwrap());
        }

        // If Cache is still fresh, return None
        if tags.is_empty() {
            return None;
        }
        Some(Tags { tags })
    }
    /// Search Tag Caches
    /// Take `TagSearchParams` that contain `start` and `end` then
    /// return Option of `Tags`. `tags` filed is always reversed.
    /// if `None`, no tags within `TagParams` was cached
    async fn search_tags(&self, params: TagsSearchParams) -> Option<Tags> {
        let start_seq = params.start.unwrap() + 1;
        let end_seq = params.end.unwrap();
        let query = params.query;
        debug!("Finding InMemoryCache {TAG_KEY_PREFIX}-{start_seq} - {TAG_KEY_PREFIX}-{end_seq} containing {query}");

        let mut tags = Vec::new();
        // rev() method to reverse Tag order
        // TODO: Observe the effect of Tag order reversal
        // I forgot why I need to reverse the order in Talk cache
        for id in (start_seq..=end_seq).rev() {
            let value = self.find(id).await;
            if value.is_none() {
                debug!("{TAG_KEY_PREFIX}-{id} cache miss");
                continue;
            }

            if !value.clone().unwrap().name.contains(&query)
            //|| (value.clone().unwrap().name.as_str() != &query)
            {
                debug!("{TAG_KEY_PREFIX}-{id} ignored");
                continue;
            }

            debug!("{TAG_KEY_PREFIX}-{id} cache hit");
            tags.push(value.clone().unwrap());
        }

        // If Cache is still fresh, return None
        if tags.is_empty() {
            return None;
        }

        // Filter tags to contain the query
        let _ = tags.iter().filter(|tag| tag.name.contains(&query));

        Some(Tags { tags })
    }
}

#[async_trait]
impl TagOperationRepo for InMemoryCache {
    /// Insert Tag Cache
    /// Take a `Tag` object and store it in the `InMemoryCache`
    /// Return Option of `TagCommandStatus`. If `None`, insertion failed
    async fn insert(&mut self, tag: Tag) -> Option<TagCommandStatus> {
        let key = format!("{TAG_KEY_PREFIX}-{}", &tag.id);
        info!("Inserting {} into InMemoryCache", &key);
        self.tags_cache.insert(key, tag).await;
        Some(TagCommandStatus::CacheInserted)
    }
    /// Invalidate Tag Cache
    /// Invalidate (discard value from the cached key) tag cache by tag id
    /// Return Option of `TagCommandStatus`. If `None`, invalidation failed
    async fn invalidate(&mut self, id: i64) -> Option<TagCommandStatus> {
        let key = format!("{TAG_KEY_PREFIX}-{id}");
        info!("Invalidating {} from InMemoryCache", &key);
        self.tags_cache.invalidate(&key).await;
        Some(TagCommandStatus::CacheInvalidated)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_insert_find_and_invalidate_tag() {
        let cache = InMemoryCache::new(3600).await;
        let test_val = Tag {
            id: 1,
            name: "test".to_string(),
        };

        // Insert cache
        let insert_status = cache.clone().insert(test_val.clone()).await;
        assert!(insert_status.is_some(), "tag-1 insertion failed");
        assert_eq!(insert_status.unwrap(), TagCommandStatus::CacheInserted);

        // Find cache
        let result = cache.clone().find(test_val.id).await;
        assert!(result.is_some(), "tag-1 find operation failed");
        assert_eq!(result.unwrap(), test_val.clone());

        // Invalidate cache
        let invalidate_status = cache.clone().invalidate(test_val.id).await;
        assert!(invalidate_status.is_some(), "tag-1 invalidation failed");
        assert_eq!(
            invalidate_status.unwrap(),
            TagCommandStatus::CacheInvalidated
        );

        // Find cache after invalidation
        let result = cache.clone().find(test_val.id).await;
        assert!(result.is_none(), "tag-1 find a cache! it shouldn't be!");
    }

    #[tokio::test]
    async fn test_insert_find_and_invalidate_tags() {
        let cache = InMemoryCache::new(3600).await;
        let test_values = Tags {
            // Intentionally reversed order
            tags: vec![
                Tag {
                    id: 2,
                    name: "tast".to_string(),
                },
                Tag {
                    id: 1,
                    name: "test".to_string(),
                },
            ],
        };

        for test_val in &test_values.tags {
            let id = test_val.id;
            // Insert cache
            let insert_status = cache.clone().insert(test_val.clone()).await;
            assert!(
                insert_status.is_some(),
                "{}",
                format!("{TAG_KEY_PREFIX}-{id} insertion failed")
            );
            assert_eq!(insert_status.unwrap(), TagCommandStatus::CacheInserted);

            // Find cache
            let result = cache.clone().find(id).await;
            assert!(
                result.is_some(),
                "{}",
                format!("{TAG_KEY_PREFIX}-{id} find operation failed")
            );
            assert_eq!(result.unwrap(), test_val.clone());
        }

        // Find tag caches
        let tags_res = cache
            .clone()
            .find_tags(TagsListParams {
                start: Some(0),
                end: Some(2),
            })
            .await;
        assert!(tags_res.is_some(), "find_tags operation failed");
        assert_eq!(tags_res.unwrap(), test_values.clone());

        for test_val in &test_values.tags {
            let id = test_val.id;
            // Invalidate cache
            let invalidate_status = cache.clone().invalidate(id).await;
            assert!(
                invalidate_status.is_some(),
                "{}",
                format!("{TAG_KEY_PREFIX}-{id} invalidation failed")
            );
            assert_eq!(
                invalidate_status.unwrap(),
                TagCommandStatus::CacheInvalidated
            );
        }

        // Find tag caches after invalidated
        let tags_res = cache
            .clone()
            .find_tags(TagsListParams {
                start: Some(0),
                end: Some(2),
            })
            .await;
        assert!(tags_res.is_none(), "find_tags operation should failed!");
    }
}
