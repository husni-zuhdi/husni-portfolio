use crate::model::blog_tag_mappings::{BlogTagMappingCommandStatus, BlogTagMappings};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(BlogTagMappingRepo);

#[async_trait]
pub trait BlogTagMappingRepo: DynClone {
    async fn find_by_blog_id(&self, blog_id: i64) -> Option<BlogTagMappings>;
    async fn find_by_tag_id(&self, tag_id: i64) -> Option<BlogTagMappings>;
    async fn add(&mut self, blog_id: i64, tag_id: i64) -> Option<BlogTagMappingCommandStatus>;
    async fn delete_by_id(&mut self, blog_id: i64) -> Option<BlogTagMappingCommandStatus>;
}
