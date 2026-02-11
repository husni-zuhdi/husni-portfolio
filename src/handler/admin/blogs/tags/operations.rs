use crate::handler::admin::blogs::tags::displays::{get_admin_tag, get_admin_tags_list};
use crate::handler::admin::blogs::tags::process_tag_body;
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::axum::AppState;
use crate::model::tags::{TagCommandStatus, TagsListParams};
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::response::Html;
use tracing::{debug, error, info, warn};

use crate::handler::auth::{process_login_header, verify_jwt};
use crate::handler::status::get_401_unauthorized;
use axum::http::HeaderMap;

/// post_add_admin_tag
/// Serve POST add tag endpoint
#[debug_handler]
pub async fn post_add_admin_tag(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    let (user_agent, token) = process_login_header(headers.clone()).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    // Locking Mutex
    let tag_uc = app_state.tag_db_usecase.lock().await.clone();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_cache_enabled = tags_cache_uc_opt.is_some();

    let tag = process_tag_body(body);
    let add_result = tag_uc
        .unwrap()
        .tag_operation_repo
        .add(tag.id, tag.clone().name)
        .await;

    if add_result.is_none() {
        info!("Failed to add Tag with Id {}.", &tag.id);
        return get_404_not_found().await;
    }

    if add_result.unwrap() != TagCommandStatus::Stored {
        error!("Failed to add Tag with Id {}", &tag.id);
        return get_500_internal_server_error();
    }

    // Insert cache
    if is_cache_enabled {
        debug!("Caching tag {}", &tag.id);
        let _ = tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_operation_repo
            .insert(tag.clone())
            .await;
    }

    let params = TagsListParams {
        start: None,
        end: None,
    };

    get_admin_tags_list(State(app_state), headers, Query(params)).await
}

/// put_edit_admin_tag
/// Serve PUT edit tag HTML file
#[debug_handler]
pub async fn put_edit_admin_tag(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    let (user_agent, token) = process_login_header(headers.clone()).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let tag_uc = app_state.tag_db_usecase.lock().await.clone();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_cache_enabled = tags_cache_uc_opt.is_some();

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

    let tag = process_tag_body(body);

    let edit_result = tag_uc
        .unwrap()
        .tag_operation_repo
        .update(id.unwrap(), Some(tag.clone().name))
        .await;

    if edit_result.is_none() {
        info!("Failed to edit Tag with Id {}.", &tag.id);
        return get_404_not_found().await;
    }

    if edit_result.unwrap() != TagCommandStatus::Updated {
        error!("Failed to edit Tag with Id {}", &tag.id);
        return get_500_internal_server_error();
    }

    // Insert cache
    if is_cache_enabled {
        debug!("Invalidating tag {} cache", &tag.id);
        let _ = tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_operation_repo
            .invalidate(tag.id)
            .await;
        debug!("Re-caching tag {}", &tag.id);
        let _ = tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_operation_repo
            .insert(tag.clone())
            .await;
    }

    get_admin_tag(Path(path), State(app_state), headers).await
}

/// delete_delete_admin_tag
/// Serve DELETE delete tag HTML file
#[debug_handler]
pub async fn delete_delete_admin_tag(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let (user_agent, token) = process_login_header(headers.clone()).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let tag_uc = app_state.tag_db_usecase.lock().await.clone();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_cache_enabled = tags_cache_uc_opt.is_some();

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

    let delete_result = tag_uc
        .unwrap()
        .tag_operation_repo
        .delete(id.clone().unwrap())
        .await;

    if delete_result.is_none() || (delete_result.unwrap() != TagCommandStatus::Updated) {
        error!("Failed to edit Tag with Id {}", &id.clone().unwrap());
        return get_500_internal_server_error();
    }

    // Invalidate cache
    if is_cache_enabled {
        debug!("Invalidating tag {} cache", &id.clone().unwrap());
        let _ = tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_operation_repo
            .invalidate(id.unwrap())
            .await;
    }

    let params = TagsListParams {
        start: None,
        end: None,
    };

    get_admin_tags_list(State(app_state), headers, Query(params)).await
}
