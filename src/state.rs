use crate::cache::inmemory::InMemoryCache;
use crate::config::Config;
use crate::database::turso::TursoDatabase;
use crate::model::axum::AppState;
use crate::model::blogs::BlogsParams;
use crate::model::tags::TagsListParams;
use crate::model::talks::TalksParams;
use crate::repo::blog_tag_mappings::BlogTagMappingDisplayRepo;
use crate::repo::blogs::BlogDisplayRepo;
use crate::repo::tags::TagDisplayRepo;
use crate::repo::talks::TalkDisplayRepo;
use crate::usecase::auth::AuthDBUseCase;
use crate::usecase::blog_tag_mappings::{BlogTagMappingCacheUseCase, BlogTagMappingDBUseCase};
use crate::usecase::blogs::{BlogCacheUseCase, BlogDBUseCase};
use crate::usecase::tags::{TagCacheUseCase, TagDBUseCase};
use crate::usecase::talks::{TalkCacheUseCase, TalkDBUseCase};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Debug)]
struct DBUsecases {
    blog: BlogDBUseCase,
    talk: TalkDBUseCase,
    tag: TagDBUseCase,
    btm: BlogTagMappingDBUseCase,
}
#[derive(Debug)]
struct CacheUsecases {
    blog: BlogCacheUseCase,
    talk: TalkCacheUseCase,
    tag: TagCacheUseCase,
    btm: BlogTagMappingCacheUseCase,
}

/// Pre-fill InMemory cache before application starting
async fn prefill_inmemory_cache(db_usecases: DBUsecases, mut cache_usecases: CacheUsecases) {
    let talks_opt = db_usecases
        .talk
        .find_talks(TalksParams {
            start: None,
            end: None,
        })
        .await;
    match talks_opt {
        Some(talks) => {
            info!("Inserting talks cache");
            for talk in talks.talks {
                let insert_opt = cache_usecases
                    .talk
                    .talk_operation_repo
                    .insert(talk.clone())
                    .await;
                if insert_opt.is_none() {
                    warn!("Failed to insert talk with id {} into cache", talk.id);
                }
            }
        }
        None => {
            warn!("Talks from database are empty");
        }
    }

    let tags_opt = db_usecases
        .tag
        .find_tags(TagsListParams {
            start: None,
            end: None,
        })
        .await;
    match tags_opt {
        Some(tags) => {
            info!("Inserting tags cache");
            for val in tags.tags {
                let insert_opt = cache_usecases
                    .tag
                    .tag_operation_repo
                    .insert(val.clone())
                    .await;
                if insert_opt.is_none() {
                    warn!("Failed to insert tag with id {} into cache", val.id);
                }
            }
        }
        None => {
            warn!("Tags from database are empty");
        }
    }

    let blogs_opt = db_usecases
        .blog
        .find_blogs(BlogsParams {
            start: None,
            end: None,
            tags: None,
        })
        .await;
    match blogs_opt.clone() {
        Some(blogs) => {
            info!("Inserting blogs cache");
            for val in blogs {
                let insert_opt = cache_usecases
                    .blog
                    .blog_operation_repo
                    .insert(val.clone())
                    .await;
                if insert_opt.is_none() {
                    warn!("Failed to insert blog with id {} into cache", val.id);
                }
            }
        }
        None => {
            warn!("Blogs from database are empty");
        }
    }

    let blog_ids: Vec<i64> = blogs_opt.unwrap().iter().map(|b| b.id).collect();
    for id in blog_ids {
        let btms_opt = db_usecases.btm.find_by_blog_id(id).await;

        match btms_opt {
            Some(btms) => {
                info!("Inserting blog tag mapping for blog id {} into cache", id);
                for val in btms.maps {
                    let insert_opt = cache_usecases
                        .btm
                        .operation
                        .insert(val.blog_id, val.tag_id)
                        .await;
                    if insert_opt.is_none() {
                        warn!("Failed to insert blog tag mapping with blog_id {} and tag_id {} into cache", val.blog_id, val.tag_id);
                    }
                }
            }
            None => {
                warn!("Blog Tag Mapping for blog id {} in database are empty", id);
            }
        }
    }
}

/// Create In-Memory cache usecases
async fn create_inmemory_cache_usecases(
    config: Config,
) -> (
    Option<BlogCacheUseCase>,
    Option<TalkCacheUseCase>,
    Option<TagCacheUseCase>,
    Option<BlogTagMappingCacheUseCase>,
) {
    info!("Building In Memory usecases.");
    let cache_repo = InMemoryCache::new(config.cache_ttl.unwrap());
    (
        Some(BlogCacheUseCase::new(
            Box::new(cache_repo.clone()),
            Box::new(cache_repo.clone()),
        )),
        Some(TalkCacheUseCase::new(
            Box::new(cache_repo.clone()),
            Box::new(cache_repo.clone()),
        )),
        Some(TagCacheUseCase::new(
            Box::new(cache_repo.clone()),
            Box::new(cache_repo.clone()),
        )),
        Some(BlogTagMappingCacheUseCase::new(
            Box::new(cache_repo.clone()),
            Box::new(cache_repo),
        )),
    )
}

/// Create SQLite database usecases
async fn create_sqlite_db_usecases(
    config: Config,
) -> (
    Option<BlogDBUseCase>,
    Option<TalkDBUseCase>,
    Option<TagDBUseCase>,
    Option<BlogTagMappingDBUseCase>,
    Option<AuthDBUseCase>,
) {
    info!("Building SQLite usecases.");
    let db_repo = TursoDatabase::new(
        config.data_source.clone(),
        config.secrets.database_url.clone(),
        None,
    )
    .await;
    (
        Some(BlogDBUseCase::new(
            Box::new(db_repo.clone()),
            Box::new(db_repo.clone()),
        )),
        Some(TalkDBUseCase::new(
            Box::new(db_repo.clone()),
            Box::new(db_repo.clone()),
        )),
        Some(TagDBUseCase::new(
            Box::new(db_repo.clone()),
            Box::new(db_repo.clone()),
        )),
        Some(BlogTagMappingDBUseCase::new(
            Box::new(db_repo.clone()),
            Box::new(db_repo.clone()),
        )),
        Some(AuthDBUseCase::new(Box::new(db_repo))),
    )
}

/// Create Turso database usecases
async fn create_turso_db_usecases(
    config: Config,
) -> (
    Option<BlogDBUseCase>,
    Option<TalkDBUseCase>,
    Option<TagDBUseCase>,
    Option<BlogTagMappingDBUseCase>,
    Option<AuthDBUseCase>,
) {
    info!("Building Turso usecases.");
    let db_repo = TursoDatabase::new(
        config.data_source.clone(),
        config.secrets.database_url.clone(),
        config.secrets.turso_auth_token.clone(),
    )
    .await;
    (
        Some(BlogDBUseCase::new(
            Box::new(db_repo.clone()),
            Box::new(db_repo.clone()),
        )),
        Some(TalkDBUseCase::new(
            Box::new(db_repo.clone()),
            Box::new(db_repo.clone()),
        )),
        Some(TagDBUseCase::new(
            Box::new(db_repo.clone()),
            Box::new(db_repo.clone()),
        )),
        Some(BlogTagMappingDBUseCase::new(
            Box::new(db_repo.clone()),
            Box::new(db_repo.clone()),
        )),
        Some(AuthDBUseCase::new(Box::new(db_repo))),
    )
}

/// Build App State for Axum Application
/// take a Config and return AppState
/// AppState contains `Config` and `BlogDBDBUseCase`
/// and can contain optional:
///
/// - TalkDBUseCase
/// - TagDBUseCase
/// - BlogTagMappingDBUseCase
/// - AuthDBUseCase
/// - TalkCacheUseCase
/// - TagCacheUseCase
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
    let data_source_is_configured_sqlite = config.data_source == "sqlite";
    let data_source_is_configured_turso =
        config.data_source == "turso" && config.secrets.turso_auth_token.is_some();
    let cache_is_enabled = config.cache_type.is_some();

    let (blog_db_uc, talk_db_uc, tag_db_uc, btm_db_uc, auth_db_uc) =
        if data_source_is_configured_turso {
            create_turso_db_usecases(config.clone()).await
        } else if data_source_is_configured_sqlite {
            create_sqlite_db_usecases(config.clone()).await
        } else {
            (None, None, None, None, None)
        };

    if blog_db_uc.is_none() {
        panic!("In version 0.3.5+, we drop the memory database support. Please use SQLite or Turso Database.");
    }

    let (blog_cache_uc, talk_cache_uc, tag_cache_uc, btm_cache_uc) = if cache_is_enabled {
        create_inmemory_cache_usecases(config.clone()).await
    } else {
        (None, None, None, None)
    };

    if cache_is_enabled {
        prefill_inmemory_cache(
            DBUsecases {
                blog: blog_db_uc.clone().unwrap(),
                talk: talk_db_uc.clone().unwrap(),
                tag: tag_db_uc.clone().unwrap(),
                btm: btm_db_uc.clone().unwrap(),
            },
            CacheUsecases {
                blog: blog_cache_uc.clone().unwrap(),
                talk: talk_cache_uc.clone().unwrap(),
                tag: tag_cache_uc.clone().unwrap(),
                btm: btm_cache_uc.clone().unwrap(),
            },
        )
        .await;
    }

    let blog_db_usecase = Arc::new(Mutex::new(blog_db_uc.unwrap()));
    let talk_db_usecase = Arc::new(Mutex::new(talk_db_uc));
    let tag_db_usecase = Arc::new(Mutex::new(tag_db_uc));
    let blog_tag_mapping_db_usecase = Arc::new(Mutex::new(btm_db_uc));
    let auth_db_usecase = Arc::new(Mutex::new(auth_db_uc));
    let talk_cache_usecase = Arc::new(Mutex::new(talk_cache_uc));
    let tag_cache_usecase = Arc::new(Mutex::new(tag_cache_uc));
    let blog_cache_usecase = Arc::new(Mutex::new(blog_cache_uc));
    let blog_tag_mapping_cache_usecase = Arc::new(Mutex::new(btm_cache_uc));

    AppState {
        config,
        blog_db_usecase,
        talk_db_usecase,
        tag_db_usecase,
        blog_tag_mapping_db_usecase,
        auth_db_usecase,
        talk_cache_usecase,
        tag_cache_usecase,
        blog_cache_usecase,
        blog_tag_mapping_cache_usecase,
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
