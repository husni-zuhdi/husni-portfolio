use crate::model::talks::{TalkCommandStatus, TalkId};
use async_trait::async_trait;

#[async_trait]
pub trait TalkCommandPort {
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
