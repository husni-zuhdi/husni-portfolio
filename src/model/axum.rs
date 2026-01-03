use crate::config::Config;
use crate::usecase::auth::AuthDBUseCase;
use crate::usecase::blog_tag_mappings::BlogTagMappingDBUseCase;
use crate::usecase::blogs::BlogDBUseCase;
use crate::usecase::tags::TagDBUseCase;
use crate::usecase::talks::{TalkCacheUseCase, TalkDBUseCase};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Axum state
/// Consist of Config and UseCases to be used in handlers
/// Other than Blog usecase, all usecases Arc Mutex are wrapped with Option
/// since they can be deactivated based on Database used.
#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub blog_db_usecase: Arc<Mutex<BlogDBUseCase>>,
    pub talk_db_usecase: Arc<Mutex<Option<TalkDBUseCase>>>,
    pub tag_db_usecase: Arc<Mutex<Option<TagDBUseCase>>>,
    pub blog_tag_mapping_db_usecase: Arc<Mutex<Option<BlogTagMappingDBUseCase>>>,
    pub auth_db_usecase: Arc<Mutex<Option<AuthDBUseCase>>>,
    pub talk_cache_usecase: Arc<Mutex<Option<TalkCacheUseCase>>>,
}
