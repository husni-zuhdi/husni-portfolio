use crate::model::blogs::{Blog, BlogCommandStatus, BlogMetadata, BlogsParams};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(BlogRepo);

#[async_trait]
pub trait BlogRepo: DynClone {
    async fn find(&self, id: i64) -> Option<Blog>;
    async fn find_blogs(&self, query_params: BlogsParams) -> Option<Vec<BlogMetadata>>;
    async fn check_id(&self, id: i64) -> Option<BlogCommandStatus>;
    async fn get_new_id(&self) -> Option<i64>;
    async fn add(&mut self, blog: Blog) -> Option<BlogCommandStatus>;
    async fn update(&mut self, blog: Blog) -> Option<BlogCommandStatus>;
    async fn delete(&mut self, id: i64) -> Option<BlogCommandStatus>;
}
