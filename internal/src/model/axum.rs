use crate::config::Config;
use crate::usecase::blogs::BlogUseCase;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Axum state
/// Consist of Config and BlogUseCase
#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub blog_usecase: Arc<Mutex<BlogUseCase>>,
}
