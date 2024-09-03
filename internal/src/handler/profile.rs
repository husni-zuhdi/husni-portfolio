use crate::handler::error::get_500_internal_server_error;
use crate::model::templates::ProfileTemplate;
use askama::Template;
use axum::response::Html;
use log::{error, info};

/// get_profile
/// Serve Profile/Biography HTML file
pub async fn get_profile() -> Html<String> {
    let profile = ProfileTemplate.render();
    match profile {
        Ok(res) => {
            info!("Profile askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render profile.html. {}", err);
            get_500_internal_server_error()
        }
    }
}
