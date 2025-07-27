use crate::config::Config;
use crate::handler;
use crate::state::state_factory;
use axum::{
    routing::{get, get_service},
    Router,
};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

/// Run the axum web application
pub async fn app() -> () {
    // Setup Config
    let config = Config::from_envar();
    let endpoint = format!("{}:{}", &config.svc_endpoint, &config.svc_port);

    // Initialize Tracing
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .init();

    // Init app state
    let app_state = state_factory(config).await;

    info!("Starting HTTP Server at http://{}", endpoint);

    // Axum Application
    let app = Router::new()
        .route("/", get(handler::profile::get_profile))
        .route("/version", get(handler::version::get_version))
        .route("/blogs", get(handler::blogs::get_blogs))
        .route("/blogs/:blog_id", get(handler::blogs::get_blog))
        .route("/talks", get(handler::talks::get_talks))
        .route("/etc/passwd", get(handler::status::get_418_i_am_a_teapot))
        .nest_service("/statics", get_service(ServeDir::new("./statics/favicon/")))
        .nest_service(
            "/statics/styles.css",
            get_service(ServeFile::new("./statics/styles.css")),
        )
        .with_state(app_state)
        .layer(ServiceBuilder::new().layer(CompressionLayer::new()))
        .fallback(get(handler::status::get_404_not_found));

    // Start Axum Application
    let listener = tokio::net::TcpListener::bind(endpoint).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
