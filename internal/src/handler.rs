use crate::config::Config;
use crate::model::data::{AppState, BlogsData};
use crate::router::*;
use axum::{
    extract::State,
    routing::{get, get_service},
    Router,
};
use log::info;
use tower_http::services::{ServeDir, ServeFile};

pub async fn handler(cfg: Config) -> () {
    // Initialize Tracing and Logger
    // tracing_subscriber::fmt::init();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(cfg.log_level.clone()));

    let endpoint = cfg.svc_endpoint.as_str();
    let port = cfg.svc_port.as_str();
    let running_endpoint = format!("{}:{}", endpoint, port);

    // Setup config and blogs_data states
    let config = cfg.clone();
    let mut blogs_data = BlogsData::default();
    if !config.gh_owner.is_empty() && !config.gh_repo.is_empty() && !config.gh_branch.is_empty() {
        blogs_data = BlogsData::with_gh(&config.gh_owner, &config.gh_repo, &config.gh_branch).await;
    }
    let app_state = AppState { config, blogs_data };

    info!("Starting HTTP Server at http://{}", running_endpoint);

    // Axum Application
    let app = Router::new()
        .route("/", get(get_profile))
        .route("/not-found", get(get_404_not_found))
        .route("/version", get(get_version))
        .route("/blogs", get(get_blogs))
        .route("/blogs/:blog_id", get(get_blog))
        .nest_service("/statics", get_service(ServeDir::new("./statics/favicon/")))
        .nest_service(
            "/statics/styles.css",
            get_service(ServeFile::new("./statics/styles.css")),
        )
        .with_state(app_state)
        .fallback(get(get_404_not_found));

    // Start Axum Application
    let listener = tokio::net::TcpListener::bind(running_endpoint)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
