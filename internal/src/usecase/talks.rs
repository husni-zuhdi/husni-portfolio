use crate::model::talks::{Talk, TalkCommandStatus, TalkEndPage, TalkId, TalkStartPage};
use crate::port::talks::{command::TalkCommandPort, query::TalkQueryPort};
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
impl TalkQueryPort for TalkUseCase {
    async fn find(&self, id: TalkId) -> Option<Talk> {
        self.talk_repo.find(id).await
    }
    async fn find_talks(&self, start: TalkStartPage, end: TalkEndPage) -> Option<Vec<Talk>> {
        self.talk_repo.find_talks(start, end).await
    }
}

#[async_trait]
impl TalkCommandPort for TalkUseCase {
    async fn add(
        &mut self,
        id: TalkId,
        name: String,
        media_link: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus> {
        self.talk_repo.add(id, name, media_link, org_link).await
    }
    async fn update(
        &mut self,
        id: TalkId,
        name: Option<String>,
        media_link: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus> {
        self.talk_repo.update(id, name, media_link, org_link).await
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
