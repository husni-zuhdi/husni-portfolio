use crate::model::blog::{Blog, BlogMetadata};
use crate::port::api::query::ApiQueryPort;
use crate::repo::api::ApiRepo;
use async_trait::async_trait;

#[derive(Clone)]
pub struct ApiUseCase {
    pub api_repo: Box<dyn ApiRepo + Send + Sync>,
}

#[async_trait]
impl ApiQueryPort for ApiUseCase {
    async fn list_metadata(&self) -> Vec<BlogMetadata> {
        self.api_repo.list_metadata().await
    }
    async fn fetch(&self, metadata: BlogMetadata) -> Blog {
        self.api_repo.fetch(metadata).await
    }
}

impl ApiUseCase {
    pub fn new(api_repo: Box<dyn ApiRepo + Send + Sync>) -> ApiUseCase {
        ApiUseCase { api_repo }
    }
}
