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

    if !verify_jwt(&token, &app_state.config.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    // Locking Mutex
    let tag_uc = app_state.tag_usecase.lock().await.clone();

    let tag = process_tag_body(body);
    let add_result = tag_uc.unwrap().tag_repo.add(tag.id, tag.name).await;

    match add_result {
        Some(tag_command_status) => {
            if tag_command_status != TagCommandStatus::Stored {
                error!("Failed to add tag with Id {}", &tag.id);
                return get_500_internal_server_error();
            }
        }
        None => {
            info!("Failed to add tag with Id {}.", &tag.id);
            return get_404_not_found().await;
        }
    }

    let query_params = TagsListParams {
        start: None,
        end: None,
    };

    get_admin_tags_list(State(app_state), headers, Query(query_params)).await
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

    if !verify_jwt(&token, &app_state.config.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let tag_uc = app_state.tag_usecase.lock().await.clone();
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

    let update_result = tag_uc
        .unwrap()
        .tag_repo
        .update(id.unwrap(), Some(tag.name))
        .await;

    match update_result {
        Some(tag_command_status) => {
            if tag_command_status != TagCommandStatus::Updated {
                error!("Failed to edit Tag with Id {}", &path);
                return get_500_internal_server_error();
            }
        }
        None => {
            info!("Failed to edit Tag with Id {}.", &path);
            return get_404_not_found().await;
        }
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

    if !verify_jwt(&token, &app_state.config.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let tag_uc = app_state.tag_usecase.lock().await.clone();
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

    let delete_result = tag_uc.unwrap().tag_repo.delete(id.unwrap()).await;

    match delete_result {
        Some(tag_command_status) => {
            if tag_command_status != TagCommandStatus::Deleted {
                error!("Failed to delete Tag with Id {}", &path);
                return get_500_internal_server_error();
            }
        }
        None => {
            info!("Failed to delete Tag with Id {}.", &path);
            return get_404_not_found().await;
        }
    }

    let query_params = TagsListParams {
        start: None,
        end: None,
    };

    get_admin_tags_list(State(app_state), headers, Query(query_params)).await
}
