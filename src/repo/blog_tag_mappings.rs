use crate::model::blog_tag_mappings::{BlogTagMappingCommandStatus, BlogTagMappings};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(BlogTagMappingDisplayRepo);
clone_trait_object!(BlogTagMappingOperationRepo);
clone_trait_object!(BlogTagMappingCacheOperationRepo);

#[async_trait]
pub trait BlogTagMappingDisplayRepo: DynClone {
    async fn find_by_blog_id(&self, blog_id: i64) -> Option<BlogTagMappings>;
    async fn find_by_tag_id(&self, tag_id: i64) -> Option<BlogTagMappings>;
}

#[async_trait]
pub trait BlogTagMappingOperationRepo: DynClone {
    async fn add(&mut self, blog_id: i64, tag_id: i64) -> Option<BlogTagMappingCommandStatus>;
    async fn delete_by_blog_id(&mut self, blog_id: i64) -> Option<BlogTagMappingCommandStatus>;
    async fn delete_by_blog_id_and_tag_id(
        &mut self,
        blog_id: i64,
        tag_id: i64,
    ) -> Option<BlogTagMappingCommandStatus>;
}

#[async_trait]
pub trait BlogTagMappingCacheOperationRepo: DynClone {
    async fn insert(&mut self, blog_id: i64, tag_id: i64) -> Option<BlogTagMappingCommandStatus>;
    async fn invalidate(
        &mut self,
        blog_id: i64,
        tag_id: i64,
    ) -> Option<BlogTagMappingCommandStatus>;
    async fn invalidate_by_blog_id(&mut self, blog_id: i64) -> Option<BlogTagMappingCommandStatus>;
}
