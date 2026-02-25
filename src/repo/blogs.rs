use crate::model::blogs::{Blog, BlogCommandStatus, BlogsParams};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(BlogDisplayRepo);
clone_trait_object!(BlogOperationRepo);
clone_trait_object!(BlogCacheOperationRepo);

#[async_trait]
pub trait BlogDisplayRepo: DynClone {
    async fn find(&self, id: i64) -> Option<Blog>;
    async fn find_blogs(&self, params: BlogsParams) -> Option<Vec<Blog>>;
}

#[async_trait]
pub trait BlogOperationRepo: DynClone {
    async fn check_id(&self, id: i64) -> Option<BlogCommandStatus>;
    async fn get_new_id(&self) -> Option<i64>;
    async fn add(&mut self, blog: Blog) -> Option<BlogCommandStatus>;
    async fn update(&mut self, blog: Blog) -> Option<BlogCommandStatus>;
    async fn delete(&mut self, id: i64) -> Option<BlogCommandStatus>;
}

#[async_trait]
pub trait BlogCacheOperationRepo: DynClone {
    async fn insert(&mut self, blog: Blog) -> Option<BlogCommandStatus>;
    async fn invalidate(&mut self, id: i64) -> Option<BlogCommandStatus>;
}
