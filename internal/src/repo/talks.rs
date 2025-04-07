use crate::model::talks::{Talk, TalkCommandStatus, TalkEndPage, TalkId, TalkStartPage};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(TalkRepo);

#[async_trait]
pub trait TalkRepo: DynClone {
    async fn find(&self, id: TalkId) -> Option<Talk>;
    async fn find_talks(&self, start: TalkStartPage, end: TalkEndPage) -> Option<Vec<Talk>>;
    async fn add(
        &mut self,
        id: TalkId,
        name: String,
        date: String,
        media_link: Option<String>,
        org_name: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus>;
    async fn update(
        &mut self,
        id: TalkId,
        name: Option<String>,
        date: Option<String>,
        media_link: Option<String>,
        org_name: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus>;
    async fn delete(&mut self, id: TalkId) -> Option<TalkCommandStatus>;
}
