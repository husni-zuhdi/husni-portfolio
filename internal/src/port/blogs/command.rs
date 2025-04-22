use crate::model::blogs::{Blog, BlogCommandStatus, BlogId};
use async_trait::async_trait;

#[async_trait]
pub trait BlogCommandPort {
    // TODO: instead of manually input
    // why don't we create a struct to input the blog
    async fn add(&mut self, blog: Blog) -> Option<BlogCommandStatus>;
    async fn update(&mut self, blog: Blog) -> Option<BlogCommandStatus>;
    async fn delete(&mut self, id: BlogId) -> Option<BlogCommandStatus>;
}
