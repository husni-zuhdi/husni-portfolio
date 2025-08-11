use crate::model::blog_tag_mappings::BlogTagMappingCommandStatus;
use async_trait::async_trait;

#[async_trait]
pub trait BlogTagMappingCommandPort {
    async fn add(&mut self, blog_id: i64, tag_id: i64) -> Option<BlogTagMappingCommandStatus>;
    async fn delete_by_blog_id(&mut self, blog_id: i64) -> Option<BlogTagMappingCommandStatus>;
}
