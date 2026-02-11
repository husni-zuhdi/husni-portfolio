use crate::handler::auth::{process_login_header, verify_jwt};
use crate::handler::status::{
    get_401_unauthorized, get_404_not_found, get_500_internal_server_error,
};
use crate::model::talks::TalksParams;
use crate::model::{
    axum::AppState,
    templates_admin::{
        AdminGetAddTalkTemplate, AdminGetDeleteTalkTemplate, AdminGetEditTalkTemplate,
        AdminGetTalkTemplate, AdminTalksTemplate,
    },
};
use crate::repo::talks::TalkDisplayRepo;
use askama::Template;
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use tracing::{debug, error, info, warn};

/// get_base_admin_talks
/// Serve GET (base) admin talks HTML file
/// Under endpoint /admin/talks
/// It's the base of Admin Talks feature
#[debug_handler]
pub async fn get_base_admin_talks(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let talks_res = AdminTalksTemplate {}.render();
    match talks_res {
        Ok(res) => {
            info!("Talks askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render admin/talks/talks.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_admin_talks_list
/// Serve to list talks HTML file and return point for /admin/talks/add cancel button
#[debug_handler]
pub async fn get_admin_talks_list(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    params: Query<TalksParams>,
) -> Html<String> {
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let talks_db_uc = app_state.talk_db_usecase.lock().await.clone().unwrap();
    let talks_cache_uc_opt = app_state.talk_cache_usecase.lock().await.clone();
    let is_cache_enabled = talks_cache_uc_opt.is_some();

    let sanitized_params = params.sanitize();

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        talks_cache_uc_opt
            .clone()
            .unwrap()
            .find_talks(sanitized_params.clone())
            .await
    } else {
        None
    };
    // If cache hit, return early
    if let Some(res) = cache_result {
        let talks_res = res.sanitize().to_admin_list_template().render();
        if talks_res.is_err() {
            error!(
                "Failed to render get_talks.html. {}",
                talks_res.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("Talks askama template rendered.");
        return Html(talks_res.unwrap());
    }

    // If not, get data from database
    let db_result = talks_db_uc
        .talk_display_repo
        .find_talks(sanitized_params.clone())
        .await;

    if db_result.is_none() {
        error!(
            "Failed to find talks with Talk Id started at {} and ended at {}.",
            &sanitized_params.start.unwrap(),
            &sanitized_params.end.unwrap()
        );
        return get_500_internal_server_error();
    }

    // Insert cache
    if is_cache_enabled {
        for talk in db_result.clone().unwrap().talks {
            debug!("Caching talk {}", &talk.id);
            let _ = talks_cache_uc_opt
                .clone()
                .unwrap()
                .talk_operation_repo
                .insert(talk)
                .await;
        }
    }

    // Render Admin List Talks
    let talks_res = db_result
        .unwrap()
        .sanitize()
        .to_admin_list_template()
        .render();
    if talks_res.is_err() {
        error!(
            "Failed to render admin/talks/list_talks.html. {}",
            talks_res.unwrap_err()
        );
        return get_500_internal_server_error();
    }

    info!("AdminListTalks askama template rendered.");
    Html(talks_res.unwrap())
}

/// get_admin_talk
/// Serve GET talk HTML file and return point for several cancelation endpoints
/// Returned single talk
#[debug_handler]
pub async fn get_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let talks_db_uc = app_state.talk_db_usecase.lock().await.clone().unwrap();
    let talks_cache_uc_opt = app_state.talk_cache_usecase.lock().await;
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

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        talks_cache_uc_opt
            .clone()
            .unwrap()
            .find(id.clone().unwrap())
            .await
    } else {
        None
    };

    // If cache hit, return early
    if let Some(res) = cache_result {
        let talk_res = AdminGetTalkTemplate {
            talk: res.sanitize_talk_media_org().to_admin_template(),
        }
        .render();
        if talk_res.is_err() {
            error!(
                "Failed to render admin/talks/get_talk.html. {}",
                talk_res.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("AdminGetTalk askama template rendered.");
        return Html(talk_res.unwrap());
    }

    // If not, get data from database
    let db_result = talks_db_uc
        .talk_display_repo
        .find(id.clone().unwrap())
        .await;

    // Early check db result. If empty, return 404 error
    if db_result.is_none() {
        info!("Failed to find Talk with Id {}.", &id.unwrap());
        return get_404_not_found().await;
    }

    // Insert cache
    if is_cache_enabled {
        debug!("Caching talk {}", &id.unwrap());
        let _ = talks_cache_uc_opt
            .clone()
            .unwrap()
            .talk_operation_repo
            .insert(db_result.clone().unwrap().sanitize_talk_media_org())
            .await;
    }

    // Render Talks
    let talk = AdminGetTalkTemplate {
        talk: db_result
            .clone()
            .unwrap()
            .sanitize_talk_media_org()
            .to_admin_template(),
    }
    .render();
    if talk.is_err() {
        error!(
            "Failed to render admin/talks/get_talk.html. {}",
            talk.unwrap_err()
        );
        return get_500_internal_server_error();
    }
    info!("AdminGetTalk askama template rendered.");
    Html(talk.unwrap())
}

/// get_add_admin_talk
/// Serve GET add talk HTML file in a form format.
#[debug_handler]
pub async fn get_add_admin_talk(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let talks_db_uc = app_state.talk_db_usecase.lock().await.clone().unwrap();
    // Calculate new Talk Id
    let result = talks_db_uc.talk_operation_repo.get_new_id().await;

    if result.is_none() {
        info!("Failed to add Talk.");
        return get_404_not_found().await;
    }

    let talk_id = result.unwrap();
    debug!("Construct AdminGetAddTalkTemplate for Talk Id {}", &talk_id);
    let add_talk = AdminGetAddTalkTemplate {
        id: talk_id,
        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
    }
    .render();

    if add_talk.is_err() {
        error!(
            "Failed to render admin/talks/get_add_talk.html. {}",
            add_talk.unwrap_err()
        );
        return get_500_internal_server_error();
    }

    info!("AdminGetAddTalk askama template rendered.");
    Html(add_talk.unwrap())
}

/// get_edit_admin_talk
/// Serve GET edit talk HTML file to edit a talk
#[debug_handler]
pub async fn get_edit_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let talks_db_uc = app_state.talk_db_usecase.lock().await.clone().unwrap();
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

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        talks_cache_uc_opt
            .clone()
            .unwrap()
            .find(id.clone().unwrap())
            .await
    } else {
        None
    };

    // If cache hit, return early
    if let Some(res) = cache_result {
        let edit_talk = AdminGetEditTalkTemplate {
            talk: res.sanitize_talk_media_org().to_admin_template(),
        }
        .render();
        if edit_talk.is_err() {
            error!(
                "Failed to render admin/talks/get_edit_talk.html. {}",
                edit_talk.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("AdminGetEditTalk askama template rendered.");
        return Html(edit_talk.unwrap());
    }

    // If not, get data from database
    let db_result = talks_db_uc
        .talk_display_repo
        .find(id.clone().unwrap())
        .await;

    // Early check db result. If empty, return 404 error
    if db_result.is_none() {
        info!("Failed to find Talk with Id {}.", &id.unwrap());
        return get_404_not_found().await;
    }

    // Insert cache
    if is_cache_enabled {
        debug!("Caching talk {}", &id.unwrap());
        let _ = talks_cache_uc_opt
            .clone()
            .unwrap()
            .talk_operation_repo
            .insert(db_result.clone().unwrap().sanitize_talk_media_org())
            .await;
    }

    // Render Talks
    let edit_talk = AdminGetEditTalkTemplate {
        talk: db_result
            .clone()
            .unwrap()
            .sanitize_talk_media_org()
            .to_admin_template(),
    }
    .render();
    if edit_talk.is_err() {
        error!(
            "Failed to render admin/talks/get_edit_talk.html. {}",
            edit_talk.unwrap_err()
        );
        return get_500_internal_server_error();
    }
    info!("AdminGetEditTalk askama template rendered.");
    Html(edit_talk.unwrap())
}

/// get_delete_admin_talk
/// Serve GET delete talk HTML file to delete a talk
#[debug_handler]
pub async fn get_delete_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let talks_db_uc = app_state.talk_db_usecase.lock().await.clone().unwrap();
    let talks_cache_uc_opt = app_state.talk_cache_usecase.lock().await;
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

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        talks_cache_uc_opt
            .clone()
            .unwrap()
            .find(id.clone().unwrap())
            .await
    } else {
        None
    };

    // If cache hit, return early
    if let Some(res) = cache_result {
        let delete_talk = AdminGetDeleteTalkTemplate {
            id: res.sanitize_talk_media_org().id,
        }
        .render();
        if delete_talk.is_err() {
            error!(
                "Failed to render admin/talks/get_delete_talk.html. {}",
                delete_talk.unwrap_err()
            );
            return get_500_internal_server_error();
        }
        info!("AdminGetDeleteTalk askama template rendered.");
        return Html(delete_talk.unwrap());
    }

    // If not, get data from database
    let db_result = talks_db_uc
        .talk_display_repo
        .find(id.clone().unwrap())
        .await;

    // Early check db result. If empty, return 404 error
    if db_result.is_none() {
        info!("Failed to find Talk with Id {}.", &id.unwrap());
        return get_404_not_found().await;
    }

    // Insert cache
    if is_cache_enabled {
        debug!("Caching talk {}", &id.unwrap());
        let _ = talks_cache_uc_opt
            .clone()
            .unwrap()
            .talk_operation_repo
            .insert(db_result.clone().unwrap().sanitize_talk_media_org())
            .await;
    }

    // Render Talks
    let delete_talk = AdminGetDeleteTalkTemplate {
        id: db_result.clone().unwrap().id,
    }
    .render();
    if delete_talk.is_err() {
        error!(
            "Failed to render admin/talks/get_delete_talk.html. {}",
            delete_talk.unwrap_err()
        );
        return get_500_internal_server_error();
    }
    info!("AdminGetDeleteTalk askama template rendered.");
    Html(delete_talk.unwrap())
}
