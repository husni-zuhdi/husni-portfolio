use crate::handler::admin::talks::displays::{get_admin_talk, get_admin_talks_list};
use crate::handler::admin::talks::process_talk_body;
use crate::handler::auth::csrf::verify_csrf_token;
use crate::handler::auth::is_auth_verified;
use crate::handler::status::get_401_unauthorized;
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::axum::AppState;
use crate::model::talks::{TalkCommandStatus, TalksParams};
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use tracing::{error, info, warn};

/// post_add_admin_talk
/// Serve POST add talk endpoint
#[debug_handler]
pub async fn post_add_admin_talk(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let mut talk_uc = app_state.talk_usecase.lock().await.clone();

    let talk_res = process_talk_body(body);
    if talk_res.is_none() {
        warn!("Failed to process a new Talk body.");
        return get_500_internal_server_error();
    }
    let talk = talk_res.unwrap().sanitize_talk_media_org();

    let add_result = talk_uc.add(talk.clone()).await;
    if add_result.is_none() {
        info!("Failed to add Talk with Id {}.", &talk.id);
        return get_404_not_found().await;
    }
    if add_result.unwrap() != TalkCommandStatus::Stored {
        error!("Failed to add Talk with Id {}", &talk.id);
        return get_500_internal_server_error();
    }

    info!("Added Talk with Id {}.", &talk.id);

    let params = TalksParams {
        start: None,
        end: None,
    };
    get_admin_talks_list(State(app_state), headers, Query(params)).await
}

/// put_edit_admin_talk
/// Serve PUT edit talk HTML file
#[debug_handler]
pub async fn put_edit_admin_talk(
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

    // Sanitize `path` as a sanity gate; the edit target id comes from the
    // talk body, matching the previous edit flow.
    let Ok(_id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    let mut talk_uc = app_state.talk_usecase.lock().await.clone();

    let talk_res = process_talk_body(body);
    if talk_res.is_none() {
        warn!("Failed to process Talk body with Id {}.", &path);
        return get_500_internal_server_error();
    }
    let talk = talk_res.unwrap().sanitize_talk_media_org();

    let edit_result = talk_uc.update(talk).await;
    if edit_result.is_none() {
        info!("Failed to edit Talk with Id {}.", &path);
        return get_404_not_found().await;
    }
    if edit_result.unwrap() != TalkCommandStatus::Updated {
        error!("Failed to edit Talk with Id {}", &path);
        return get_500_internal_server_error();
    }

    info!("Edited Talk with Id {}.", &path);
    get_admin_talk(Path(path), State(app_state), headers).await
}

/// delete_delete_admin_talk
/// Serve DELETE delete talk HTML file
#[debug_handler]
pub async fn delete_delete_admin_talk(
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

    let mut talk_uc = app_state.talk_usecase.lock().await.clone();

    let delete_result = talk_uc.delete(id).await;
    if delete_result.is_none() || delete_result.unwrap() != TalkCommandStatus::Deleted {
        error!("Failed to delete Talk with Id {}", &path);
        return get_500_internal_server_error();
    }

    info!("Deleted Talk with Id {}.", &path);

    let params = TalksParams {
        start: None,
        end: None,
    };
    get_admin_talks_list(State(app_state), headers, Query(params)).await
}
