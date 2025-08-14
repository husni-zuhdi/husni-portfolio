use crate::config::Config;
use crate::usecase::blog_tag_mappings::BlogTagMappingUseCase;
use crate::usecase::blogs::BlogUseCase;
use crate::usecase::tags::TagUseCase;
use crate::usecase::talks::TalkUseCase;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Axum state
/// Consist of Config and UseCases
#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub blog_usecase: Arc<Mutex<BlogUseCase>>,
    pub talk_usecase: Arc<Mutex<Option<TalkUseCase>>>,
    pub tag_usecase: Arc<Mutex<Option<TagUseCase>>>,
    pub blog_tag_mapping_usecase: Arc<Mutex<Option<BlogTagMappingUseCase>>>,
}
