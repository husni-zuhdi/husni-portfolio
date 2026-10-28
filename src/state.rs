use crate::cache::inmemory::InMemoryCache;
use crate::config::Config;
use crate::database::turso::TursoDatabase;
use crate::model::axum::AppState;
use crate::usecase::auth::AuthDBUseCase;
use crate::usecase::blog_tag_mappings::BlogTagMappingUseCase;
use crate::usecase::blogs::BlogUseCase;
use crate::usecase::profile::ProfileUseCase;
use crate::usecase::tags::TagUseCase;
use crate::usecase::talks::TalkUseCase;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// Create all per-entity usecases. Each usecase carries its own cache when
/// `cache_repo` is attached; the Blog usecase additionally owns the Blog
/// tag usecase for its write choreography.
#[allow(clippy::type_complexity)]
async fn create_db_usecases(
    config: &Config,
    cache_repo: Option<InMemoryCache>,
    turso_auth_token: Option<String>,
) -> (
    Option<BlogUseCase>,
    Option<TagUseCase>,
    Option<TalkUseCase>,
    Option<ProfileUseCase>,
    Option<AuthDBUseCase>,
) {
    info!("Building database usecases.");
    let db_repo = TursoDatabase::new(
        config.data_source.clone(),
        config.secrets.database_url.clone(),
        turso_auth_token,
    )
    .await;

    let mut blog_usecase = BlogUseCase::new(Box::new(db_repo.clone()), Box::new(db_repo.clone()));
    let mut tag_usecase = TagUseCase::new(Box::new(db_repo.clone()), Box::new(db_repo.clone()));
    let mut btm_usecase =
        BlogTagMappingUseCase::new(Box::new(db_repo.clone()), Box::new(db_repo.clone()));
    let mut talk_usecase = TalkUseCase::new(Box::new(db_repo.clone()), Box::new(db_repo.clone()));
    let mut profile_usecase =
        ProfileUseCase::new(Box::new(db_repo.clone()), Box::new(db_repo.clone()));
    if let Some(cache_repo) = cache_repo.clone() {
        btm_usecase = btm_usecase.with_cache(Box::new(cache_repo.clone()));
        blog_usecase = blog_usecase
            .with_cache(Box::new(cache_repo.clone()), Box::new(cache_repo.clone()))
            .with_tag_usecase(tag_usecase.clone())
            .with_btm_usecase(btm_usecase);
        tag_usecase =
            tag_usecase.with_cache(Box::new(cache_repo.clone()), Box::new(cache_repo.clone()));
        talk_usecase =
            talk_usecase.with_cache(Box::new(cache_repo.clone()), Box::new(cache_repo.clone()));
        profile_usecase =
            profile_usecase.with_cache(Box::new(cache_repo.clone()), Box::new(cache_repo));
    } else {
        // The Blog write choreography always needs the Tag and Blog tag
        // usecases; their caches are resolved at their own construction.
        blog_usecase = blog_usecase
            .with_tag_usecase(tag_usecase.clone())
            .with_btm_usecase(btm_usecase);
    }

    (
        Some(blog_usecase),
        Some(tag_usecase),
        Some(talk_usecase),
        Some(profile_usecase),
        Some(AuthDBUseCase::new(Box::new(db_repo))),
    )
}

/// Build App State for Axum Application
/// take a Config and return AppState
/// AppState contains `Config` and the per-entity usecases:
///
/// - BlogUseCase (owns the Blog tag mappings)
/// - TagUseCase
/// - TalkUseCase
/// - ProfileUseCase
/// - AuthDBUseCase (optional)
///
/// Deprecated features:
///
/// - Filesystem blogs detection
/// - Populate blogs from github repository
pub async fn state_factory(config: Config) -> AppState {
    // Setup blog use case
    let data_source_is_configured_sqlite = config.data_source == "sqlite";
    let data_source_is_configured_turso =
        config.data_source == "turso" && config.secrets.turso_auth_token.is_some();
    let cache_is_enabled = config.cache_type.is_some();

    let cache_repo = if cache_is_enabled {
        info!("Building In Memory cache.");
        Some(InMemoryCache::new(config.cache_ttl.unwrap()))
    } else {
        None
    };

    let turso_auth_token = if data_source_is_configured_sqlite {
        None
    } else {
        config.secrets.turso_auth_token.clone()
    };
    let (blog_uc, tag_uc, talk_uc, profile_uc, auth_db_uc) =
        if data_source_is_configured_turso || data_source_is_configured_sqlite {
            create_db_usecases(&config, cache_repo, turso_auth_token).await
        } else {
            (None, None, None, None, None)
        };

    if blog_uc.is_none() {
        panic!("In version 0.3.5+, we drop the memory database support. Please use SQLite or Turso Database.");
    }

    let mut blog_usecase = blog_uc.unwrap();
    let mut tag_usecase = tag_uc.unwrap();
    let mut talk_usecase = talk_uc.unwrap();
    let mut profile_usecase = profile_uc.unwrap();
    if cache_is_enabled {
        // Warm the per-entity caches from the database before serving.
        blog_usecase.prefill().await;
        tag_usecase.prefill().await;
        talk_usecase.prefill().await;
        profile_usecase.prefill().await;
    }

    let blog_usecase = Arc::new(Mutex::new(blog_usecase));
    let tag_usecase = Arc::new(Mutex::new(tag_usecase));
    let talk_usecase = Arc::new(Mutex::new(talk_usecase));
    let profile_usecase = Arc::new(Mutex::new(profile_usecase));
    let auth_db_usecase = Arc::new(Mutex::new(auth_db_uc));

    AppState {
        config,
        blog_usecase,
        tag_usecase,
        talk_usecase,
        profile_usecase,
        auth_db_usecase,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_state() {
        // Config default data source is `sqlite`
        let config = Config::default();

        let state = state_factory(config).await;
        // Assume `BlogUseCase` and the other usecases always created.
        let auth_uc = state.auth_db_usecase.lock().await.take();

        state
            .tag_usecase
            .lock()
            .await
            .get_new_id()
            .await
            .expect("TagUseCase must resolve a new id");
        state
            .talk_usecase
            .lock()
            .await
            .find_talks(crate::model::talks::TalksParams {
                start: Some(0),
                end: Some(100),
            })
            .await;
        assert!(auth_uc.is_some(), "AuthDBUseCase is None");
    }

    #[tokio::test]
    async fn test_sqlite_state_with_cache() {
        let mut config = Config::default();
        config.cache_type = Some(crate::config::Cache::InMemory);
        config.cache_ttl = Some(3600);

        let state = state_factory(config).await;
        let mut blog_uc = state.blog_usecase.lock().await.clone();
        let mut tag_uc = state.tag_usecase.lock().await.clone();
        let mut talk_uc = state.talk_usecase.lock().await.clone();
        let mut profile_uc = state.profile_usecase.lock().await.clone();

        // Per-entity prefill warms every cache from the seeded SQLite
        // database without failing.
        blog_uc.prefill().await;
        tag_uc.prefill().await;
        talk_uc.prefill().await;
        profile_uc.prefill().await;
    }
}
