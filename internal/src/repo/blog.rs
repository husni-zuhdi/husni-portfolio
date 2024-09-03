use crate::model::blog::{
    Blog, BlogBody, BlogDeleted, BlogEndPage, BlogFilename, BlogId, BlogName, BlogSource,
    BlogStartPage,
};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(BlogRepo);

#[async_trait]
pub trait BlogRepo: DynClone {
    async fn find(&self, id: BlogId) -> Blog;
    async fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage) -> Vec<Blog>;
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
