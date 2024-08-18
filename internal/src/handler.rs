use crate::config::Config;
use crate::model::data::BlogsData;
use crate::router::*;
use axum::{
    routing::{get, get_service},
    Router,
};
use log::info;
use tower_http::services::{ServeDir, ServeFile};

pub async fn handler(cfg: Config) -> () {
    // initialize tracing and logger
    // tracing_subscriber::fmt::init();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(cfg.log_level.clone()));

    let endpoint = cfg.svc_endpoint.as_str();
    let port = cfg.svc_port.as_str();
    let running_endpoint = format!("{}:{}", endpoint, port);

    let config = cfg.clone();

    // Build BlogsData
    let mut blogs_data = BlogsData::default();
    if !config.gh_owner.is_empty() && !config.gh_repo.is_empty() && !config.gh_branch.is_empty() {
        blogs_data = BlogsData::with_gh(&config.gh_owner, &config.gh_repo, &config.gh_branch).await;
    }

    info!("Starting HTTP Server at http://{}", running_endpoint);

    // Axum Application
    let app = Router::new()
        .route("/", get(get_profile))
        .route("/not-found", get(get_404_not_found))
        .nest_service("/statics", get_service(ServeDir::new("./statics/favicon/")))
        .nest_service(
            "/statics/styles.css",
            get_service(ServeFile::new("./statics/styles.css")),
        )
        .fallback(get(get_404_not_found));
    // .route("/blogs", get(get_blogs))
    // .route("/blogs/:blog_id", get(get_blog))
    // .route("/version", get(get_version))

    // Start Axum Application
    let listener = tokio::net::TcpListener::bind(running_endpoint)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
