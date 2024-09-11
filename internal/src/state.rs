use crate::api::filesystem::FilesystemApiUseCase;
use crate::api::github::GithubApiUseCase;
use crate::database::memory::MemoryBlogRepo;
use crate::database::sqlite::SqliteBlogRepo;
use crate::model::axum::AppState;
use crate::port::blog::command::BlogCommandPort;
use crate::port::blog::query::BlogQueryPort;
use crate::repo::api::ApiRepo;
use crate::{config::Config, usecase::blog::BlogUseCase};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Build App State for Axum Application
pub async fn state_factory(config: Config) -> AppState {
    // Setup blog use case
    let data_source_is_configured_sqlite =
        config.data_source == "sqlite" && config.database_url != "";
    let github_api_is_enabled =
        !config.gh_owner.is_empty() && !config.gh_repo.is_empty() && !config.gh_branch.is_empty();

    let mut blog_uc = if data_source_is_configured_sqlite {
        let repo = SqliteBlogRepo::new(config.database_url.clone()).await;
        BlogUseCase::new(Box::new(repo))
    } else {
        let repo = MemoryBlogRepo::default();
        BlogUseCase::new(Box::new(repo))
    };

    let fs_usecase = FilesystemApiUseCase::new("./statics/blogs/".to_string()).await;
    let _ = populate_blog(Box::new(fs_usecase), &mut blog_uc).await;

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

    AppState {
        config,
        blog_usecase,
    }
}

async fn populate_blog(api_uc: Box<dyn ApiRepo + Send + Sync>, blog_uc: &mut BlogUseCase) {
    let blogs_metadata = api_uc.list_metadata().await;
    for metadata in blogs_metadata {
        let blog_is_not_stored = !blog_uc.check_id(metadata.id.clone()).await.0;
        if blog_is_not_stored {
            info!("Start to populate Blog {}.", &metadata.id);
            debug!("Start to fetch Blog {}.", &metadata.id);
            let blog = api_uc.fetch(metadata.clone()).await;
            debug!("Finished to fetch Blog {}.", &metadata.id);

            debug!("Start to store Blog {}.", &metadata.id);
            let _ = blog_uc
                .add(blog.id, blog.name, blog.filename, blog.source, blog.body)
                .await;
            debug!("Finished to store Blog {}.", &metadata.id);
        }
    }
}
