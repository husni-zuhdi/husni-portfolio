use crate::model::profile::{
    MoveDirection, ProfileCommandStatus, ProfileMeta, ProfilePage, ProfileSection, ProfileSections,
};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(ProfileDisplayRepo);
clone_trait_object!(ProfileOperationRepo);
clone_trait_object!(ProfileCacheOperationRepo);

#[async_trait]
pub trait ProfileDisplayRepo: DynClone {
    async fn find_sections(&self) -> Option<ProfileSections>;
    async fn find_section(&self, id: i64) -> Option<ProfileSection>;
    async fn find_meta(&self) -> Option<ProfileMeta>;
}

#[async_trait]
pub trait ProfileOperationRepo: DynClone {
    async fn get_new_id(&self) -> Option<i64>;
    async fn add(&mut self, section: ProfileSection) -> Option<ProfileCommandStatus>;
    async fn update(&mut self, section: ProfileSection) -> Option<ProfileCommandStatus>;
    async fn delete(&mut self, id: i64) -> Option<ProfileCommandStatus>;
    async fn move_section(
        &mut self,
        id: i64,
        direction: MoveDirection,
    ) -> Option<ProfileCommandStatus>;
    async fn update_meta(&mut self, meta: ProfileMeta) -> Option<ProfileCommandStatus>;
}

#[async_trait]
pub trait ProfileCacheOperationRepo: DynClone {
    async fn insert_page(&mut self, page: ProfilePage) -> Option<ProfileCommandStatus>;
    async fn invalidate_page(&mut self) -> Option<ProfileCommandStatus>;
}
