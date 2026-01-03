use crate::model::talks::{Talk, TalkCommandStatus, Talks, TalksParams};
use crate::repo::talks::*;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct TalkDBUseCase {
    pub talk_display_repo: Box<dyn TalkDisplayRepo + Send + Sync>,
    pub talk_operation_repo: Box<dyn TalkOperationRepo + Send + Sync>,
}

#[derive(Clone, Debug)]
pub struct TalkCacheUseCase {
    pub talk_display_repo: Box<dyn TalkDisplayRepo + Send + Sync>,
    pub talk_operation_repo: Box<dyn TalkCacheOperationRepo + Send + Sync>,
}

impl Debug for dyn TalkDisplayRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TalkDisplayRepo")
    }
}

impl Debug for dyn TalkOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TalkOperationRepo")
    }
}

impl Debug for dyn TalkCacheOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TalkCacheOperationRepo")
    }
}

#[async_trait]
impl TalkDisplayRepo for TalkDBUseCase {
    async fn find(&self, id: i64) -> Option<Talk> {
        self.talk_display_repo.find(id).await
    }
    async fn find_talks(&self, params: TalksParams) -> Option<Talks> {
        self.talk_display_repo.find_talks(params).await
    }
}

#[async_trait]
impl TalkOperationRepo for TalkDBUseCase {
    async fn get_new_id(&self) -> Option<i64> {
        self.talk_operation_repo.get_new_id().await
    }
    async fn add(
        &mut self,
        id: i64,
        name: String,
        date: String,
        media_link: Option<String>,
        org_name: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus> {
        self.talk_operation_repo
            .add(id, name, date, media_link, org_name, org_link)
            .await
    }
    async fn update(
        &mut self,
        id: i64,
        name: Option<String>,
        date: Option<String>,
        media_link: Option<String>,
        org_name: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus> {
        self.talk_operation_repo
            .update(id, name, date, media_link, org_name, org_link)
            .await
    }
    async fn delete(&mut self, id: i64) -> Option<TalkCommandStatus> {
        self.talk_operation_repo.delete(id).await
    }
}

impl TalkDBUseCase {
    pub fn new(
        talk_display_repo: Box<dyn TalkDisplayRepo + Send + Sync>,
        talk_operation_repo: Box<dyn TalkOperationRepo + Send + Sync>,
    ) -> TalkDBUseCase {
        TalkDBUseCase {
            talk_display_repo,
            talk_operation_repo,
        }
    }
}

#[async_trait]
impl TalkDisplayRepo for TalkCacheUseCase {
    async fn find(&self, id: i64) -> Option<Talk> {
        self.talk_display_repo.find(id).await
    }
    async fn find_talks(&self, params: TalksParams) -> Option<Talks> {
        self.talk_display_repo.find_talks(params).await
    }
}

#[async_trait]
impl TalkCacheOperationRepo for TalkCacheUseCase {
    async fn insert(&mut self, talk: Talk) -> Option<TalkCommandStatus> {
        self.talk_operation_repo.insert(talk).await
    }
    async fn invalidate(&mut self, id: i64) -> Option<TalkCommandStatus> {
        self.talk_operation_repo.invalidate(id).await
    }
}

impl TalkCacheUseCase {
    pub fn new(
        talk_display_repo: Box<dyn TalkDisplayRepo + Send + Sync>,
        talk_operation_repo: Box<dyn TalkCacheOperationRepo + Send + Sync>,
    ) -> TalkCacheUseCase {
        TalkCacheUseCase {
            talk_display_repo,
            talk_operation_repo,
        }
    }
}
