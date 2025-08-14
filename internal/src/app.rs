use crate::config::Config;
use crate::routes::main_route;
use crate::state::state_factory;
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
    let app = main_route(app_state);

    // Start Axum Application
    let listener = tokio::net::TcpListener::bind(endpoint).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
