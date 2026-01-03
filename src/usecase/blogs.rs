use crate::model::blogs::{Blog, BlogCommandStatus, BlogMetadata, BlogsParams};
use crate::repo::blogs::BlogRepo;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct BlogDBUseCase {
    pub blog_repo: Box<dyn BlogRepo + Send + Sync>,
}

impl Debug for dyn BlogRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogRepo")
    }
}

#[async_trait]
impl BlogRepo for BlogDBUseCase {
    async fn find(&self, id: i64) -> Option<Blog> {
        self.blog_repo.find(id).await
    }
    async fn find_blogs(&self, query_params: BlogsParams) -> Option<Vec<BlogMetadata>> {
        self.blog_repo.find_blogs(query_params).await
    }
    async fn check_id(&self, id: i64) -> Option<BlogCommandStatus> {
        self.blog_repo.check_id(id).await
    }
    async fn get_new_id(&self) -> Option<i64> {
        self.blog_repo.get_new_id().await
    }
    async fn add(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        self.blog_repo.add(blog).await
    }
    async fn update(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        self.blog_repo.update(blog).await
    }
    async fn delete(&mut self, id: i64) -> Option<BlogCommandStatus> {
        self.blog_repo.delete(id).await
    }
}

impl BlogDBUseCase {
    pub fn new(blog_repo: Box<dyn BlogRepo + Send + Sync>) -> BlogDBUseCase {
        BlogDBUseCase { blog_repo }
    }
}
