use crate::config::Config;
use crate::usecase::blog::BlogUseCase;

/// Axum state
/// Consist of Config and BlogUseCase
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub blog_usecase: BlogUseCase,
}
