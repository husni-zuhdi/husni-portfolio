use crate::database::memory::MemoryBlogRepo;
use crate::handler;
use crate::model::axum::AppState;
use crate::{config::Config, usecase::blog::BlogUseCase};
use axum::{
    routing::{get, get_service},
    Router,
};
use log::info;
use std::sync::{Arc, Mutex};
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
    // Setup config and blogs_data states
    let mut blog_repo = MemoryBlogRepo::new();
    if !config.gh_owner.is_empty() && !config.gh_repo.is_empty() && !config.gh_branch.is_empty() {
        blog_repo =
            MemoryBlogRepo::from_github(&config.gh_owner, &config.gh_repo, &config.gh_branch).await;
    }
    let blog_usecase = Arc::new(Mutex::new(BlogUseCase::new(Box::new(blog_repo))));
    let app_state = AppState {
        config,
        blog_usecase,
    };
    app_state
}
