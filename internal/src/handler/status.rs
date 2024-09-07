use crate::model::templates::{InternalServerErrorTemplate, NotFoundTemplate};
use askama::Template;
use axum::response::Html;
use tracing::{error, info};

/// get_404_not_found
/// Serve 404 Not found HTML file
pub async fn get_404_not_found() -> Html<String> {
    let not_found = NotFoundTemplate.render();
    match not_found {
        Ok(res) => {
            info!("NotFound askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render 404_not_found.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_500_internal_server_error
/// Serve 500 Internal Server Error HTML file
pub fn get_500_internal_server_error() -> Html<String> {
    let internal_server_error = InternalServerErrorTemplate.render();
    match internal_server_error {
        Ok(res) => {
            info!("InternalServerError askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render 500_internal_server_error.html. {}", err);
            Html("We're fucked up.".to_string())
        }
    }
}
