use crate::model::talks::{Talk, TalkCommandStatus, Talks, TalksParams};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(TalkDisplayRepo);
clone_trait_object!(TalkOperationRepo);
clone_trait_object!(TalkCacheOperationRepo);

#[async_trait]
pub trait TalkDisplayRepo: DynClone {
    async fn find(&self, id: i64) -> Option<Talk>;
    async fn find_talks(&self, params: TalksParams) -> Option<Talks>;
}

#[async_trait]
pub trait TalkOperationRepo: DynClone {
    async fn get_new_id(&self) -> Option<i64>;
    async fn add(
        &mut self,
        id: i64,
        name: String,
        date: String,
        media_link: Option<String>,
        org_name: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus>;
    async fn update(
        &mut self,
        id: i64,
        name: Option<String>,
        date: Option<String>,
        media_link: Option<String>,
        org_name: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus>;
    async fn delete(&mut self, id: i64) -> Option<TalkCommandStatus>;
}

#[async_trait]
pub trait TalkCacheOperationRepo: DynClone {
    async fn insert(&mut self, talk: Talk) -> Option<TalkCommandStatus>;
    async fn invalidate(&mut self, id: i64) -> Option<TalkCommandStatus>;
}
