use crate::model::blog::{Blog, BlogMetadata};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(ApiRepo);

#[async_trait]
pub trait ApiRepo: DynClone {
    async fn list_metadata(&self) -> Vec<BlogMetadata>;
    async fn fetch(&self, metadata: BlogMetadata) -> Blog;
}
