use crate::model::blogs::{Blog, BlogCommandStatus, BlogId};
use async_trait::async_trait;

#[async_trait]
pub trait BlogCommandPort {
    async fn add(&mut self, blog: Blog) -> Option<BlogCommandStatus>;
    async fn update(&mut self, blog: Blog) -> Option<BlogCommandStatus>;
    async fn delete(&mut self, id: BlogId) -> Option<BlogCommandStatus>;
}
