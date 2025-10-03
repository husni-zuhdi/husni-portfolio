use crate::model::blogs::{Blog, BlogMetadata};
use crate::repo::api::ApiRepo;
use async_trait::async_trait;
use core::fmt::Debug;

#[derive(Clone, Debug)]
pub struct ApiUseCase {
    pub api_repo: Box<dyn ApiRepo + Send + Sync>,
}

impl Debug for dyn ApiRepo + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiRepo",)
    }
}

#[async_trait]
impl ApiRepo for ApiUseCase {
    async fn list_metadata(&self) -> Option<Vec<BlogMetadata>> {
        self.api_repo.list_metadata().await
    }
    async fn fetch(&self, metadata: BlogMetadata) -> Option<Blog> {
        self.api_repo.fetch(metadata).await
    }
}

impl ApiUseCase {
    pub fn new(api_repo: Box<dyn ApiRepo + Send + Sync>) -> ApiUseCase {
        ApiUseCase { api_repo }
    }
}
