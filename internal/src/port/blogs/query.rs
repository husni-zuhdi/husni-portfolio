use crate::model::blogs::{Blog, BlogEndPage, BlogId, BlogMetadata, BlogStartPage, BlogStored};
use async_trait::async_trait;

#[async_trait]
pub trait BlogQueryPort {
    async fn find(&self, id: BlogId) -> Option<Blog>;
    async fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage)
        -> Option<Vec<BlogMetadata>>;
    async fn check_id(&self, id: BlogId) -> Option<BlogStored>;
}
