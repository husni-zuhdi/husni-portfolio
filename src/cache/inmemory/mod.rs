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
            .weigher(|_key: &String, value: &Talk| -> u32 {
                (size_of_val(&value.id)
                    + size_of_val(&value.name)
                    + size_of_val(&value.date)
                    + size_of_val(&value.org_name)
                    + size_of_val(&value.org_link)
                    + size_of_val(&value.media_link)) as u32
            })
            // Set max cache capacity to 32MiB
            .max_capacity(32 * 1024 * 1024)
            .build();
        InMemoryCache { talks_cache }
    }
}
