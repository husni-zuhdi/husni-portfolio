use crate::model::templates::{
    IamATeapotTemplate, InternalServerErrorTemplate, NotFoundTemplate, UnauthorizedTemplate,
};
use askama::Template;
use axum::response::Html;
use tracing::{error, info};

/// get_401_unauthorized
/// Serve 401 Unauthorized HTML file
pub async fn get_401_unauthorized() -> Html<String> {
    let unauthorized = UnauthorizedTemplate.render();
    match unauthorized {
        Ok(res) => {
            info!("Unauthorized askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render statuses/401_unauthorized.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

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
            error!("Failed to render statuses/404_not_found.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_418_i_am_a_teapot
/// Serve 418 I am a teapot HTML file
pub async fn get_418_i_am_a_teapot() -> Html<String> {
    let not_found = IamATeapotTemplate.render();
    match not_found {
        Ok(res) => {
            info!("I am A Teapot askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render statuses/418_i_am_a_teapot.html. {}", err);
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
            error!(
                "Failed to render statuses/500_internal_server_error.html. {}",
                err
            );
            Html("We're fucked up.".to_string())
        }
    }
}
