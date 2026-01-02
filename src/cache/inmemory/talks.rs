use crate::cache::inmemory::InMemoryCache;
use crate::model::talks::*;
use crate::repo::talks::{TalkCacheOperationRepo as TalkOperationRepo, TalkDisplayRepo};
use async_trait::async_trait;
use tracing::{debug, info};

#[async_trait]
impl TalkDisplayRepo for InMemoryCache {
    async fn find(&self, id: i64) -> Option<Talk> {
        info!("Looking Talk with id {id} in InMemoryCache");
        let key = format!("talk-{id}");
        self.talks_cache.get(&key).await
    }
    async fn find_talks(&self, params: TalksParams) -> Option<Talks> {
        let start_seq = params.start.unwrap() + 1;
        let end_seq = params.end.unwrap();
        info!("Looking Talks with id started at {start_seq} to {end_seq} in InMemoryCache");

        let mut talks = Vec::new();
        // rev() method to reverse Talk order
        for id in (start_seq..=end_seq).rev() {
            let key = format!("talk-{id}");
            let value = self.talks_cache.get(&key).await;
            if value.is_none() {
                debug!("Talk with id {id} is not cached");
                continue;
            }

            talks.push(value.unwrap());
        }

        // If Cache is still fresh, return None
        if talks.is_empty() {
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
        info!("Inserting Talk with id {} into InMemoryCache", &talk.id);
        let key = format!("talk-{}", &talk.id);
        self.talks_cache.insert(key, talk).await;
        Some(TalkCommandStatus::CacheInserted)
    }
    async fn invalidate(&mut self, id: i64) -> Option<TalkCommandStatus> {
        info!("Invalidating Talk with id {id} into InMemoryCache");
        let key = format!("talk-{id}");
        self.talks_cache.invalidate(&key).await;
        Some(TalkCommandStatus::CacheInvalidated)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_insert_find_and_invalidate_talk() {
        let cache = InMemoryCache::new(3600).await;
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
        let cache = InMemoryCache::new(3600).await;
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
                format!("Talk-{id} insertion failed")
            );
            assert_eq!(insert_status.unwrap(), TalkCommandStatus::CacheInserted);

            // Find cache
            let result = cache.clone().find(id).await;
            assert!(
                result.is_some(),
                "{}",
                format!("talk-{id} find operation failed")
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
                format!("talk-{id} invalidation failed")
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
