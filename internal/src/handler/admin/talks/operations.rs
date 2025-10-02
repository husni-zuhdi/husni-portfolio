use crate::handler::admin::talks::displays::get_admin_talks_list;
use crate::handler::admin::talks::{process_talk_body, sanitize_talk_media_org};
use crate::handler::auth::{process_login_header, verify_jwt};
use crate::handler::status::get_401_unauthorized;
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::axum::AppState;
use crate::model::talks::{TalkCommandStatus, TalksParams};
use crate::model::templates_admin::{AdminGetTalkTemplate, AdminTalkTemplate};
use askama::Template;
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

    if !verify_jwt(&token, &app_state.config.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let mut talks_uc = app_state.talk_usecase.lock().await.clone().unwrap();
    let talk = process_talk_body(body);

    let add_result = talks_uc
        .talk_repo
        .add(
            talk.id,
            talk.name,
            talk.date,
            talk.media_link,
            talk.org_name,
            talk.org_link,
        )
        .await;

    match add_result {
        Some(talk_command_status) => {
            if talk_command_status != TalkCommandStatus::Stored {
                error!("Failed to add Talk with Id {}", &talk.id);
                return get_500_internal_server_error();
            }
        }
        None => {
            info!("Failed to add Talk with Id {}.", &talk.id);
            return get_404_not_found().await;
        }
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
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let mut talks_uc = app_state.talk_usecase.lock().await.clone().unwrap();
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

    let talk = process_talk_body(body);

    let result = talks_uc
        .talk_repo
        .update(
            talk.id,
            Some(talk.name.clone()),
            Some(talk.date.clone()),
            talk.media_link.clone(),
            talk.org_name.clone(),
            talk.org_link.clone(),
        )
        .await;

    match result {
        Some(talk_command_status) => {
            if talk_command_status != TalkCommandStatus::Updated {
                error!("Failed to edit Talk with Id {}", &path);
                return get_500_internal_server_error();
            }
        }
        None => {
            info!("Failed to edit Talk with Id {}.", &path);
            return get_404_not_found().await;
        }
    }

    let get_result = talks_uc.talk_repo.find(talk.id).await;

    match get_result {
        Some(talk_data) => {
            debug!("Construct AdminTalkTemplate for Talk Id {}", &talk_data.id);
            debug!("AdminTalkTemplate {:?}", &talk_data);
            let (media_link, org_name, org_link) = sanitize_talk_media_org(&talk);

            let edit_talk = AdminGetTalkTemplate {
                talk: AdminTalkTemplate {
                    id: id.clone().unwrap(),
                    name: talk_data.name.clone(),
                    date: talk_data.date.clone(),
                    media_link,
                    org_name,
                    org_link,
                },
            }
            .render();
            debug!("AdminGetTalkTemplate : {:?}", &edit_talk);

            match edit_talk {
                Ok(res) => {
                    info!("Talks askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/get_talk.html. {}", err);
                    get_500_internal_server_error()
                }
            }
        }
        None => {
            info!("Failed to find Talk with Id {}.", &path);
            get_404_not_found().await
        }
    }
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

    if !verify_jwt(&token, &app_state.config.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let mut talks_uc = app_state.talk_usecase.lock().await.clone().unwrap();
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

    let delete_result = talks_uc.talk_repo.delete(id.clone().unwrap()).await;

    match delete_result {
        Some(talk_command_status) => {
            if talk_command_status != TalkCommandStatus::Deleted {
                error!("Failed to delete Talk with Id {}", &path);
                return get_500_internal_server_error();
            }
        }
        None => {
            info!("Failed to edit Talk with Id {}.", &path);
            return get_404_not_found().await;
        }
    }
    let params = TalksParams {
        start: None,
        end: None,
    };
    get_admin_talks_list(State(app_state), headers, Query(params)).await
}
