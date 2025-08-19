use crate::model::talks::{Talk, TalkCommandStatus, TalkEndPage, TalkId, TalkStartPage};
use crate::repo::talks::TalkRepo;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct TalkUseCase {
    pub talk_repo: Box<dyn TalkRepo + Send + Sync>,
}

impl Debug for dyn TalkRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TalkRepo{:?}", self)
    }
}

#[async_trait]
impl TalkRepo for TalkUseCase {
    async fn find(&self, id: TalkId) -> Option<Talk> {
        self.talk_repo.find(id).await
    }
    async fn find_talks(&self, start: TalkStartPage, end: TalkEndPage) -> Option<Vec<Talk>> {
        self.talk_repo.find_talks(start, end).await
    }
    async fn get_new_id(&self) -> Option<TalkId> {
        self.talk_repo.get_new_id().await
    }
    async fn add(
        &mut self,
        id: TalkId,
        name: String,
        date: String,
        media_link: Option<String>,
        org_name: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus> {
        self.talk_repo
            .add(id, name, date, media_link, org_name, org_link)
            .await
    }
    async fn update(
        &mut self,
        id: TalkId,
        name: Option<String>,
        date: Option<String>,
        media_link: Option<String>,
        org_name: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus> {
        self.talk_repo
            .update(id, name, date, media_link, org_name, org_link)
            .await
    }
    async fn delete(&mut self, id: TalkId) -> Option<TalkCommandStatus> {
        self.talk_repo.delete(id).await
    }
}

impl TalkUseCase {
    pub fn new(talk_repo: Box<dyn TalkRepo + Send + Sync>) -> TalkUseCase {
        TalkUseCase { talk_repo }
    }
}
