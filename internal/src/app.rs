use crate::api::filesystem::FilesystemApiUseCase;
use crate::api::github::GithubApiUseCase;
use crate::database::memory::MemoryBlogRepo;
use crate::database::sqlite::SqliteBlogRepo;
use crate::handler;
use crate::model::axum::AppState;
use crate::port::blog::command::BlogCommandPort;
use crate::port::blog::query::BlogQueryPort;
use crate::repo::api::ApiRepo;
use crate::{config::Config, usecase::blog::BlogUseCase};
use axum::{
    routing::{get, get_service},
    Router,
};
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};

/// Run the axum web application
pub async fn app() -> () {
    // Setup Config
    let config = Config::from_envar();
    let endpoint = format!("{}:{}", &config.svc_endpoint, &config.svc_port);

    // Initialize Logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(config.log_level.clone()));

    // Init app state
    let app_state = state_factory(config).await;

    info!("Starting HTTP Server at http://{}", endpoint);

    // Axum Application
    let app = Router::new()
        .route("/", get(handler::profile::get_profile))
        .route("/version", get(handler::version::get_version))
        .route("/blogs", get(handler::blog::get_blogs))
        .route("/blogs/:blog_id", get(handler::blog::get_blog))
        .nest_service("/statics", get_service(ServeDir::new("./statics/favicon/")))
        .nest_service(
            "/statics/styles.css",
            get_service(ServeFile::new("./statics/styles.css")),
        )
        .with_state(app_state)
        .fallback(get(handler::status::get_404_not_found));

    // Start Axum Application
    let listener = tokio::net::TcpListener::bind(endpoint).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Build App State for Axum Application
async fn state_factory(config: Config) -> AppState {
    // Setup blog use case
    let data_source_is_configured_sqlite =
        config.data_source == "sqlite" && config.database_url != "";
    let github_api_is_enabled =
        !config.gh_owner.is_empty() && !config.gh_repo.is_empty() && !config.gh_branch.is_empty();

    let mut blog_uc = if data_source_is_configured_sqlite {
        // Use SqliteBlogRepo
        let repo = SqliteBlogRepo::new(config.database_url.clone()).await;
        BlogUseCase::new(Box::new(repo))
    } else {
        // // Use MemoryBlogRepo
        let repo = MemoryBlogRepo::default();
        BlogUseCase::new(Box::new(repo))
        // }
    };

    let fs_usecase = FilesystemApiUseCase::new("./statics/blogs/".to_string()).await;
    let blogs_metadata = fs_usecase.list_metadata().await;
    for metadata in blogs_metadata {
        // Check if blog id is in the database
        let blog_is_not_stored = !blog_uc.check_id(metadata.id.clone()).await.0;
        if blog_is_not_stored {
            info!("Start to fetch Blog {}.", &metadata.id);
            let blog = fs_usecase.fetch(metadata.clone()).await;
            info!("Finished to fetch Blog {}.", &metadata.id);

            info!("Start to store Blog {}.", &metadata.id);
            let _ = blog_uc
                .add(blog.id, blog.name, blog.filename, blog.source, blog.body)
                .await;
            info!("Finished to store Blog {}.", &metadata.id);
        }
    }

    if github_api_is_enabled {
        let github_usecase = GithubApiUseCase::new(
            config.gh_owner.clone(),
            config.gh_repo.clone(),
            config.gh_branch.clone(),
        )
        .await;
        let blogs_metadata = github_usecase.list_metadata().await;
        for metadata in blogs_metadata {
            // Check if blog id is in the database
            let blog_is_not_stored = !blog_uc.check_id(metadata.id.clone()).await.0;
            if blog_is_not_stored {
                info!("Start to fetch Blog {}.", &metadata.id);
                let blog = github_usecase.fetch(metadata.clone()).await;
                info!("Finished to fetch Blog {}.", &metadata.id);

                info!("Start to store Blog {}.", &metadata.id);
                let _ = blog_uc
                    .add(blog.id, blog.name, blog.filename, blog.source, blog.body)
                    .await;
                info!("Finished to store Blog {}.", &metadata.id);
            }
        }
    }

    let blog_usecase = Arc::new(Mutex::new(blog_uc));

    AppState {
        config,
        blog_usecase,
    }
}
