use crate::model::tags::{Tag, TagCommandStatus, Tags, TagsListParams, TagsSearchParams};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(TagDisplayRepo);
clone_trait_object!(TagOperationRepo);
clone_trait_object!(TagCacheOperationRepo);

#[async_trait]
pub trait TagDisplayRepo: DynClone {
    async fn find(&self, id: i64) -> Option<Tag>;
    async fn find_tags(&self, params: TagsListParams) -> Option<Tags>;
    async fn search_tags(&self, params: TagsSearchParams) -> Option<Tags>;
}

#[async_trait]
pub trait TagOperationRepo: DynClone {
    async fn get_new_id(&self) -> Option<i64>;
    async fn add(&mut self, id: i64, name: String) -> Option<TagCommandStatus>;
    async fn update(&mut self, id: i64, name: Option<String>) -> Option<TagCommandStatus>;
    async fn delete(&mut self, id: i64) -> Option<TagCommandStatus>;
}

#[async_trait]
pub trait TagCacheOperationRepo: DynClone {
    async fn insert(&mut self, tag: Tag) -> Option<TagCommandStatus>;
    async fn invalidate(&mut self, id: i64) -> Option<TagCommandStatus>;
}
