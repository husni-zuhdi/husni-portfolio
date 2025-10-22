use crate::model::axum::AppState;
use crate::{
    handler::{
        auth::{process_login_header, verify_jwt},
        status::get_404_not_found,
        HX_REDIRECT,
    },
    model::templates::{LoginRetryTemplate, LoginSuccessTemplate, LoginTemplate, LogoutTemplate},
};
use askama::Template;
use axum::extract::State;
use axum::{http::HeaderMap, response::Html};
use tracing::{error, info};

/// get_login
/// Serve Login HTML template
pub async fn get_login(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> (HeaderMap, Html<String>) {
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    let mut resp_headers = HeaderMap::new();
    // Redirect User to Admin Blog when token is verified
    if verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        resp_headers.insert(HX_REDIRECT, "/admin".parse().unwrap());
    }

    let login = LoginTemplate.render();
    match login {
        Ok(res) => {
            info!("Get Login askama template rendered.");
            (resp_headers, Html(res))
        }
        Err(err) => {
            error!("Failed to render auth/login.html. {}", err);
            (resp_headers, get_404_not_found().await)
        }
    }
}

/// get_login_retry
/// Serve Login Retry HTML template
pub async fn get_login_retry(header_map: Option<HeaderMap>) -> (HeaderMap, Html<String>) {
    let header_map_final = match header_map {
        Some(hm) => hm,
        None => HeaderMap::new(),
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
    let header_map_final = match header_map {
        Some(hm) => hm,
        None => HeaderMap::new(),
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
pub async fn get_logout(headers: HeaderMap) -> (HeaderMap, Html<String>) {
    let login = LogoutTemplate.render();
    match login {
        Ok(res) => {
            info!("Get Logout askama template rendered.");
            (headers, Html(res))
        }
        Err(err) => {
            error!("Failed to render auth/logout.html. {}", err);
            (headers, get_404_not_found().await)
        }
    }
}
