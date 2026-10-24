use super::{process_profile_meta_body, process_profile_section_body, rebuild_profile_page_cache};
use crate::handler::admin::profile::displays::{
    get_admin_profile_list, get_admin_profile_list_with_error, get_admin_profile_section,
};
use crate::handler::auth::csrf::verify_csrf_token;
use crate::handler::auth::is_auth_verified;
use crate::handler::status::get_401_unauthorized;
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::axum::AppState;
use crate::model::profile::{MoveParams, ProfileCommandStatus};
use crate::model::templates_admin::AdminProfilePreviewTemplate;
use askama::Template;
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use std::str::FromStr;
use tracing::{debug, error, info, warn};
use urlencoding::decode;

/// post_add_admin_profile_section
/// Serve POST add profile section endpoint
#[debug_handler]
pub async fn post_add_admin_profile_section(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let mut profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    let section_res = process_profile_section_body(body);
    if section_res.is_none() {
        warn!("Failed to process a new Profile Section body.");
        return get_500_internal_server_error();
    }
    let section = section_res.unwrap();

    let add_result = profile_db_uc.profile_operation_repo.add(section).await;
    if add_result.is_none() {
        info!("Failed to add Profile Section.");
        return get_404_not_found().await;
    }

    if add_result.unwrap() != ProfileCommandStatus::Stored {
        error!("Failed to add Profile Section.");
        return get_500_internal_server_error();
    }

    if is_cache_enabled {
        rebuild_profile_page_cache(&app_state).await;
    }

    get_admin_profile_list(State(app_state), headers).await
}

/// put_edit_admin_profile_section
/// Serve PUT edit profile section HTML file
#[debug_handler]
pub async fn put_edit_admin_profile_section(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let mut profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    // Sanitize `path`
    let id = path.parse::<i64>();
    match &id {
        Ok(val) => {
            debug!("Successfully parse path {} into {} i64", &path, &val);
        }
        Err(err) => {
            warn!("Failed to parse path {} to i64. Err: {}", &path, err);
            return get_404_not_found().await;
        }
    };

    let section_res = process_profile_section_body(body);
    if section_res.is_none() {
        warn!("Failed to process Profile Section body with Id {}.", path);
        return get_500_internal_server_error();
    }
    let mut section = section_res.unwrap();
    section.id = id.unwrap();

    let edit_result = profile_db_uc.profile_operation_repo.update(section).await;
    if edit_result.is_none() {
        info!("Failed to edit Profile Section with Id {}.", path);
        return get_404_not_found().await;
    }

    if edit_result.unwrap() != ProfileCommandStatus::Updated {
        error!("Failed to edit Profile Section with Id {}", path);
        return get_500_internal_server_error();
    }

    if is_cache_enabled {
        rebuild_profile_page_cache(&app_state).await;
    }

    get_admin_profile_section(Path(path), State(app_state), headers).await
}

/// post_move_admin_profile_section
/// Serve POST move profile section endpoint (reorder up/down)
#[debug_handler]
pub async fn post_move_admin_profile_section(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
    params: Query<MoveParams>,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let mut profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    // Sanitize `path`
    let id = path.parse::<i64>();
    match &id {
        Ok(val) => {
            debug!("Successfully parse path {} into {} i64", &path, &val);
        }
        Err(err) => {
            warn!("Failed to parse path {} to i64. Err: {}", &path, err);
            return get_404_not_found().await;
        }
    };

    let direction = params.sanitize();
    let move_result = profile_db_uc
        .profile_operation_repo
        .move_section(id.unwrap(), direction)
        .await;
    if move_result.is_none() {
        info!("Failed to move Profile Section with Id {}.", path);
        return get_500_internal_server_error();
    }

    if is_cache_enabled {
        rebuild_profile_page_cache(&app_state).await;
    }

    get_admin_profile_list(State(app_state), headers).await
}

/// delete_delete_admin_profile_section
/// Serve DELETE delete profile section HTML file.
/// The page title section (first by position) is protected from deletion;
/// deleting it re-renders the list with an error notice.
#[debug_handler]
pub async fn delete_delete_admin_profile_section(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let mut profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    // Sanitize `path`
    let id = path.parse::<i64>();
    match &id {
        Ok(val) => {
            debug!("Successfully parse path {} into {} i64", &path, &val);
        }
        Err(err) => {
            warn!("Failed to parse path {} to i64. Err: {}", &path, err);
            return get_404_not_found().await;
        }
    };

    let delete_result = profile_db_uc
        .profile_operation_repo
        .delete(id.unwrap())
        .await;

    if delete_result.is_none() {
        info!(
            "Failed to delete Profile Section with Id {}. It may be the protected title section.",
            path
        );
        return get_admin_profile_list_with_error(
            app_state,
            format!(
                "Section with ID {} is the page title section. It cannot be deleted.",
                path
            ),
        )
        .await;
    }

    if delete_result.unwrap() != ProfileCommandStatus::Deleted {
        error!("Failed to delete Profile Section with Id {}", path);
        return get_500_internal_server_error();
    }

    if is_cache_enabled {
        rebuild_profile_page_cache(&app_state).await;
    }

    get_admin_profile_list(State(app_state), headers).await
}

/// put_admin_profile_meta_edit
/// Serve PUT edit profile meta HTML file
#[debug_handler]
pub async fn put_admin_profile_meta_edit(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let mut profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    let meta_res = process_profile_meta_body(body);
    if meta_res.is_none() {
        warn!("Failed to process Profile Meta body.");
        return get_500_internal_server_error();
    }
    let meta = meta_res.unwrap();

    let edit_result = profile_db_uc.profile_operation_repo.update_meta(meta).await;
    if edit_result.is_none() {
        info!("Failed to edit Profile Meta.");
        return get_404_not_found().await;
    }

    if edit_result.unwrap() != ProfileCommandStatus::Updated {
        error!("Failed to edit Profile Meta.");
        return get_500_internal_server_error();
    }

    if is_cache_enabled {
        rebuild_profile_page_cache(&app_state).await;
    }

    get_admin_profile_list(State(app_state), headers).await
}

/// post_admin_profile_preview
/// Serve POST preview profile section content endpoint.
/// Renders the markdown body without touching the database
#[debug_handler]
pub async fn post_admin_profile_preview(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    // Extract `section_body` and `section_layout` from the form body
    let mut section_body = String::new();
    let mut section_layout = String::new();
    for req_field in body.split("&") {
        let Some((key, value)) = req_field.split_once("=") else {
            continue;
        };
        let value_decoded = decode(value).unwrap_or_default();
        match key {
            "section_body" => section_body = value_decoded.to_string(),
            "section_layout" => section_layout = value_decoded.to_string(),
            _ => {}
        }
    }

    let layout = crate::model::profile::Layout::from_str(&section_layout)
        .unwrap_or(crate::model::profile::Layout::Stacked);
    let preview_res = AdminProfilePreviewTemplate {
        html: crate::utils::convert_markdown_to_html(&section_body),
        css_class: layout.as_css_class().to_string(),
    }
    .render();

    match preview_res {
        Ok(html) => {
            info!("AdminProfilePreview askama template rendered.");
            Html(html)
        }
        Err(err) => {
            error!("Failed to render admin/profile/preview.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Config;
    use crate::state::state_factory;
    use axum::extract::Path;

    fn unauthorized_html(html: &Html<String>) -> bool {
        html.0.contains("401 Unauthorized")
    }

    #[tokio::test]
    async fn test_post_add_admin_profile_section_unauthorized() {
        let state = state_factory(Config::default()).await;
        let headers = HeaderMap::new();
        let html = post_add_admin_profile_section(State(state), headers, String::new()).await;
        assert!(unauthorized_html(&html));
    }

    #[tokio::test]
    async fn test_put_edit_admin_profile_section_unauthorized() {
        let state = state_factory(Config::default()).await;
        let headers = HeaderMap::new();
        let html = put_edit_admin_profile_section(
            Path("1".to_string()),
            State(state),
            headers,
            String::new(),
        )
        .await;
        assert!(unauthorized_html(&html));
    }

    #[tokio::test]
    async fn test_post_move_admin_profile_section_unauthorized() {
        let state = state_factory(Config::default()).await;
        let headers = HeaderMap::new();
        let html = post_move_admin_profile_section(
            Path("1".to_string()),
            State(state),
            headers,
            Query(MoveParams { dir: None }),
        )
        .await;
        assert!(unauthorized_html(&html));
    }

    #[tokio::test]
    async fn test_delete_delete_admin_profile_section_unauthorized() {
        let state = state_factory(Config::default()).await;
        let headers = HeaderMap::new();
        let html =
            delete_delete_admin_profile_section(Path("1".to_string()), State(state), headers).await;
        assert!(unauthorized_html(&html));
    }

    #[tokio::test]
    async fn test_put_admin_profile_meta_edit_unauthorized() {
        let state = state_factory(Config::default()).await;
        let headers = HeaderMap::new();
        let html = put_admin_profile_meta_edit(State(state), headers, String::new()).await;
        assert!(unauthorized_html(&html));
    }

    #[tokio::test]
    async fn test_post_admin_profile_preview_unauthorized() {
        let state = state_factory(Config::default()).await;
        let headers = HeaderMap::new();
        let html = post_admin_profile_preview(State(state), headers, String::new()).await;
        assert!(unauthorized_html(&html));
    }

    #[tokio::test]
    async fn test_post_admin_profile_preview_renders_sanitized_html() {
        let state = state_factory(Config::default()).await;

        // Authorized request with valid JWT and CSRF token
        let secret = &state.config.secrets.jwt_secret.clone();
        let token = crate::handler::auth::create_jwt(secret).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            axum::http::HeaderValue::from_str(&format!("token={token}; _csrf_token=abc123"))
                .unwrap(),
        );
        headers.insert(
            crate::handler::auth::csrf::CSRF_HEADER_NAME.clone(),
            axum::http::HeaderValue::from_str("abc123").unwrap(),
        );

        let body = "section_body=%23%20Hello&section_layout=stacked&section_id=1&section_title=T"
            .to_string();
        let html = post_admin_profile_preview(State(state), headers, body).await;
        assert!(html.0.contains("<h1>Hello</h1>"));
    }
}
