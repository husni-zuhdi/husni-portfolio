use crate::cache::inmemory::InMemoryCache;
use crate::model::talks::*;
use crate::repo::talks::{TalkCacheOperationRepo as TalkOperationRepo, TalkDisplayRepo};
use async_trait::async_trait;
use tracing::{debug, info};

const TALK_KEY_PREFIX: &str = "talk";

#[async_trait]
impl TalkDisplayRepo for InMemoryCache {
    /// Find a Talk Cache
    /// Take talk id and return Option of `Talk`. If `None`, no talk was cached
    async fn find(&self, id: i64) -> Option<Talk> {
        debug!("Finding InMemoryCache {TALK_KEY_PREFIX}-{id}");
        let key = format!("{TALK_KEY_PREFIX}-{id}");
        self.talks_cache.get(&key).await
    }
    /// Find a Talk Caches
    /// Take `TalkParams` that contain `start` and `end` then
    /// return Option of `Talks`. `talks` field is reversed (descending ids).
    ///
    /// The read is all-or-nothing: only indexed Talk ids inside the range are
    /// expected. If any indexed id is missing from the cache, the read is a
    /// miss (`None`) so the caller falls through to the database. Ids outside
    /// the index are gaps (Talks that do not exist in the database).
    /// `None` is also returned when no indexed Talk falls inside the range.
    async fn find_talks(&self, params: TalksParams) -> Option<Talks> {
        let (start_seq, end_seq) = Self::sanitized_talks_range(params.start, params.end);
        debug!("Finding InMemoryCache {TALK_KEY_PREFIX}-{start_seq} - {TALK_KEY_PREFIX}-{end_seq}");

        let talks = self.complete_talks_in_range(start_seq, end_seq).await?;
        if talks.is_empty() {
            // Cache is still fresh: nothing indexed inside the range.
            return None;
        }
        Some(Talks { talks })
    }
}

#[async_trait]
impl TalkOperationRepo for InMemoryCache {
    /// Insert Talk Cache
    /// Take a `Talk` object and store it in the `InMemoryCache`
    /// Return Option of `TalkCommandStatus`. If `None`, insertion failed
    async fn insert(&mut self, talk: Talk) -> Option<TalkCommandStatus> {
        let key = format!("{TALK_KEY_PREFIX}-{}", talk.id);
        info!("Inserting {} into InMemoryCache", &key);
        self.talks_cache.insert(key, talk.clone()).await;
        self.talks_index.write().unwrap().insert(talk.id);
        Some(TalkCommandStatus::CacheInserted)
    }
    /// Invalidate Talk Cache
    /// Invalidate (discard value from the cached key) talk cache by talk id
    /// Return Option of `TalkCommandStatus`. If `None`, invalidation failed
    async fn invalidate(&mut self, id: i64) -> Option<TalkCommandStatus> {
        let key = format!("{TALK_KEY_PREFIX}-{id}");
        info!("Invalidating {} from InMemoryCache", &key);
        self.talks_cache.invalidate(&key).await;
        self.talks_index.write().unwrap().remove(&id);
        Some(TalkCommandStatus::CacheInvalidated)
    }
}

impl InMemoryCache {
    /// Sanity-check a talks pagination range into inclusive id bounds.
    fn sanitized_talks_range(start: Option<i64>, end: Option<i64>) -> (i64, i64) {
        let sanitized = TalksParams { start, end }.sanitize();
        (sanitized.start.unwrap() + 1, sanitized.end.unwrap())
    }

    /// Collect the Talks inside `[start, end]` in descending id order, but
    /// only when the cache can answer completely: any indexed id that is
    /// missing from the cache makes the whole read a miss. Overlapping with
    /// the index bounds the scan to known ids only.
    async fn complete_talks_in_range(&self, start: i64, end: i64) -> Option<Vec<Talk>> {
        let mut expected_ids: Vec<i64> = self
            .talks_index
            .read()
            .unwrap()
            .iter()
            .filter(|id| (start..=end).contains(id))
            .copied()
            // Descending id order keeps the historical rendering order of
            // the talks list.
            .collect();
        expected_ids.sort_unstable_by(|a, b| b.cmp(a));

        let mut talks = Vec::with_capacity(expected_ids.len());
        for id in expected_ids {
            match self
                .talks_cache
                .get(&format!("{TALK_KEY_PREFIX}-{id}"))
                .await
            {
                Some(talk) => talks.push(talk),
                None => {
                    debug!("{TALK_KEY_PREFIX}-{id} cache miss: all-or-nothing read fails");
                    return None;
                }
            }
        }
        Some(talks)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_insert_find_and_invalidate_talk() {
        let cache = InMemoryCache::new(3600);
        let test_val = Talk {
            id: 1,
            name: "test".to_string(),
            date: "2025-10-10".to_string(),
            media_link: None,
            org_name: None,
            org_link: None,
        };

        // Insert cache
        let insert_status = cache.clone().insert(test_val.clone()).await;
        assert!(insert_status.is_some(), "talk-1 insertion failed");
        assert_eq!(insert_status.unwrap(), TalkCommandStatus::CacheInserted);

        // Find cache
        let result = cache.clone().find(test_val.id).await;
        assert!(result.is_some(), "talk-1 find operation failed");
        assert_eq!(result.unwrap(), test_val.clone());

        // Invalidate cache
        let invalidate_status = cache.clone().invalidate(test_val.id).await;
        assert!(invalidate_status.is_some(), "talk-1 invalidation failed");
        assert_eq!(
            invalidate_status.unwrap(),
            TalkCommandStatus::CacheInvalidated
        );

        // Find cache after invalidation
        let result = cache.clone().find(test_val.id).await;
        assert!(result.is_none(), "talk-1 find a cache! it shouldn't be!");
    }

    #[tokio::test]
    async fn test_insert_find_and_invalidate_talks() {
        let cache = InMemoryCache::new(3600);
        let test_values = Talks {
            // Intentionally reversed order
            talks: vec![
                Talk {
                    id: 2,
                    name: "tast".to_string(),
                    date: "2025-10-10".to_string(),
                    media_link: Some("https://youtube.com".to_string()),
                    org_name: None,
                    org_link: None,
                },
                Talk {
                    id: 1,
                    name: "test".to_string(),
                    date: "2025-10-10".to_string(),
                    media_link: None,
                    org_name: None,
                    org_link: None,
                },
            ],
        };

        for test_val in &test_values.talks {
            let id = test_val.id;
            // Insert cache
            let insert_status = cache.clone().insert(test_val.clone()).await;
            assert!(
                insert_status.is_some(),
                "{}",
                format!("{TALK_KEY_PREFIX}-{id} insertion failed")
            );
            assert_eq!(insert_status.unwrap(), TalkCommandStatus::CacheInserted);

            // Find cache
            let result = cache.clone().find(id).await;
            assert!(
                result.is_some(),
                "{}",
                format!("{TALK_KEY_PREFIX}-{id} find operation failed")
            );
            assert_eq!(result.unwrap(), test_val.clone());
        }

        // Find talk caches
        let talks_res = cache
            .clone()
            .find_talks(TalksParams {
                start: Some(0),
                end: Some(2),
            })
            .await;
        assert!(talks_res.is_some(), "find_talks operation failed");
        assert_eq!(talks_res.unwrap(), test_values.clone());

        for test_val in &test_values.talks {
            let id = test_val.id;
            // Invalidate cache
            let invalidate_status = cache.clone().invalidate(id).await;
            assert!(
                invalidate_status.is_some(),
                "{}",
                format!("{TALK_KEY_PREFIX}-{id} invalidation failed")
            );
            assert_eq!(
                invalidate_status.unwrap(),
                TalkCommandStatus::CacheInvalidated
            );
        }

        // Find talk caches after invalidated
        let talks_res = cache
            .clone()
            .find_talks(TalksParams {
                start: Some(0),
                end: Some(2),
            })
            .await;
        assert!(talks_res.is_none(), "find_talks operation should failed!");
    }
}
