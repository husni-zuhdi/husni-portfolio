use crate::api::github::GithubApiUseCase;
use crate::config::Config;
use crate::database::memory::MemoryBlogRepo;
use crate::database::turso::TursoDatabase;
use crate::model::axum::AppState;
use crate::repo::api::ApiRepo;
use crate::repo::blogs::BlogRepo;
use crate::usecase::auth::AuthUseCase;
use crate::usecase::blog_tag_mappings::BlogTagMappingUseCase;
use crate::usecase::blogs::BlogUseCase;
use crate::usecase::tags::TagUseCase;
use crate::{api::filesystem::FilesystemApiUseCase, usecase::talks::TalkUseCase};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Build App State for Axum Application
pub async fn state_factory(config: Config) -> AppState {
    // Setup blog use case
    let data_source_is_configured_sqlite =
        config.data_source == "sqlite" && !config.database_url.is_empty();
    let data_source_is_configured_turso = config.data_source == "turso"
        && !config.database_url.is_empty()
        && !config.turso_auth_token.is_empty();
    let github_api_is_enabled =
        !config.gh_owner.is_empty() && !config.gh_repo.is_empty() && !config.gh_branch.is_empty();

    let (mut blog_uc, talk_uc, tag_uc, blog_tag_mapping_uc, auth_uc) =
        if data_source_is_configured_sqlite {
            let repo = TursoDatabase::new(
                config.data_source.clone(),
                config.database_url.clone(),
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
            let repo = TursoDatabase::new(
                config.data_source.clone(),
                config.database_url.clone(),
                Some(config.turso_auth_token.clone()),
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

    if !config.filesystem_dir.is_empty() {
        let fs_usecase = FilesystemApiUseCase::new(config.filesystem_dir.clone()).await;
        let _ = populate_blog(Box::new(fs_usecase), &mut blog_uc).await;
    }

    if github_api_is_enabled {
        let github_usecase = GithubApiUseCase::new(
            config.gh_owner.clone(),
            config.gh_repo.clone(),
            config.gh_branch.clone(),
        )
        .await;
        let _ = populate_blog(Box::new(github_usecase), &mut blog_uc).await;
    }

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

async fn populate_blog(api_uc: Box<dyn ApiRepo + Send + Sync>, blog_uc: &mut BlogUseCase) {
    let blogs_metadata = api_uc.list_metadata().await.unwrap();
    for metadata in blogs_metadata {
        let blog_is_not_stored = blog_uc.check_id(metadata.id).await.is_none();
        if blog_is_not_stored {
            info!("Start to populate Blog {}.", &metadata.id);
            debug!("Start to fetch Blog {}.", &metadata.id);
            let blog = api_uc.fetch(metadata.clone()).await.unwrap();
            debug!("Finished to fetch Blog {}.", &metadata.id);

            debug!("Start to store Blog {}.", &metadata.id);
            let _ = blog_uc.add(blog).await;
            debug!("Finished to store Blog {}.", &metadata.id);
        }
    }
}
