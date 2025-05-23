use crate::handler::status::get_500_internal_server_error;
use crate::model::version::Version;
use crate::model::{axum::AppState, templates::VersionTemplate};
use askama::Template;
use axum::extract::State;
use axum::response::Html;
use tracing::{error, info};

/// get_version
/// Serve get_version HTML file
pub async fn get_version(State(app_state): State<AppState>) -> Html<String> {
    let version_data = Version::new().expect("Failed to generate Version struct");
    let version = VersionTemplate {
        version: version_data.version,
        environment: app_state.config.environment,
        build_hash: version_data.build_hash,
        build_date: version_data.build_date,
    }
    .render();

    match version {
        Ok(res) => {
            info!("Version askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render version.html. {}", err);
            get_500_internal_server_error()
        }
    }
}
