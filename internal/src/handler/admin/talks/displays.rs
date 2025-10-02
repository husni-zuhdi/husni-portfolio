use crate::handler::admin::talks::{sanitize_params, sanitize_talk_media_org};
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::talks::TalksParams;
use crate::model::{
    axum::AppState,
    templates_admin::{
        AdminGetAddTalkTemplate, AdminGetDeleteTalkTemplate, AdminGetEditTalkTemplate,
        AdminGetTalkTemplate, AdminListTalksTemplate, AdminTalkTemplate, AdminTalksTemplate,
    },
};
use crate::repo::talks::TalkRepo;
use askama::Template;
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::response::Html;
use tracing::{debug, error, info, warn};

/// get_base_admin_talks
/// Serve GET (base) admin talks HTML file
/// Under endpoint /admin/talks
/// It's the base of Admin Talks feature
#[debug_handler]
pub async fn get_base_admin_talks() -> Html<String> {
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
    params: Query<TalksParams>,
) -> Html<String> {
    let talks_uc = app_state.talk_usecase.lock().await.clone().unwrap();
    debug!("Params {:?}", &params);
    let sanitized_params = sanitize_params(params);

    // Construct TalksTemplate Struct
    let result = talks_uc
        .talk_repo
        .find_talks(sanitized_params.clone())
        .await;
    match result {
        Some(talks_data) => {
            let talks: Vec<AdminTalkTemplate> = talks_data
                .talks
                .iter()
                .map(|talk| {
                    debug!("Construct AdminTalkTemplate for Talk Id {}", &talk.id);
                    debug!("AdminTalkTemplate {:?}", &talk);
                    let (media_link, org_name, org_link) = sanitize_talk_media_org(talk);

                    AdminTalkTemplate {
                        id: talk.id,
                        name: talk.name.clone(),
                        date: talk.date.clone(),
                        media_link,
                        org_name,
                        org_link,
                    }
                })
                .collect();
            debug!("AdminTalksTemplate talks : {:?}", &talks);

            let talks_res = AdminListTalksTemplate { talks }.render();
            match talks_res {
                Ok(res) => {
                    info!("Talks askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/talks/list_talks.html. {}", err);
                    get_500_internal_server_error()
                }
            }
        }
        None => {
            error!(
                "Failed to find talks with Talk Id started at {} and ended at {}.",
                &sanitized_params.start.unwrap(),
                &sanitized_params.end.unwrap()
            );
            get_500_internal_server_error()
        }
    }
}

/// get_admin_talk
/// Serve GET talk HTML file and return point for several cancelation endpoints
/// Returned single talk
#[debug_handler]
pub async fn get_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
) -> Html<String> {
    let talks_uc = app_state.talk_usecase.lock().await.clone().unwrap();
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

    let result = talks_uc.talk_repo.find(id.clone().unwrap()).await;

    match result {
        Some(talk_data) => {
            debug!("Construct AdminTalkTemplate for Talk Id {}", &talk_data.id);
            debug!("AdminTalkTemplate {:?}", &talk_data);
            let (media_link, org_name, org_link) = sanitize_talk_media_org(&talk_data);

            let talk = AdminGetTalkTemplate {
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
            debug!("AdminGetTalkTemplate : {:?}", &talk);

            match talk {
                Ok(res) => {
                    info!("Talks askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/talks/get_talk.html. {}", err);
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

/// get_add_admin_talk
/// Serve GET add talk HTML file in a form format.
#[debug_handler]
pub async fn get_add_admin_talk(State(app_state): State<AppState>) -> Html<String> {
    let talks_uc = app_state.talk_usecase.lock().await.clone().unwrap();
    // Calculate new Talk Id
    let result = talks_uc.get_new_id().await;

    match result {
        Some(talk_id) => {
            debug!("Construct AdminGetAddTalkTemplate for Talk Id {}", &talk_id);
            let add_talk = AdminGetAddTalkTemplate {
                id: talk_id,
                date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            }
            .render();
            debug!("AdminGetAddTalkTemplate : {:?}", &add_talk);

            match add_talk {
                Ok(res) => {
                    info!("Talks askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/talks/get_add_talk.html. {}", err);
                    get_500_internal_server_error()
                }
            }
        }
        None => {
            info!("Failed to add Talk.");
            get_404_not_found().await
        }
    }
}

/// get_edit_admin_talk
/// Serve GET edit talk HTML file to edit a talk
#[debug_handler]
pub async fn get_edit_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
) -> Html<String> {
    let talks_uc = app_state.talk_usecase.lock().await.clone().unwrap();
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

    let result = talks_uc.talk_repo.find(id.clone().unwrap()).await;

    match result {
        Some(talk_data) => {
            debug!(
                "Construct AdminGetEditTalkTemplate for Talk Id {}",
                &talk_data.id
            );
            debug!("Talk {:?}", &talk_data);
            let (media_link, org_name, org_link) = sanitize_talk_media_org(&talk_data);

            let edit_talk = AdminGetEditTalkTemplate {
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
            debug!("AdminGetEditTalkTemplate : {:?}", &edit_talk);

            match edit_talk {
                Ok(res) => {
                    info!("Talks askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/talks/get_edit_talk.html. {}", err);
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

/// get_delete_admin_talk
/// Serve GET delete talk HTML file to delete a talk
#[debug_handler]
pub async fn get_delete_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
) -> Html<String> {
    let talks_uc = app_state.talk_usecase.lock().await.clone().unwrap();
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

    let result = talks_uc.talk_repo.find(id.clone().unwrap()).await;

    match result {
        Some(talk_data) => {
            debug!(
                "Construct AdminGetDeleteTalkTemplate for Talk Id {}",
                &talk_data.id
            );
            debug!("Talk {:?}", &talk_data);

            let delete_talk = AdminGetDeleteTalkTemplate { id: id.unwrap() }.render();
            debug!("AdminGetDeleteTalkTemplate : {:?}", &delete_talk);

            match delete_talk {
                Ok(res) => {
                    info!("Talks askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/talks/get_delete_talk.html. {}", err);
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
