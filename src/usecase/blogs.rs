use crate::model::blogs::{Blog, BlogCommandStatus, BlogsParams};
use crate::repo::blogs::*;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct BlogDBUseCase {
    pub blog_display_repo: Box<dyn BlogDisplayRepo + Send + Sync>,
    pub blog_operation_repo: Box<dyn BlogOperationRepo + Send + Sync>,
}

#[derive(Clone, Debug)]
pub struct BlogCacheUseCase {
    pub blog_display_repo: Box<dyn BlogDisplayRepo + Send + Sync>,
    pub blog_operation_repo: Box<dyn BlogCacheOperationRepo + Send + Sync>,
}

impl Debug for dyn BlogDisplayRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogDisplayRepo")
    }
}

impl Debug for dyn BlogOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogOperationRepo")
    }
}

impl Debug for dyn BlogCacheOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogCacheOperationRepo")
    }
}

#[async_trait]
impl BlogDisplayRepo for BlogDBUseCase {
    async fn find(&self, id: i64) -> Option<Blog> {
        self.blog_display_repo.find(id).await
    }
    async fn find_blogs(&self, params: BlogsParams) -> Option<Vec<Blog>> {
        self.blog_display_repo.find_blogs(params).await
    }
}

#[async_trait]
impl BlogOperationRepo for BlogDBUseCase {
    async fn check_id(&self, id: i64) -> Option<BlogCommandStatus> {
        self.blog_operation_repo.check_id(id).await
    }
    async fn get_new_id(&self) -> Option<i64> {
        self.blog_operation_repo.get_new_id().await
    }
    async fn add(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        self.blog_operation_repo.add(blog).await
    }
    async fn update(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        self.blog_operation_repo.update(blog).await
    }
    async fn delete(&mut self, id: i64) -> Option<BlogCommandStatus> {
        self.blog_operation_repo.delete(id).await
    }
}

#[async_trait]
impl BlogDisplayRepo for BlogCacheUseCase {
    async fn find(&self, id: i64) -> Option<Blog> {
        self.blog_display_repo.find(id).await
    }
    async fn find_blogs(&self, params: BlogsParams) -> Option<Vec<Blog>> {
        self.blog_display_repo.find_blogs(params).await
    }
}

#[async_trait]
impl BlogCacheOperationRepo for BlogCacheUseCase {
    async fn insert(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        self.blog_operation_repo.insert(blog).await
    }
    async fn invalidate(&mut self, id: i64) -> Option<BlogCommandStatus> {
        self.blog_operation_repo.invalidate(id).await
    }
}

impl BlogDBUseCase {
    pub fn new(
        blog_display_repo: Box<dyn BlogDisplayRepo + Send + Sync>,
        blog_operation_repo: Box<dyn BlogOperationRepo + Send + Sync>,
    ) -> Self {
        Self {
            blog_display_repo,
            blog_operation_repo,
        }
    }
}

impl BlogCacheUseCase {
    pub fn new(
        blog_display_repo: Box<dyn BlogDisplayRepo + Send + Sync>,
        blog_operation_repo: Box<dyn BlogCacheOperationRepo + Send + Sync>,
    ) -> Self {
        Self {
            blog_display_repo,
            blog_operation_repo,
        }
    }
}
