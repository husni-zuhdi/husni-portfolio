use crate::model::tags::Tag;
use async_trait::async_trait;

#[async_trait]
pub trait TagQueryPort {
    async fn find(&self, id: i64) -> Option<Tag>;
    async fn get_new_id(&self) -> Option<i64>;
}
