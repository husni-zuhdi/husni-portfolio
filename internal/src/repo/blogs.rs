use crate::model::blogs::{
    Blog, BlogCommandStatus, BlogEndPage, BlogId, BlogMetadata, BlogStartPage,
};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(BlogRepo);

#[async_trait]
pub trait BlogRepo: DynClone {
    async fn find(&self, id: BlogId) -> Option<Blog>;
    async fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage)
        -> Option<Vec<BlogMetadata>>;
    async fn check_id(&self, id: BlogId) -> Option<BlogCommandStatus>;
    async fn add(&mut self, blog: Blog) -> Option<BlogCommandStatus>;
    async fn update(&mut self, blog: Blog) -> Option<BlogCommandStatus>;
    async fn delete(&mut self, id: BlogId) -> Option<BlogCommandStatus>;
}
