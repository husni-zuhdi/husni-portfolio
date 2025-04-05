use crate::model::blogs::{Blog, BlogDeleted, BlogId, BlogSource};
use async_trait::async_trait;

#[async_trait]
pub trait BlogCommandPort {
    // TODO: instead of manually input
    // why don't we create a struct to input the blog
    // and return BlogStored instead?
    async fn add(
        &mut self,
        id: BlogId,
        name: String,
        filename: String,
        source: BlogSource,
        body: String,
    ) -> Option<Blog>;
    async fn update(
        &mut self,
        id: BlogId,
        name: Option<String>,
        filename: Option<String>,
        source: Option<BlogSource>,
        body: Option<String>,
    ) -> Option<Blog>;
    async fn delete(&mut self, id: BlogId) -> Option<BlogDeleted>;
}
