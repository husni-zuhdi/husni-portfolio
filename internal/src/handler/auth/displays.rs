use crate::{
    handler::status::get_404_not_found,
    model::templates::{LoginRetryTemplate, LoginSuccessTemplate, LoginTemplate, LogoutTemplate},
};
use askama::Template;
use axum::{http::HeaderMap, response::Html};
use tracing::{error, info};

/// get_login
/// Serve Login HTML template
pub async fn get_login() -> Html<String> {
    let login = LoginTemplate.render();
    match login {
        Ok(res) => {
            info!("Get Login askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render auth/login.html. {}", err);
            get_404_not_found().await
        }
    }
}

/// get_login_retry
/// Serve Login Retry HTML template
pub async fn get_login_retry(header_map: Option<HeaderMap>) -> (HeaderMap, Html<String>) {
    let header_map_final = if header_map.is_none() {
        HeaderMap::new()
    } else {
        header_map.unwrap()
    };

    let login_retry = LoginRetryTemplate.render();
    match login_retry {
        Ok(res) => {
            info!("Get Login Retry askama template rendered.");
            (header_map_final, Html(res))
        }
        Err(err) => {
            error!("Failed to render auth/login_retry.html. {}", err);
            (header_map_final, get_404_not_found().await)
        }
    }
}

/// get_login_success
/// Serve Login Success HTML template
pub async fn get_login_sucess(header_map: Option<HeaderMap>) -> (HeaderMap, Html<String>) {
    let header_map_final = if header_map.is_none() {
        HeaderMap::new()
    } else {
        header_map.unwrap()
    };
    let login_success = LoginSuccessTemplate.render();
    match login_success {
        Ok(res) => {
            info!("Get Login Success askama template rendered.");
            (header_map_final, Html(res))
        }
        Err(err) => {
            error!("Failed to render auth/login_success.html. {}", err);
            (header_map_final, get_404_not_found().await)
        }
    }
}

/// get_logout
/// Serve Logout HTML template
pub async fn get_logout() -> Html<String> {
    let login = LogoutTemplate.render();
    match login {
        Ok(res) => {
            info!("Get Logout askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render auth/logout.html. {}", err);
            get_404_not_found().await
        }
    }
}
