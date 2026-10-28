use crate::handler::admin::blogs::tags::displays::{get_admin_tag, get_admin_tags_list};
use crate::handler::admin::blogs::tags::process_tag_body;
use crate::handler::auth::csrf::verify_csrf_token;
use crate::handler::status::{
    get_401_unauthorized, get_404_not_found, get_500_internal_server_error,
};
use crate::model::axum::AppState;
use crate::model::tags::{Tag, TagCommandStatus, TagsListParams};
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::response::Html;
use tracing::{error, info, warn};

use crate::handler::auth::is_auth_verified;
use axum::http::HeaderMap;

/// post_add_admin_tag
/// Serve POST add tag endpoint
#[debug_handler]
pub async fn post_add_admin_tag(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let mut tag_uc = app_state.tag_usecase.lock().await.clone();
    let tag = process_tag_body(body);

    let add_result = tag_uc.add(tag.clone()).await;
    if add_result.is_none() {
        info!("Failed to add Tag with Id {}.", &tag.id);
        return get_404_not_found().await;
    }
    if add_result.unwrap() != TagCommandStatus::Stored {
        error!("Failed to add Tag with Id {}", &tag.id);
        return get_500_internal_server_error();
    }

    info!("Added Tag with Id {}.", &tag.id);

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
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    let mut tag_uc = app_state.tag_usecase.lock().await.clone();
    let tag = process_tag_body(body);

    let edit_result = tag_uc.update(Tag { id, name: tag.name }).await;
    if edit_result.is_none() {
        info!("Failed to edit Tag with Id {}.", &id);
        return get_404_not_found().await;
    }
    if edit_result.unwrap() != TagCommandStatus::Updated {
        error!("Failed to edit Tag with Id {}", &id);
        return get_500_internal_server_error();
    }

    info!("Edited Tag with Id {}.", &id);
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
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    let mut tag_uc = app_state.tag_usecase.lock().await.clone();

    let delete_result = tag_uc.delete(id).await;
    if delete_result.is_none() || (delete_result.unwrap() != TagCommandStatus::Deleted) {
        error!("Failed to delete Tag with Id {}", &id);
        return get_500_internal_server_error();
    }

    info!("Deleted Tag with Id {}.", &id);

    let params = TagsListParams {
        start: None,
        end: None,
    };

    get_admin_tags_list(State(app_state), headers, Query(params)).await
}
