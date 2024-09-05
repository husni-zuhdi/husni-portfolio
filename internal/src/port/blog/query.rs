use crate::model::blog::{Blog, BlogEndPage, BlogId, BlogStartPage, BlogStored};
use async_trait::async_trait;

#[async_trait]
pub trait BlogQueryPort {
    async fn find(&self, id: BlogId) -> Blog;
    async fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage) -> Vec<Blog>;
    async fn check_id(&self, id: BlogId) -> BlogStored;
}
