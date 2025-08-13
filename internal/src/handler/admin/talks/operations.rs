use crate::handler::admin::talks::sanitize_talk_media_org;
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::axum::AppState;
use crate::model::talks::{Talk, TalkCommandStatus, TalkEndPage, TalkId, TalkStartPage};
use crate::model::templates_admin::{
    AdminGetTalkTemplate, AdminGetTalksTemplate, AdminTalkTemplate,
};
use askama::Template;
use axum::debug_handler;
use axum::extract::{Path, State};
use axum::response::Html;
use tracing::{debug, error, info, warn};
use urlencoding::decode;

// Take request body String from PUT and POST operations to create a new Talk
fn process_talk_body(body: String) -> Talk {
    // Initialize fields
    let mut talk_id = 0_i64;
    let mut talk_name = String::new();
    let mut talk_media_link = String::new();
    let mut talk_date = String::new();
    let mut talk_org_name = String::new();
    let mut talk_org_link = String::new();

    let req_fields: Vec<&str> = body.split("&").collect();
    for req_field in req_fields {
        let (key, value) = req_field.split_once("=").unwrap();
        let value_decoded = decode(value).unwrap();
        debug!("Request field key/value {:?}/{:?}", key, value_decoded);
        match key {
            "talk_id" => {
                talk_id = value_decoded
                    .parse::<i64>()
                    .expect("Failed to parse path from request body")
            }
            "talk_name" => talk_name = value_decoded.to_string(),
            "talk_media_link" => talk_media_link = value_decoded.to_string(),
            "talk_date" => talk_date = value_decoded.to_string(),
            "talk_org_name" => talk_org_name = value_decoded.to_string(),
            "talk_org_link" => talk_org_link = value_decoded.to_string(),
            _ => {
                warn!("Unrecognized key/value: {:?}/{:?}", key, value_decoded);
                continue;
            }
        }
    }

    Talk {
        id: TalkId { id: talk_id },
        name: talk_name,
        date: talk_date,
        media_link: Some(talk_media_link),
        org_name: Some(talk_org_name),
        org_link: Some(talk_org_link),
    }
}

/// post_add_admin_talk
/// Serve POST add talk endpoint
#[debug_handler]
pub async fn post_add_admin_talk(State(app_state): State<AppState>, body: String) -> Html<String> {
    match app_state.talk_usecase.lock().await.clone() {
        Some(mut data) => {
            let talk = process_talk_body(body);

            let add_result = data
                .talk_repo
                .add(
                    talk.id.clone(),
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

            // Construct TalksTemplate Struct
            let find_talks_result = data
                .talk_repo
                .find_talks(TalkStartPage(0), TalkEndPage(10))
                .await;

            match find_talks_result {
                Some(talks_data) => {
                    let talks: Vec<AdminTalkTemplate> = talks_data
                        .iter()
                        .map(|talk| {
                            debug!("Construct AdminTalkTemplate for Talk Id {}", &talk.id);
                            debug!("AdminTalkTemplate {:?}", &talk);
                            let (media_link, org_name, org_link) = sanitize_talk_media_org(talk);

                            AdminTalkTemplate {
                                id: talk.id.id,
                                name: talk.name.clone(),
                                date: talk.date.clone(),
                                media_link,
                                org_name,
                                org_link,
                            }
                        })
                        .collect();
                    debug!("AdminGetTalksTemplate talks : {:?}", &talks);

                    let talks_res = AdminGetTalksTemplate { talks }.render();
                    match talks_res {
                        Ok(res) => {
                            info!("Talks askama template rendered.");
                            Html(res)
                        }
                        Err(err) => {
                            error!("Failed to render admin/get_talks.html for post_add_admin_talk operation. {}", err);
                            get_500_internal_server_error()
                        }
                    }
                }
                None => {
                    error!("Failed to find talks with Talk Id started at 0 and ended at 10.",);
                    get_500_internal_server_error()
                }
            }
        }
        None => get_404_not_found().await,
    }
}

/// put_edit_admin_talk
/// Serve PUT edit talk HTML file
#[debug_handler]
pub async fn put_edit_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    body: String,
) -> Html<String> {
    match app_state.talk_usecase.lock().await.clone() {
        Some(mut data) => {
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

            let result = data
                .talk_repo
                .update(
                    talk.id.clone(),
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

            let get_result = data.talk_repo.find(talk.id.clone()).await;

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
        None => get_404_not_found().await,
    }
}

/// delete_delete_admin_talk
/// Serve DELETE delete talk HTML file
#[debug_handler]
pub async fn delete_delete_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
) -> Html<String> {
    match app_state.talk_usecase.lock().await.clone() {
        Some(mut data) => {
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

            let delete_result = data
                .talk_repo
                .delete(TalkId {
                    id: id.clone().unwrap(),
                })
                .await;

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

            // Construct TalksTemplate Struct
            let find_talks_result = data
                .talk_repo
                .find_talks(TalkStartPage(0), TalkEndPage(10))
                .await;

            match find_talks_result {
                Some(talks_data) => {
                    let talks: Vec<AdminTalkTemplate> = talks_data
                        .iter()
                        .map(|talk| {
                            debug!("Construct AdminTalkTemplate for Talk Id {}", &talk.id);
                            debug!("AdminTalkTemplate {:?}", &talk);
                            let (media_link, org_name, org_link) = sanitize_talk_media_org(talk);

                            AdminTalkTemplate {
                                id: talk.id.id,
                                name: talk.name.clone(),
                                date: talk.date.clone(),
                                media_link,
                                org_name,
                                org_link,
                            }
                        })
                        .collect();
                    debug!("AdminGetTalksTemplate talks : {:?}", &talks);

                    let talks_res = AdminGetTalksTemplate { talks }.render();
                    match talks_res {
                        Ok(res) => {
                            info!("Talks askama template rendered.");
                            Html(res)
                        }
                        Err(err) => {
                            error!("Failed to render admin/get_talks.html for delete_delete_admin_talk operation. {}", err);
                            get_500_internal_server_error()
                        }
                    }
                }
                None => {
                    error!("Failed to find talks with Talk Id started at 0 and ended at 10.",);
                    get_500_internal_server_error()
                }
            }
        }
        None => get_404_not_found().await,
    }
}
