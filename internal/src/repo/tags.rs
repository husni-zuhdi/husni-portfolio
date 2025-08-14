use crate::model::tags::{Tag, TagCommandStatus, Tags};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(TagRepo);

#[async_trait]
pub trait TagRepo: DynClone {
    async fn find(&self, id: i64) -> Option<Tag>;
    async fn find_all(&self) -> Option<Tags>;
    async fn get_new_id(&self) -> Option<i64>;
    async fn add(&mut self, id: i64, name: String) -> Option<TagCommandStatus>;
    async fn update(&mut self, id: i64, name: Option<String>) -> Option<TagCommandStatus>;
    async fn delete(&mut self, id: i64) -> Option<TagCommandStatus>;
}
