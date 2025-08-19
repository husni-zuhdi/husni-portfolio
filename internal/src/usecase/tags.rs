use crate::model::tags::{Tag, TagCommandStatus, Tags};
use crate::repo::tags::TagRepo;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct TagUseCase {
    pub tag_repo: Box<dyn TagRepo + Send + Sync>,
}

impl Debug for dyn TagRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TagRepo{:?}", self)
    }
}

#[async_trait]
impl TagRepo for TagUseCase {
    async fn find(&self, id: i64) -> Option<Tag> {
        self.tag_repo.find(id).await
    }
    async fn find_all(&self) -> Option<Tags> {
        self.tag_repo.find_all().await
    }
    async fn get_new_id(&self) -> Option<i64> {
        self.tag_repo.get_new_id().await
    }
    async fn add(&mut self, id: i64, name: String) -> Option<TagCommandStatus> {
        self.tag_repo.add(id, name).await
    }
    async fn update(&mut self, id: i64, name: Option<String>) -> Option<TagCommandStatus> {
        self.tag_repo.update(id, name).await
    }
    async fn delete(&mut self, id: i64) -> Option<TagCommandStatus> {
        self.tag_repo.delete(id).await
    }
}

impl TagUseCase {
    pub fn new(tag_repo: Box<dyn TagRepo + Send + Sync>) -> TagUseCase {
        TagUseCase { tag_repo }
    }
}
