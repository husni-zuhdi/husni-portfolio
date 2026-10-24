use crate::model::profile::{
    MoveDirection, ProfileCommandStatus, ProfileMeta, ProfilePage, ProfileSection, ProfileSections,
};
use crate::repo::profile::*;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct ProfileDBUseCase {
    pub profile_display_repo: Box<dyn ProfileDisplayRepo + Send + Sync>,
    pub profile_operation_repo: Box<dyn ProfileOperationRepo + Send + Sync>,
}

#[derive(Clone, Debug)]
pub struct ProfileCacheUseCase {
    pub profile_display_repo: Box<dyn ProfileDisplayRepo + Send + Sync>,
    pub profile_operation_repo: Box<dyn ProfileCacheOperationRepo + Send + Sync>,
}

impl Debug for dyn ProfileDisplayRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProfileDisplayRepo")
    }
}

impl Debug for dyn ProfileOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProfileOperationRepo")
    }
}

impl Debug for dyn ProfileCacheOperationRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProfileCacheOperationRepo")
    }
}

#[async_trait]
impl ProfileDisplayRepo for ProfileDBUseCase {
    async fn find_sections(&self) -> Option<ProfileSections> {
        self.profile_display_repo.find_sections().await
    }
    async fn find_section(&self, id: i64) -> Option<ProfileSection> {
        self.profile_display_repo.find_section(id).await
    }
    async fn find_meta(&self) -> Option<ProfileMeta> {
        self.profile_display_repo.find_meta().await
    }
}

#[async_trait]
impl ProfileOperationRepo for ProfileDBUseCase {
    async fn get_new_id(&self) -> Option<i64> {
        self.profile_operation_repo.get_new_id().await
    }
    async fn add(&mut self, section: ProfileSection) -> Option<ProfileCommandStatus> {
        self.profile_operation_repo.add(section).await
    }
    async fn update(&mut self, section: ProfileSection) -> Option<ProfileCommandStatus> {
        self.profile_operation_repo.update(section).await
    }
    async fn delete(&mut self, id: i64) -> Option<ProfileCommandStatus> {
        self.profile_operation_repo.delete(id).await
    }
    async fn move_section(
        &mut self,
        id: i64,
        direction: MoveDirection,
    ) -> Option<ProfileCommandStatus> {
        self.profile_operation_repo
            .move_section(id, direction)
            .await
    }
    async fn update_meta(&mut self, meta: ProfileMeta) -> Option<ProfileCommandStatus> {
        self.profile_operation_repo.update_meta(meta).await
    }
}

impl ProfileDBUseCase {
    pub fn new(
        profile_display_repo: Box<dyn ProfileDisplayRepo + Send + Sync>,
        profile_operation_repo: Box<dyn ProfileOperationRepo + Send + Sync>,
    ) -> Self {
        Self {
            profile_display_repo,
            profile_operation_repo,
        }
    }
}

#[async_trait]
impl ProfileDisplayRepo for ProfileCacheUseCase {
    async fn find_sections(&self) -> Option<ProfileSections> {
        self.profile_display_repo.find_sections().await
    }
    async fn find_section(&self, id: i64) -> Option<ProfileSection> {
        self.profile_display_repo.find_section(id).await
    }
    async fn find_meta(&self) -> Option<ProfileMeta> {
        self.profile_display_repo.find_meta().await
    }
}

#[async_trait]
impl ProfileCacheOperationRepo for ProfileCacheUseCase {
    async fn insert_page(&mut self, page: ProfilePage) -> Option<ProfileCommandStatus> {
        self.profile_operation_repo.insert_page(page).await
    }
    async fn invalidate_page(&mut self) -> Option<ProfileCommandStatus> {
        self.profile_operation_repo.invalidate_page().await
    }
}

impl ProfileCacheUseCase {
    pub fn new(
        profile_display_repo: Box<dyn ProfileDisplayRepo + Send + Sync>,
        profile_operation_repo: Box<dyn ProfileCacheOperationRepo + Send + Sync>,
    ) -> Self {
        Self {
            profile_display_repo,
            profile_operation_repo,
        }
    }
}
