use crate::model::blog::{Blog, BlogMetadata};
use async_trait::async_trait;

#[async_trait]
pub trait ApiQueryPort {
    async fn list_metadata(&self) -> Vec<BlogMetadata>;
    async fn fetch(&self, metadata: BlogMetadata) -> Blog;
}
