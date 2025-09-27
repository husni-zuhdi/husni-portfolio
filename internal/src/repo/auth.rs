use crate::model::auth::{Session, SessionCommandStatus, User, UserCommandStatus};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

clone_trait_object!(AuthRepo);

#[async_trait]
pub trait AuthRepo: DynClone {
    async fn find_user_by_id(&self, id: String) -> Option<User>;
    async fn find_user_by_email(&self, email: String) -> Option<User>;
    async fn add_user(&self, id: String, email: String, hpass: String)
        -> Option<UserCommandStatus>;
    async fn update_user(
        &self,
        id: String,
        email: Option<String>,
        hpass: Option<String>,
    ) -> Option<UserCommandStatus>;
    async fn delete_user(&self, id: String) -> Option<UserCommandStatus>;
    async fn find_session(&self, id: String) -> Option<Session>;
    async fn add_session(
        &self,
        id: String,
        user_id: String,
        token: String,
        expire: String,
    ) -> Option<SessionCommandStatus>;
    async fn delete_session(&self, id: String) -> Option<SessionCommandStatus>;
}
