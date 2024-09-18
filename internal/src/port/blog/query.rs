use crate::model::blog::{Blog, BlogEndPage, BlogId, BlogStartPage, BlogStored};
use async_trait::async_trait;

#[async_trait]
pub trait BlogQueryPort {
    async fn find(&self, id: BlogId) -> Option<Blog>;
    async fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage) -> Option<Vec<Blog>>;
    async fn check_id(&self, id: BlogId) -> Option<BlogStored>;
}
