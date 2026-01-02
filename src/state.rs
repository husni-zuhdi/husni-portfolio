use crate::cache::inmemory::InMemoryCache;
use crate::config::Config;
use crate::database::memory::MemoryBlogRepo;
use crate::database::turso::TursoDatabase;
use crate::model::axum::AppState;
use crate::usecase::auth::AuthDBUseCase;
use crate::usecase::blog_tag_mappings::BlogTagMappingDBUseCase;
use crate::usecase::blogs::BlogDBUseCase;
use crate::usecase::tags::TagDBUseCase;
use crate::usecase::talks::{TalkCacheUseCase, TalkDBUseCase};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// Build App State for Axum Application
/// take a Config and return AppState
/// AppState contains `Config` and `BlogDBDBUseCase`
/// and can contain optional:
///
/// - TalkDBUseCase
/// - TagDBUseCase
/// - BlogTagMappingDBUseCase
/// - AuthDBUseCase
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
    let cache_is_enabled = config.cache_type.is_some();

    let (blog_uc, talk_uc, tag_uc, blog_tag_mapping_uc, auth_uc) =
        if data_source_is_configured_sqlite {
            info!("Building SQLite usecases.");
            let db_repo = TursoDatabase::new(
                config.data_source.clone(),
                config.secrets.database_url.clone().unwrap(),
                None,
            )
            .await;
            (
                BlogDBUseCase::new(Box::new(db_repo.clone())),
                Some(TalkDBUseCase::new(
                    Box::new(db_repo.clone()),
                    Box::new(db_repo.clone()),
                )),
                Some(TagDBUseCase::new(Box::new(db_repo.clone()))),
                Some(BlogTagMappingDBUseCase::new(Box::new(db_repo.clone()))),
                Some(AuthDBUseCase::new(Box::new(db_repo))),
            )
        } else if data_source_is_configured_turso {
            info!("Building Turso usecases.");
            let db_repo = TursoDatabase::new(
                config.data_source.clone(),
                config.secrets.database_url.clone().unwrap(),
                config.secrets.turso_auth_token.clone(),
            )
            .await;
            (
                BlogDBUseCase::new(Box::new(db_repo.clone())),
                Some(TalkDBUseCase::new(
                    Box::new(db_repo.clone()),
                    Box::new(db_repo.clone()),
                )),
                Some(TagDBUseCase::new(Box::new(db_repo.clone()))),
                Some(BlogTagMappingDBUseCase::new(Box::new(db_repo.clone()))),
                Some(AuthDBUseCase::new(Box::new(db_repo))),
            )
        } else {
            let repo = MemoryBlogRepo::default();
            (BlogDBUseCase::new(Box::new(repo)), None, None, None, None)
        };

    let talk_cache_uc = if cache_is_enabled {
        let cache_repo = InMemoryCache::new(config.cache_ttl.unwrap()).await;
        Some(TalkCacheUseCase::new(
            Box::new(cache_repo.clone()),
            Box::new(cache_repo.clone()),
        ))
    } else {
        None
    };

    let blog_db_usecase = Arc::new(Mutex::new(blog_uc));
    let talk_db_usecase = Arc::new(Mutex::new(talk_uc));
    let tag_db_usecase = Arc::new(Mutex::new(tag_uc));
    let blog_tag_mapping_db_usecase = Arc::new(Mutex::new(blog_tag_mapping_uc));
    let auth_db_usecase = Arc::new(Mutex::new(auth_uc));
    let talk_cache_usecase = Arc::new(Mutex::new(talk_cache_uc));

    AppState {
        config,
        blog_db_usecase,
        talk_db_usecase,
        tag_db_usecase,
        blog_tag_mapping_db_usecase,
        auth_db_usecase,
        talk_cache_usecase,
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
        // Assume `BlogDBDBUseCase` always created.
        let talk_uc = state.talk_db_usecase.lock().await.take();
        let tag_uc = state.tag_db_usecase.lock().await.take();
        let blogtag_uc = state.blog_tag_mapping_db_usecase.lock().await.take();
        let auth_uc = state.auth_db_usecase.lock().await.take();
        let talk_cache_uc = state.talk_cache_usecase.lock().await.take();

        assert!(talk_uc.is_none(), "TalkDBUseCase is Some()");
        assert!(tag_uc.is_none(), "TagDBUseCase is Some()");
        assert!(blogtag_uc.is_none(), "BlogTagMappingDBUseCase is Some()");
        assert!(auth_uc.is_none(), "AuthDBUseCase is Some()");
        assert!(talk_cache_uc.is_none(), "TalkCacheUseCase is Some()");
    }

    #[tokio::test]
    async fn test_sqlite_state() {
        // Config default data source is `memory`
        let mut config = Config {
            data_source: "sqlite".to_string(),
            ..Default::default()
        };
        //let mut config = Config::default();
        //config.data_source = "sqlite".to_string();
        config.secrets.database_url = Some("../husni-portfolio.db".to_string());

        let state = state_factory(config).await;
        // Assume `BlogDBDBUseCase` always created.
        let talk_uc = state.talk_db_usecase.lock().await.take();
        let tag_uc = state.tag_db_usecase.lock().await.take();
        let blogtag_uc = state.blog_tag_mapping_db_usecase.lock().await.take();
        let auth_uc = state.auth_db_usecase.lock().await.take();

        assert!(talk_uc.is_some(), "TalkDBUseCase is None");
        assert!(tag_uc.is_some(), "TagDBUseCase is None");
        assert!(blogtag_uc.is_some(), "BlogTagMappingDBUseCase is None");
        assert!(auth_uc.is_some(), "AuthDBUseCase is None");
    }

    //#[tokio::test]
    //async fn test_turso_state() {
    //    todo!()
    //}

    //#[tokio::test]
    //async fn test_inmemory_cache_state() {
    //    todo!()
    //}
}
