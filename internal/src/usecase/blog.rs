use crate::model::blog::{
    Blog, BlogBody, BlogDeleted, BlogEndPage, BlogFilename, BlogId, BlogName, BlogSource,
    BlogStartPage,
};
use crate::port::blog::{command::BlogQueryCommand, query::BlogQueryPort};
use crate::repo::blog::BlogRepo;
use async_trait::async_trait;

#[derive(Clone)]
pub struct BlogUseCase {
    pub blog_repo: Box<dyn BlogRepo + Send + Sync>,
}

#[async_trait]
impl BlogQueryPort for BlogUseCase {
    async fn find(&self, id: BlogId) -> Blog {
        self.blog_repo.find(id).await
    }
    async fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage) -> Vec<Blog> {
        self.blog_repo.find_blogs(start, end).await
    }
}

#[async_trait]
impl BlogQueryCommand for BlogUseCase {
    async fn add(
        &mut self,
        id: BlogId,
        name: BlogName,
        filename: BlogFilename,
        source: BlogSource,
        body: BlogBody,
    ) -> Blog {
        self.blog_repo.add(id, name, filename, source, body).await
    }
    async fn update(
        &mut self,
        id: BlogId,
        name: Option<BlogName>,
        filename: Option<BlogFilename>,
        source: Option<BlogSource>,
        body: Option<BlogBody>,
    ) -> Blog {
        self.blog_repo
            .update(id, name, filename, source, body)
            .await
    }
    async fn delete(&mut self, id: BlogId) -> BlogDeleted {
        self.blog_repo.delete(id).await
    }
}

impl BlogUseCase {
    pub fn new(blog_repo: Box<dyn BlogRepo + Send + Sync>) -> BlogUseCase {
        BlogUseCase { blog_repo }
    }
}
