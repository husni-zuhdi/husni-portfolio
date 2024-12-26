use crate::model::blogs::{
    Blog, BlogBody, BlogDeleted, BlogFilename, BlogId, BlogName, BlogSource,
};
use async_trait::async_trait;

#[async_trait]
pub trait BlogCommandPort {
    // TODO: instead of manually input
    // why don't we create a struct to input the blog
    // and return BlogStored instead?
    async fn add(
        &mut self,
        id: BlogId,
        name: BlogName,
        filename: BlogFilename,
        source: BlogSource,
        body: BlogBody,
    ) -> Option<Blog>;
    async fn update(
        &mut self,
        id: BlogId,
        name: Option<BlogName>,
        filename: Option<BlogFilename>,
        source: Option<BlogSource>,
        body: Option<BlogBody>,
    ) -> Option<Blog>;
    async fn delete(&mut self, id: BlogId) -> Option<BlogDeleted>;
}
