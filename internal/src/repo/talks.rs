use crate::model::talks::{Talk, TalkCommandStatus, TalkId, Talks, TalksParams};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(TalkRepo);

#[async_trait]
pub trait TalkRepo: DynClone {
    async fn find(&self, id: TalkId) -> Option<Talk>;
    async fn find_talks(&self, params: TalksParams) -> Option<Talks>;
    async fn get_new_id(&self) -> Option<TalkId>;
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
