use crate::model::blogs::{
    Blog, BlogDeleted, BlogEndPage, BlogId, BlogMetadata, BlogSource, BlogStartPage, BlogStored,
};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(BlogRepo);

#[async_trait]
pub trait BlogRepo: DynClone {
    async fn find(&self, id: BlogId) -> Option<Blog>;
    async fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage)
        -> Option<Vec<BlogMetadata>>;
    async fn check_id(&self, id: BlogId) -> Option<BlogStored>;
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
