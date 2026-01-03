use crate::handler::admin::talks::displays::{get_admin_talk, get_admin_talks_list};
use crate::handler::admin::talks::process_talk_body;
use crate::handler::auth::{process_login_header, verify_jwt};
use crate::handler::status::get_401_unauthorized;
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::axum::AppState;
use crate::model::talks::{TalkCommandStatus, TalksParams};
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use tracing::{debug, error, info, warn};

/// post_add_admin_talk
/// Serve POST add talk endpoint
#[debug_handler]
pub async fn post_add_admin_talk(
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

    let mut talks_db_uc = app_state.talk_db_usecase.lock().await.clone().unwrap();
    let talks_cache_uc_opt = app_state.talk_cache_usecase.lock().await.clone();
    let is_cache_enabled = talks_cache_uc_opt.is_some();

    let talk_res = process_talk_body(body);
    if talk_res.is_none() {
        warn!("Failed to process a new Talk body.");
        return get_500_internal_server_error();
    }
    let talk = talk_res.unwrap().sanitize_talk_media_org();

    let add_result = talks_db_uc
        .talk_operation_repo
        .add(
            talk.id,
            talk.name.clone(),
            talk.date.clone(),
            talk.media_link.clone(),
            talk.org_name.clone(),
            talk.org_link.clone(),
        )
        .await;

    if add_result.is_none() {
        info!("Failed to add Talk with Id {}.", &talk.id);
        return get_404_not_found().await;
    }

    if add_result.unwrap() != TalkCommandStatus::Stored {
        error!("Failed to add Talk with Id {}", &talk.id);
        return get_500_internal_server_error();
    }

    if is_cache_enabled {
        debug!("Caching talk {}", talk.id);
        let _ = talks_cache_uc_opt
            .clone()
            .unwrap()
            .talk_operation_repo
            .insert(talk.clone())
            .await;
    }

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
    let (user_agent, token) = process_login_header(headers.clone()).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let mut talks_db_uc = app_state.talk_db_usecase.lock().await.clone().unwrap();
    let talks_cache_uc_opt = app_state.talk_cache_usecase.lock().await.clone();
    let is_cache_enabled = talks_cache_uc_opt.is_some();

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

    let talk_res = process_talk_body(body);
    if talk_res.is_none() {
        warn!("Failed to process Talk body with Id {}.", &path);
        return get_500_internal_server_error();
    }
    let talk = talk_res.unwrap().sanitize_talk_media_org();

    let edit_result = talks_db_uc
        .talk_operation_repo
        .update(
            talk.id,
            Some(talk.name.clone()),
            Some(talk.date.clone()),
            talk.media_link.clone(),
            talk.org_name.clone(),
            talk.org_link.clone(),
        )
        .await;

    if edit_result.is_none() {
        info!("Failed to edit Talk with Id {}.", &path);
        return get_404_not_found().await;
    }

    if edit_result.unwrap() != TalkCommandStatus::Updated {
        error!("Failed to edit Talk with Id {}", &path);
        return get_500_internal_server_error();
    }

    if is_cache_enabled {
        debug!("Invalidating talk {} cache", talk.id);
        let _ = talks_cache_uc_opt
            .clone()
            .unwrap()
            .talk_operation_repo
            .invalidate(talk.id)
            .await;
        debug!("Re-caching talk {}", talk.id);
        let _ = talks_cache_uc_opt
            .clone()
            .unwrap()
            .talk_operation_repo
            .insert(talk.clone())
            .await;
    }

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
    let (user_agent, token) = process_login_header(headers.clone()).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let mut talks_db_uc = app_state.talk_db_usecase.lock().await.clone().unwrap();
    let talks_cache_uc_opt = app_state.talk_cache_usecase.lock().await.clone();
    let is_cache_enabled = talks_cache_uc_opt.is_some();

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

    let delete_result = talks_db_uc
        .talk_operation_repo
        .delete(id.clone().unwrap())
        .await;

    if delete_result.is_none() {
        info!("Failed to edit Talk with Id {}.", &path);
        return get_404_not_found().await;
    }

    if delete_result.unwrap() != TalkCommandStatus::Deleted {
        error!("Failed to delete Talk with Id {}", &path);
        return get_500_internal_server_error();
    }

    if is_cache_enabled {
        debug!("Invalidating talk {} cache", id.clone().unwrap());
        let _ = talks_cache_uc_opt
            .clone()
            .unwrap()
            .talk_operation_repo
            .invalidate(id.unwrap())
            .await;
    }

    let params = TalksParams {
        start: None,
        end: None,
    };
    get_admin_talks_list(State(app_state), headers, Query(params)).await
}
