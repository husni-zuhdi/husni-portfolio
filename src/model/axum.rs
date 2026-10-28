use crate::config::Config;
use crate::usecase::auth::AuthDBUseCase;
use crate::usecase::blogs::BlogUseCase;
use crate::usecase::profile::ProfileUseCase;
use crate::usecase::tags::TagUseCase;
use crate::usecase::talks::TalkUseCase;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Axum state
/// Consists of Config and the per-entity deep usecases used in handlers.
///
/// Blog, Tag, Talk, and Profile usecases own their database repos and the
/// optional in-process cache behind their verbs. Blog tag mappings live
/// inside the Blog usecase and are not exposed.
///
/// `AuthDBUseCase` is wrapped with Option since the auth feature can be
/// deactivated based on Database used.
#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub blog_usecase: Arc<Mutex<BlogUseCase>>,
    pub tag_usecase: Arc<Mutex<TagUseCase>>,
    pub talk_usecase: Arc<Mutex<TalkUseCase>>,
    pub profile_usecase: Arc<Mutex<ProfileUseCase>>,
    pub auth_db_usecase: Arc<Mutex<Option<AuthDBUseCase>>>,
}
