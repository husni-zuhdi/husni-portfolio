use crate::model::blogs::{
    Blog, BlogCommandStatus, BlogEndPage, BlogId, BlogMetadata, BlogSource, BlogStartPage,
};
use crate::port::blogs::{command::BlogCommandPort, query::BlogQueryPort};
use crate::repo::blogs::BlogRepo;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct BlogUseCase {
    pub blog_repo: Box<dyn BlogRepo + Send + Sync>,
}

impl Debug for dyn BlogRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogRepo{:?}", self)
    }
}

#[async_trait]
impl BlogQueryPort for BlogUseCase {
    async fn find(&self, id: BlogId) -> Option<Blog> {
        self.blog_repo.find(id).await
    }
    async fn find_blogs(
        &self,
        start: BlogStartPage,
        end: BlogEndPage,
    ) -> Option<Vec<BlogMetadata>> {
        self.blog_repo.find_blogs(start, end).await
    }
    async fn check_id(&self, id: BlogId) -> Option<BlogCommandStatus> {
        self.blog_repo.check_id(id).await
    }
}

#[async_trait]
impl BlogCommandPort for BlogUseCase {
    async fn add(
        &mut self,
        id: BlogId,
        name: String,
        filename: String,
        source: BlogSource,
        body: String,
    ) -> Option<BlogCommandStatus> {
        self.blog_repo.add(id, name, filename, source, body).await
    }
    async fn update(
        &mut self,
        id: BlogId,
        name: Option<String>,
        filename: Option<String>,
        source: Option<BlogSource>,
        body: Option<String>,
    ) -> Option<BlogCommandStatus> {
        self.blog_repo
            .update(id, name, filename, source, body)
            .await
    }
    async fn delete(&mut self, id: BlogId) -> Option<BlogCommandStatus> {
        self.blog_repo.delete(id).await
    }
}

impl BlogUseCase {
    pub fn new(blog_repo: Box<dyn BlogRepo + Send + Sync>) -> BlogUseCase {
        BlogUseCase { blog_repo }
    }
}
