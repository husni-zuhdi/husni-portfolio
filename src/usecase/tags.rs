use crate::model::tags::{Tag, TagCommandStatus, Tags, TagsListParams, TagsSearchParams};
use crate::repo::tags::*;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct TagDBUseCase {
    pub tag_display_repo: Box<dyn TagDisplayRepo + Send + Sync>,
    pub tag_operation_repo: Box<dyn TagOperationRepo + Send + Sync>,
}

#[derive(Clone, Debug)]
pub struct TagCacheUseCase {
    pub tag_display_repo: Box<dyn TagDisplayRepo + Send + Sync>,
    pub tag_operation_repo: Box<dyn TagCacheOperationRepo + Send + Sync>,
}

impl Debug for dyn TagDisplayRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TagDisplayRepo")
    }
}

impl Debug for dyn TagOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TagOperationRepo")
    }
}

impl Debug for dyn TagCacheOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TagCacheOperationRepo")
    }
}

#[async_trait]
impl TagDisplayRepo for TagDBUseCase {
    async fn find(&self, id: i64) -> Option<Tag> {
        self.tag_display_repo.find(id).await
    }
    async fn find_tags(&self, params: TagsListParams) -> Option<Tags> {
        self.tag_display_repo.find_tags(params).await
    }
    async fn search_tags(&self, params: TagsSearchParams) -> Option<Tags> {
        self.tag_display_repo.search_tags(params).await
    }
}

#[async_trait]
impl TagOperationRepo for TagDBUseCase {
    async fn get_new_id(&self) -> Option<i64> {
        self.tag_operation_repo.get_new_id().await
    }
    async fn add(&mut self, id: i64, name: String) -> Option<TagCommandStatus> {
        self.tag_operation_repo.add(id, name).await
    }
    async fn update(&mut self, id: i64, name: Option<String>) -> Option<TagCommandStatus> {
        self.tag_operation_repo.update(id, name).await
    }
    async fn delete(&mut self, id: i64) -> Option<TagCommandStatus> {
        self.tag_operation_repo.delete(id).await
    }
}

#[async_trait]
impl TagCacheOperationRepo for TagCacheUseCase {
    async fn insert(&mut self, talk: Tag) -> Option<TagCommandStatus> {
        self.tag_operation_repo.insert(talk).await
    }
    async fn invalidate(&mut self, id: i64) -> Option<TagCommandStatus> {
        self.tag_operation_repo.invalidate(id).await
    }
}

impl TagDBUseCase {
    pub fn new(
        tag_display_repo: Box<dyn TagDisplayRepo + Send + Sync>,
        tag_operation_repo: Box<dyn TagOperationRepo + Send + Sync>,
    ) -> TagDBUseCase {
        TagDBUseCase {
            tag_display_repo,
            tag_operation_repo,
        }
    }
}

impl TagCacheUseCase {
    pub fn new(
        tag_display_repo: Box<dyn TagDisplayRepo + Send + Sync>,
        tag_operation_repo: Box<dyn TagCacheOperationRepo + Send + Sync>,
    ) -> TagCacheUseCase {
        TagCacheUseCase {
            tag_display_repo,
            tag_operation_repo,
        }
    }
}
