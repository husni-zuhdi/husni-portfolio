pub mod talks;

use moka::future::Cache;
use std::time::Duration;

use crate::model::talks::Talk;

#[derive(Clone)]
pub struct InMemoryCache {
    pub talks_cache: Cache<String, Talk>,
}

impl InMemoryCache {
    pub async fn new(ttl: i64) -> InMemoryCache {
        let talks_cache = Cache::builder()
            // Set time to live from the CACHE_TTL envar
            .time_to_live(Duration::from_secs(ttl as u64))
            // Weigher to set K and V varaibles type
            .weigher(|_key: &String, value: &Talk| -> u32 { value.data_size() })
            // Set max cache capacity to 32MiB
            .max_capacity(32 * 1024 * 1024)
            .build();
        InMemoryCache { talks_cache }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::time::{sleep, Duration as SleepDuration};

    #[tokio::test]
    async fn test_talks_im_cache_with_ttl() {
        let cache = InMemoryCache::new(3600);
        let test_key = "talk-1".to_string();
        let test_val = Talk {
            id: 1,
            name: "test".to_string(),
            date: "2025-10-10".to_string(),
            media_link: None,
            org_name: None,
            org_link: None,
        };
        let talks_cache = cache.await.talks_cache;

        // Insert cache
        let _ = talks_cache
            .clone()
            .insert(test_key.clone(), test_val.clone())
            .await;
        // Get cache
        assert!(
            talks_cache.clone().get(&test_key).await.is_some(),
            "talk-1 cache is None"
        );
        assert_eq!(talks_cache.clone().get(&test_key).await.unwrap(), test_val);
        // Invalidate cache
        let _ = talks_cache.clone().invalidate(&test_key).await;
        assert_eq!(talks_cache.get(&test_key).await, None);
    }

    #[tokio::test]
    async fn test_talks_im_cache_with_short_ttl() {
        let cache = InMemoryCache::new(3);
        let test_key = "talk-1".to_string();
        let test_val = Talk {
            id: 1,
            name: "test".to_string(),
            date: "2025-10-10".to_string(),
            media_link: None,
            org_name: None,
            org_link: None,
        };
        let talks_cache = cache.await.talks_cache;

        // Insert cache
        let _ = talks_cache
            .clone()
            .insert(test_key.clone(), test_val.clone())
            .await;
        // Sleep for 6 seconds to pass TTL
        sleep(SleepDuration::from_secs(4)).await;
        // Get cache
        assert!(
            talks_cache.clone().get(&test_key).await.is_none(),
            "talk-1 cache is Some! It should be None"
        );

        // Re-insert cache
        let _ = talks_cache
            .clone()
            .insert(test_key.clone(), test_val.clone())
            .await;
        // Get cache
        assert!(
            talks_cache.clone().get(&test_key).await.is_some(),
            "talk-1 cache is None! There should be a cache!"
        );
    }
}
