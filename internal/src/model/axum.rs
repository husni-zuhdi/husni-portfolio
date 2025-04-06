use crate::config::Config;
use crate::usecase::blogs::BlogUseCase;
use crate::usecase::talks::TalkUseCase;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Axum state
/// Consist of Config and BlogUseCase
#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub blog_usecase: Arc<Mutex<BlogUseCase>>,
    pub talk_usecase: Arc<Mutex<Option<TalkUseCase>>>,
}
