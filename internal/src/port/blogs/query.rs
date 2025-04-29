use crate::model::blogs::{Blog, BlogCommandStatus, BlogId, BlogMetadata, BlogsParams};
use async_trait::async_trait;

#[async_trait]
pub trait BlogQueryPort {
    async fn find(&self, id: BlogId) -> Option<Blog>;
    async fn find_blogs(&self, query_params: BlogsParams) -> Option<Vec<BlogMetadata>>;
    async fn check_id(&self, id: BlogId) -> Option<BlogCommandStatus>;
}
