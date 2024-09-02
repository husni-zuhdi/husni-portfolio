use crate::database::memory::MemoryBlogRepo;
use crate::database::sqlite::SqliteBlogRepo;
use crate::handler;
use crate::model::axum::AppState;
use crate::{config::Config, usecase::blog::BlogUseCase};
use axum::{
    routing::{get, get_service},
    Router,
};
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};

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
        .route("/", get(handler::get_profile))
        .route("/not-found", get(handler::get_404_not_found))
        .route("/version", get(handler::get_version))
        .route("/blogs", get(handler::get_blogs))
        .route("/blogs/:blog_id", get(handler::get_blog))
        .nest_service("/statics", get_service(ServeDir::new("./statics/favicon/")))
        .nest_service(
            "/statics/styles.css",
            get_service(ServeFile::new("./statics/styles.css")),
        )
        .with_state(app_state)
        .fallback(get(handler::get_404_not_found));

    // Start Axum Application
    let listener = tokio::net::TcpListener::bind(endpoint).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn state_factory(config: Config) -> AppState {
    // Setup blog use case
    let blog_usecase = if config.data_source == "sqlite" && config.database_url != "" {
        // Use SqliteBlogRepo
        let repo = SqliteBlogRepo::new(config.database_url.clone()).await;
        Arc::new(Mutex::new(BlogUseCase::new(Box::new(repo))))
    } else {
        // Use MemoryBlogRepo
        if !config.gh_owner.is_empty() && !config.gh_repo.is_empty() && !config.gh_branch.is_empty()
        {
            // Use from_github method
            let repo =
                MemoryBlogRepo::from_github(&config.gh_owner, &config.gh_repo, &config.gh_branch)
                    .await;
            Arc::new(Mutex::new(BlogUseCase::new(Box::new(repo))))
        } else {
            // Use Default method
            let repo = MemoryBlogRepo::default();
            Arc::new(Mutex::new(BlogUseCase::new(Box::new(repo))))
        }
    };

    AppState {
        config,
        blog_usecase,
    }
}
