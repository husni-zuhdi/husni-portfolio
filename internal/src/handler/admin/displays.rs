use crate::handler::auth::{process_login_header, verify_jwt};
use crate::handler::status::{get_401_unauthorized, get_500_internal_server_error};
use crate::model::axum::AppState;
use crate::model::templates_admin::AdminTemplate;
use askama::Template;
use axum::debug_handler;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Html;
use tracing::{error, info};

/// get_base_admin
/// Serve GET (base) admin HTML file
/// Under endpoint /admin
#[debug_handler]
pub async fn get_base_admin(State(app_state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    // Display 401 when User doesn't provide a JWT
    if !verify_jwt(&token, &app_state.config.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let admin_res = AdminTemplate {}.render();
    match admin_res {
        Ok(res) => {
            info!("Talks askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render admin/admin.html. {}", err);
            get_500_internal_server_error()
        }
    }
}
