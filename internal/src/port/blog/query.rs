use crate::model::blog::{Blog, BlogEndPage, BlogId, BlogStartPage};
use async_trait::async_trait;

#[async_trait]
pub trait BlogQueryPort {
    async fn find(&mut self, id: BlogId) -> Blog;
    async fn find_blogs(&mut self, start: BlogStartPage, end: BlogEndPage) -> Vec<Blog>;
}
