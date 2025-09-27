use crate::handler::auth::displays::{get_login_retry, get_login_sucess};
use crate::handler::auth::{
    create_jwt_token, is_password_match, process_login_body, sanitize_email, sanitize_password,
};
use crate::handler::HX_REDIRECT;
use crate::model::axum::AppState;
use crate::repo::auth::AuthRepo;
use axum::debug_handler;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use tracing::warn;

/// post_login
/// Serve POST login endpoint.
/// Verify if submitted email is valid and password matched with the hashed password.
/// If success, redirect user to /admin/blogs
/// If failed, inform user the email and password combination is wrong.
#[debug_handler]
pub async fn post_login(State(app_state): State<AppState>, body: String) -> impl IntoResponse {
    let auth_uc = app_state.auth_usecase.lock().await.clone().unwrap();
    let (email, password) = process_login_body(&body).unwrap();
    let sanitized_email = sanitize_email(&email);
    let sanitized_password = sanitize_password(&password);
    if sanitized_email.is_none() {
        warn!("Rendering login retry. Failed email sanitation");
        return get_login_retry(None).await;
    }

    let get_user_result = auth_uc.find_user_by_email(sanitized_email.unwrap()).await;
    if get_user_result.is_none() {
        warn!("Rendering login retry. Cannot find User");
        return get_login_retry(None).await;
    }

    let hashed_password = get_user_result.unwrap().hashed_password;
    if !is_password_match(&sanitized_password, &hashed_password) {
        warn!("Rendering login retry. Password is wrong");
        return get_login_retry(None).await;
    }

    // Create JWT (Claim and) Token
    let token = create_jwt_token();
    if token.is_none() {
        warn!("Rendering login retry. Failed to generate JWT Token");
        return get_login_retry(None).await;
    }

    let mut header_map = HeaderMap::new();
    let jwt_token_cookie = format!("token={}; Secure; HttpOnly; SameSite=Lax", token.unwrap());
    header_map.insert(SET_COOKIE, jwt_token_cookie.parse().unwrap());
    header_map.insert(HX_REDIRECT, "/admin/blogs".parse().unwrap());

    // Render HTML with header to set JWT Token in header
    get_login_sucess(Some(header_map)).await
}
