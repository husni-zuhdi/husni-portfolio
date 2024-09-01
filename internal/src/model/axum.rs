use crate::config::Config;
use crate::usecase::blog::BlogUseCase;
use std::sync::{Arc, Mutex};

/// Axum state
/// Consist of Config and BlogUseCase
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub blog_usecase: Arc<Mutex<BlogUseCase>>,
}
