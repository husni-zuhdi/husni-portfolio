use crate::model::blog_tag_mappings::{BlogTagMappingCommandStatus, BlogTagMappings};
use crate::repo::blog_tag_mappings::*;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct BlogTagMappingDBUseCase {
    pub display: Box<dyn BlogTagMappingDisplayRepo + Send + Sync>,
    pub operation: Box<dyn BlogTagMappingOperationRepo + Send + Sync>,
}

#[derive(Clone, Debug)]
pub struct BlogTagMappingCacheUseCase {
    pub display: Box<dyn BlogTagMappingDisplayRepo + Send + Sync>,
    pub operation: Box<dyn BlogTagMappingCacheOperationRepo + Send + Sync>,
}

impl Debug for dyn BlogTagMappingDisplayRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogTagMappingDisplayRepo")
    }
}

impl Debug for dyn BlogTagMappingOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogTagMappingOperationRepo")
    }
}

impl Debug for dyn BlogTagMappingCacheOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogTagMappingCacheOperationRepo")
    }
}

#[async_trait]
impl BlogTagMappingDisplayRepo for BlogTagMappingDBUseCase {
    async fn find_by_blog_id(&self, blog_id: i64) -> Option<BlogTagMappings> {
        self.display.find_by_blog_id(blog_id).await
    }
    async fn find_by_tag_id(&self, tag_id: i64) -> Option<BlogTagMappings> {
        self.display.find_by_tag_id(tag_id).await
    }
}

#[async_trait]
impl BlogTagMappingOperationRepo for BlogTagMappingDBUseCase {
    async fn add(&mut self, blog_id: i64, tag_id: i64) -> Option<BlogTagMappingCommandStatus> {
        self.operation.add(blog_id, tag_id).await
    }
    async fn delete_by_blog_id(&mut self, blog_id: i64) -> Option<BlogTagMappingCommandStatus> {
        self.operation.delete_by_blog_id(blog_id).await
    }
    async fn delete_by_blog_id_and_tag_id(
        &mut self,
        blog_id: i64,
        tag_id: i64,
    ) -> Option<BlogTagMappingCommandStatus> {
        self.operation
            .delete_by_blog_id_and_tag_id(blog_id, tag_id)
            .await
    }
}

#[async_trait]
impl BlogTagMappingDisplayRepo for BlogTagMappingCacheUseCase {
    async fn find_by_blog_id(&self, blog_id: i64) -> Option<BlogTagMappings> {
        self.display.find_by_blog_id(blog_id).await
    }
    async fn find_by_tag_id(&self, tag_id: i64) -> Option<BlogTagMappings> {
        self.display.find_by_tag_id(tag_id).await
    }
}

#[async_trait]
impl BlogTagMappingCacheOperationRepo for BlogTagMappingCacheUseCase {
    async fn insert(&mut self, blog_id: i64, tag_id: i64) -> Option<BlogTagMappingCommandStatus> {
        self.operation.insert(blog_id, tag_id).await
    }
    async fn invalidate(
        &mut self,
        blog_id: i64,
        tag_id: i64,
    ) -> Option<BlogTagMappingCommandStatus> {
        self.operation.invalidate(blog_id, tag_id).await
    }
    async fn invalidate_by_blog_id(&mut self, blog_id: i64) -> Option<BlogTagMappingCommandStatus> {
        self.operation.invalidate_by_blog_id(blog_id).await
    }
}

impl BlogTagMappingDBUseCase {
    pub fn new(
        display: Box<dyn BlogTagMappingDisplayRepo + Send + Sync>,
        operation: Box<dyn BlogTagMappingOperationRepo + Send + Sync>,
    ) -> Self {
        Self { display, operation }
    }
}

impl BlogTagMappingCacheUseCase {
    pub fn new(
        display: Box<dyn BlogTagMappingDisplayRepo + Send + Sync>,
        operation: Box<dyn BlogTagMappingCacheOperationRepo + Send + Sync>,
    ) -> Self {
        Self { display, operation }
    }
}
