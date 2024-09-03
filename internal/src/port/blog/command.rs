use crate::model::blog::{Blog, BlogBody, BlogDeleted, BlogFilename, BlogId, BlogName, BlogSource};
use async_trait::async_trait;

#[async_trait]
pub trait BlogCommandPort {
    async fn add(
        &mut self,
        id: BlogId,
        name: BlogName,
        filename: BlogFilename,
        source: BlogSource,
        body: BlogBody,
    ) -> Blog;
    async fn update(
        &mut self,
        id: BlogId,
        name: Option<BlogName>,
        filename: Option<BlogFilename>,
        source: Option<BlogSource>,
        body: Option<BlogBody>,
    ) -> Blog;
    async fn delete(&mut self, id: BlogId) -> BlogDeleted;
}
