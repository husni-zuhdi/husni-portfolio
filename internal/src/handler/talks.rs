use crate::handler::status::get_500_internal_server_error;
use crate::model::templates::TalksTemplate;
use askama::Template;
use axum::response::Html;
use tracing::{error, info};

/// get_talks
/// Serve Talks HTML file
pub async fn get_talks() -> Html<String> {
    let talks = TalksTemplate.render();
    match talks {
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
