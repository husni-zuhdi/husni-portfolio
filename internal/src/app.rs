use crate::config::Config;
use crate::handler::{profile, status, version};
use crate::routes;
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
pub async fn app() {
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
        .route("/", get(profile::get_profile))
        .route("/version", get(version::get_version))
        .route("/etc/passwd", get(status::get_418_i_am_a_teapot))
        .nest("/blogs", routes::blogs_route())
        .nest("/talks", routes::talks_route())
        .nest("/admin/talks", routes::admin_talks_route())
        .nest_service("/statics", get_service(ServeDir::new("./statics/favicon/")))
        .nest_service(
            "/statics/styles.css",
            get_service(ServeFile::new("./statics/styles.css")),
        )
        .with_state(app_state)
        .layer(ServiceBuilder::new().layer(CompressionLayer::new()))
        .fallback(get(status::get_404_not_found));

    // Start Axum Application
    let listener = tokio::net::TcpListener::bind(endpoint).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
