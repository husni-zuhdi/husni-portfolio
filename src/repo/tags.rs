use crate::model::tags::{Tag, TagCommandStatus, Tags, TagsListParams, TagsSearchParams};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(TagRepo);

#[async_trait]
pub trait TagRepo: DynClone {
    async fn find(&self, id: i64) -> Option<Tag>;
    // TODO: merge find_all_tags to find_tags
    // We don't need two implementation for same API
    async fn find_all_tags(&self) -> Option<Tags>;
    async fn find_tags(&self, params: TagsListParams) -> Option<Tags>;
    async fn search_tags(&self, params: TagsSearchParams) -> Option<Tags>;
    async fn get_new_id(&self) -> Option<i64>;
    async fn add(&mut self, id: i64, name: String) -> Option<TagCommandStatus>;
    async fn update(&mut self, id: i64, name: Option<String>) -> Option<TagCommandStatus>;
    async fn delete(&mut self, id: i64) -> Option<TagCommandStatus>;
}
