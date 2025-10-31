use crate::config::Config;
use crate::database::memory::MemoryBlogRepo;
use crate::database::turso::TursoDatabase;
use crate::model::axum::AppState;
use crate::usecase::auth::AuthUseCase;
use crate::usecase::blog_tag_mappings::BlogTagMappingUseCase;
use crate::usecase::blogs::BlogUseCase;
use crate::usecase::tags::TagUseCase;
use crate::usecase::talks::TalkUseCase;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// Build App State for Axum Application
/// take a Config and return AppState
/// AppState contains `Config` and `BlogUseCase`
/// and can contain optional:
///
/// - TalkUseCase
/// - TagUseCase
/// - BlogTagMappingUseCase
/// - AuthUseCase
///
/// To have a fully function portfolio web-app, it's sugessted to enable
/// all usecases.
///
/// Deprecated features:
///
/// - Filesystem blogs detection
/// - Populate blogs from github repository
pub async fn state_factory(config: Config) -> AppState {
    // Setup blog use case
    let data_source_is_configured_sqlite =
        config.data_source == "sqlite" && config.secrets.database_url.is_some();
    let data_source_is_configured_turso = config.data_source == "turso"
        && config.secrets.database_url.is_some()
        && config.secrets.turso_auth_token.is_some();

    let (blog_uc, talk_uc, tag_uc, blog_tag_mapping_uc, auth_uc) =
        if data_source_is_configured_sqlite {
            info!("Building SQLite usecases.");
            let repo = TursoDatabase::new(
                config.data_source.clone(),
                config.secrets.database_url.clone().unwrap(),
                None,
            )
            .await;
            (
                BlogUseCase::new(Box::new(repo.clone())),
                Some(TalkUseCase::new(Box::new(repo.clone()))),
                Some(TagUseCase::new(Box::new(repo.clone()))),
                Some(BlogTagMappingUseCase::new(Box::new(repo.clone()))),
                Some(AuthUseCase::new(Box::new(repo))),
            )
        } else if data_source_is_configured_turso {
            info!("Building Turso usecases.");
            let repo = TursoDatabase::new(
                config.data_source.clone(),
                config.secrets.database_url.clone().unwrap(),
                config.secrets.turso_auth_token.clone(),
            )
            .await;
            (
                BlogUseCase::new(Box::new(repo.clone())),
                Some(TalkUseCase::new(Box::new(repo.clone()))),
                Some(TagUseCase::new(Box::new(repo.clone()))),
                Some(BlogTagMappingUseCase::new(Box::new(repo.clone()))),
                Some(AuthUseCase::new(Box::new(repo))),
            )
        } else {
            let repo = MemoryBlogRepo::default();
            (BlogUseCase::new(Box::new(repo)), None, None, None, None)
        };

    let blog_usecase = Arc::new(Mutex::new(blog_uc));
    let talk_usecase = Arc::new(Mutex::new(talk_uc));
    let tag_usecase = Arc::new(Mutex::new(tag_uc));
    let blog_tag_mapping_usecase = Arc::new(Mutex::new(blog_tag_mapping_uc));
    let auth_usecase = Arc::new(Mutex::new(auth_uc));

    AppState {
        config,
        blog_usecase,
        talk_usecase,
        tag_usecase,
        blog_tag_mapping_usecase,
        auth_usecase,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_memory_state_factory() {
        // Config default data source is `memory`
        let config = Config::default();
        let state = state_factory(config).await;
        // Assume `BlogUseCase` always created.
        let talk_uc = state.talk_usecase.lock().await.take();
        let tag_uc = state.tag_usecase.lock().await.take();
        let blogtag_uc = state.blog_tag_mapping_usecase.lock().await.take();
        let auth_uc = state.auth_usecase.lock().await.take();

        assert!(talk_uc.is_none(), "TalkUseCase is Some()");
        assert!(tag_uc.is_none(), "TagUseCase is Some()");
        assert!(blogtag_uc.is_none(), "BlogTagMappingUseCase is Some()");
        assert!(auth_uc.is_none(), "AuthUseCase is Some()");
    }

    #[tokio::test]
    async fn test_sqlite_state() {
        // Config default data source is `memory`
        let mut config = Config::default();
        config.data_source = "sqlite".to_string();
        config.secrets.database_url = Some("../husni-portfolio.db".to_string());

        let state = state_factory(config).await;
        // Assume `BlogUseCase` always created.
        let talk_uc = state.talk_usecase.lock().await.take();
        let tag_uc = state.tag_usecase.lock().await.take();
        let blogtag_uc = state.blog_tag_mapping_usecase.lock().await.take();
        let auth_uc = state.auth_usecase.lock().await.take();

        assert!(talk_uc.is_some(), "TalkUseCase is None");
        assert!(tag_uc.is_some(), "TagUseCase is None");
        assert!(blogtag_uc.is_some(), "BlogTagMappingUseCase is None");
        assert!(auth_uc.is_some(), "AuthUseCase is None");
    }

    //#[tokio::test]
    //async fn test_turso_state() {
    //    todo!()
    //}
}
