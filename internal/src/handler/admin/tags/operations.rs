use crate::handler::admin::blogs::displays::get_admin_blogs;
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::axum::AppState;
use crate::model::blogs::BlogsParams;
use crate::model::tags::{Tag, TagCommandStatus};
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::response::Html;
use tracing::{debug, error, info, warn};
use urlencoding::decode;

// Take request tag String from PUT and POST operations to create a new tag
fn process_tag_body(body: String) -> Tag {
    // Initialize fields
    let mut tag_id = 0_i64;
    let mut tag_name = String::new();

    let req_fields: Vec<&str> = body.split("&").collect();
    for req_field in req_fields {
        let (key, value) = req_field.split_once("=").unwrap();
        let value_decoded = decode(value).unwrap();
        debug!("Request field key/value {:?}/{:?}", key, value_decoded);
        match key {
            "tag_id" => {
                tag_id = value_decoded
                    .parse::<i64>()
                    .expect("Failed to parse path from request body")
            }
            "tag_name" => tag_name = value_decoded.to_string(),
            _ => {
                warn!("Unrecognized key/value: {:?}/{:?}", key, value_decoded);
                continue;
            }
        }
    }

    Tag {
        id: tag_id,
        name: tag_name,
    }
}

/// post_add_admin_tag
/// Serve POST add tag endpoint
#[debug_handler]
pub async fn post_add_admin_tag(State(app_state): State<AppState>, body: String) -> Html<String> {
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

    let query_params = BlogsParams {
        start: None,
        end: None,
        tags: None,
    };

    get_admin_blogs(State(app_state), Query(query_params)).await
}

/// put_edit_admin_tag
/// Serve PUT edit tag HTML file
#[debug_handler]
pub async fn put_edit_admin_tag(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    body: String,
) -> Html<String> {
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

    let query_params = BlogsParams {
        start: None,
        end: None,
        tags: None,
    };

    get_admin_blogs(State(app_state), Query(query_params)).await
}

/// delete_delete_admin_tag
/// Serve DELETE delete tag HTML file
#[debug_handler]
pub async fn delete_delete_admin_tag(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
) -> Html<String> {
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

    let query_params = BlogsParams {
        start: None,
        end: None,
        tags: None,
    };

    get_admin_blogs(State(app_state), Query(query_params)).await
}
