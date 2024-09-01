use crate::model::blog::{
    Blog, BlogBody, BlogDeleted, BlogEndPage, BlogFilename, BlogId, BlogName, BlogSource,
    BlogStartPage,
};
use dyn_clone::{clone_trait_object, DynClone};

pub trait BlogRepo: DynClone {
    fn find(&self, id: BlogId) -> Blog;
    fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage) -> Vec<Blog>;
    fn find_all(&self) -> Vec<Blog>;
    fn update(
        &mut self,
        id: BlogId,
        name: Option<BlogName>,
        filename: Option<BlogFilename>,
        source: Option<BlogSource>,
        body: Option<BlogBody>,
    ) -> Blog;
    fn delete(&mut self, id: BlogId) -> BlogDeleted;
}

clone_trait_object!(BlogRepo);
