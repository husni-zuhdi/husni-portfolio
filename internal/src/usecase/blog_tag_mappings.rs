use crate::model::blog_tag_mappings::{BlogTagMappingCommandStatus, BlogTagMappings};
use crate::port::blog_tag_mappings::{
    command::BlogTagMappingCommandPort, query::BlogTagMappingQueryPort,
};
use crate::repo::blog_tag_mappings::BlogTagMappingRepo;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct BlogTagMappingUseCase {
    pub blog_tag_mapping_repo: Box<dyn BlogTagMappingRepo + Send + Sync>,
}

impl Debug for dyn BlogTagMappingRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlogTagMappingRepo{:?}", self)
    }
}

#[async_trait]
impl BlogTagMappingQueryPort for BlogTagMappingUseCase {
    async fn find_by_blog_id(&self, blog_id: i64) -> Option<BlogTagMappings> {
        self.blog_tag_mapping_repo.find_by_blog_id(blog_id).await
    }
    async fn find_by_tag_id(&self, tag_id: i64) -> Option<BlogTagMappings> {
        self.blog_tag_mapping_repo.find_by_tag_id(tag_id).await
    }
}

#[async_trait]
impl BlogTagMappingCommandPort for BlogTagMappingUseCase {
    async fn add(&mut self, blog_id: i64, tag_id: i64) -> Option<BlogTagMappingCommandStatus> {
        self.blog_tag_mapping_repo.add(blog_id, tag_id).await
    }
    async fn delete_by_blog_id(&mut self, blog_id: i64) -> Option<BlogTagMappingCommandStatus> {
        self.blog_tag_mapping_repo.delete_by_blog_id(blog_id).await
    }
}

impl BlogTagMappingUseCase {
    pub fn new(
        blog_tag_mapping_repo: Box<dyn BlogTagMappingRepo + Send + Sync>,
    ) -> BlogTagMappingUseCase {
        BlogTagMappingUseCase {
            blog_tag_mapping_repo,
        }
    }
}
