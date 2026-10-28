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
    /// return Option of `Tags`. `tags` field is reversed (descending ids).
    ///
    /// The read is all-or-nothing: only indexed Tag ids inside the range are
    /// expected. If any indexed id is missing from the cache, the read is a
    /// miss (`None`) so the caller falls through to the database. Ids outside
    /// the index are gaps (Tags that do not exist in the database).
    /// `None` is also returned when no indexed Tag falls inside the range.
    async fn find_tags(&self, params: TagsListParams) -> Option<Tags> {
        let (start_seq, end_seq) = self.sanitized_range(params.start, params.end);
        debug!("Finding InMemoryCache {TAG_KEY_PREFIX}-{start_seq} - {TAG_KEY_PREFIX}-{end_seq}");

        let tags = self.complete_tags_in_range(start_seq, end_seq).await?;
        if tags.is_empty() {
            // Cache is still fresh: nothing indexed inside the range.
            return None;
        }
        Some(Tags { tags })
    }
    /// Search Tag Caches
    /// Take `TagSearchParams` that contain `start` and `end` and a `query`
    /// then return Option of `Tags`. `tags` field is reversed (descending
    /// ids).
    ///
    /// The read is all-or-nothing like `find_tags`: the cached answer is
    /// trusted only when every indexed Tag inside the range is present. The
    /// query filter is applied after; an empty match is a valid `Some`
    /// answer, mirroring the database read contract.
    async fn search_tags(&self, params: TagsSearchParams) -> Option<Tags> {
        let (start_seq, end_seq) = self.sanitized_range(params.start, params.end);
        let query = params.query;
        debug!("Finding InMemoryCache {TAG_KEY_PREFIX}-{start_seq} - {TAG_KEY_PREFIX}-{end_seq} containing {query}");

        let tags = self
            .complete_tags_in_range(start_seq, end_seq)
            .await?
            .into_iter()
            .filter(|tag| tag.name.contains(&query))
            .collect::<Vec<Tag>>();
        Some(Tags { tags })
    }
}

#[async_trait]
impl TagOperationRepo for InMemoryCache {
    /// Insert Tag Cache
    /// Take a `Tag` object and store it in the `InMemoryCache`
    /// Return Option of `TagCommandStatus`. If `None`, insertion failed
    async fn insert(&mut self, tag: Tag) -> Option<TagCommandStatus> {
        let key = format!("{TAG_KEY_PREFIX}-{}", tag.id);
        info!("Inserting {} into InMemoryCache", &key);
        self.tags_cache.insert(key, tag.clone()).await;
        self.tags_index.write().unwrap().insert(tag.id);
        Some(TagCommandStatus::CacheInserted)
    }
    /// Invalidate Tag Cache
    /// Invalidate (discard value from the cached key) tag cache by tag id
    /// Return Option of `TagCommandStatus`. If `None`, invalidation failed
    async fn invalidate(&mut self, id: i64) -> Option<TagCommandStatus> {
        let key = format!("{TAG_KEY_PREFIX}-{id}");
        info!("Invalidating {} from InMemoryCache", &key);
        self.tags_cache.invalidate(&key).await;
        self.tags_index.write().unwrap().remove(&id);
        Some(TagCommandStatus::CacheInvalidated)
    }
}

impl InMemoryCache {
    /// Sanity-check a tags pagination range into inclusive id bounds.
    fn sanitized_range(&self, start: Option<i64>, end: Option<i64>) -> (i64, i64) {
        let sanitized = TagsListParams { start, end }.sanitize();
        (sanitized.start.unwrap() + 1, sanitized.end.unwrap())
    }

    /// Collect the Tags inside `[start, end]` in descending id order, but
    /// only when the cache can answer completely: any indexed id that is
    /// missing from the cache makes the whole read a miss. Overlapping with
    /// the index bounds the scan to known ids only.
    async fn complete_tags_in_range(&self, start: i64, end: i64) -> Option<Vec<Tag>> {
        let mut expected_ids: Vec<i64> = self
            .tags_index
            .read()
            .unwrap()
            .iter()
            .filter(|id| (start..=end).contains(id))
            .copied()
            // Descending id order keeps the historical rendering order of
            // the tags list.
            .collect();
        expected_ids.sort_unstable_by(|a, b| b.cmp(a));

        let mut tags = Vec::with_capacity(expected_ids.len());
        for id in expected_ids {
            match self.tags_cache.get(&format!("{TAG_KEY_PREFIX}-{id}")).await {
                Some(tag) => tags.push(tag),
                None => {
                    debug!("{TAG_KEY_PREFIX}-{id} cache miss: all-or-nothing read fails");
                    return None;
                }
            }
        }
        Some(tags)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_insert_find_and_invalidate_tag() {
        let cache = InMemoryCache::new(3600);
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
        let cache = InMemoryCache::new(3600);
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
