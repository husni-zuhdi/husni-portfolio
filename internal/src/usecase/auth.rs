use crate::model::auth::{Session, SessionCommandStatus, User, UserCommandStatus};
use crate::repo::auth::AuthRepo;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct AuthUseCase {
    pub auth_repo: Box<dyn AuthRepo + Send + Sync>,
}

impl Debug for dyn AuthRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AuthRepo")
    }
}

#[async_trait]
impl AuthRepo for AuthUseCase {
    async fn find_user_by_id(&self, id: String) -> Option<User> {
        self.auth_repo.find_user_by_id(id).await
    }
    async fn find_user_by_email(&self, email: String) -> Option<User> {
        self.auth_repo.find_user_by_email(email).await
    }
    async fn add_user(
        &self,
        id: String,
        email: String,
        hpass: String,
    ) -> Option<UserCommandStatus> {
        self.auth_repo.add_user(id, email, hpass).await
    }
    async fn update_user(
        &self,
        id: String,
        email: Option<String>,
        hpass: Option<String>,
    ) -> Option<UserCommandStatus> {
        self.auth_repo.update_user(id, email, hpass).await
    }
    async fn delete_user(&self, id: String) -> Option<UserCommandStatus> {
        self.auth_repo.delete_user(id).await
    }
    async fn find_session(&self, id: String) -> Option<Session> {
        self.auth_repo.find_session(id).await
    }
    async fn add_session(
        &self,
        id: String,
        user_id: String,
        token: String,
        expire: String,
    ) -> Option<SessionCommandStatus> {
        self.auth_repo.add_session(id, user_id, token, expire).await
    }
    async fn delete_session(&self, id: String) -> Option<SessionCommandStatus> {
        self.auth_repo.delete_session(id).await
    }
}

impl AuthUseCase {
    pub fn new(auth_repo: Box<dyn AuthRepo + Send + Sync>) -> AuthUseCase {
        AuthUseCase { auth_repo }
    }
}
