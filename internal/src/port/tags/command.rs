use crate::model::tags::TagCommandStatus;
use async_trait::async_trait;

#[async_trait]
pub trait TagCommandPort {
    async fn add(&mut self, id: i64, name: String) -> Option<TagCommandStatus>;
    async fn update(&mut self, id: i64, name: Option<String>) -> Option<TagCommandStatus>;
    async fn delete(&mut self, id: i64) -> Option<TagCommandStatus>;
}
