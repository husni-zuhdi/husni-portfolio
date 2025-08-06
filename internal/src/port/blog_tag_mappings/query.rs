use crate::model::blog_tag_mappings::BlogTagMappings;
use async_trait::async_trait;

#[async_trait]
pub trait BlogTagMappingQueryPort {
    async fn find_by_blog_id(&self, blog_id: i64) -> Option<BlogTagMappings>;
    async fn find_by_tag_id(&self, tag_id: i64) -> Option<BlogTagMappings>;
}
