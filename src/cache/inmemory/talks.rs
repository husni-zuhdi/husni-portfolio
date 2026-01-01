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
