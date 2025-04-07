use crate::model::talks::{Talk, TalkEndPage, TalkId, TalkStartPage};
use async_trait::async_trait;

#[async_trait]
pub trait TalkQueryPort {
    async fn find(&self, id: TalkId) -> Option<Talk>;
    async fn find_talks(&self, start: TalkStartPage, end: TalkEndPage) -> Option<Vec<Talk>>;
}
