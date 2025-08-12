use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::talks::{Talk, TalkEndPage, TalkId, TalkPagination, TalkStartPage};
use crate::model::{
    axum::AppState,
    templates_admin::{
        AdminGetAddTalkTemplate, AdminGetDeleteTalkTemplate, AdminGetEditTalkTemplate,
        AdminGetTalkTemplate, AdminGetTalksTemplate, AdminTalkTemplate, AdminTalksTemplate,
    },
};
use crate::port::talks::query::TalkQueryPort;
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
pub async fn get_base_admin_talks(
    State(app_state): State<AppState>,
    pagination: Query<TalkPagination>,
) -> Html<String> {
    match app_state.talk_usecase.lock().await.clone() {
        Some(data) => {
            // Setup Pagination
            debug!("Pagination {:?}", &pagination);
            let (start, end) = setup_pagination(pagination);

            // Construct TalksTemplate Struct
            let result = data.talk_repo.find_talks(start.clone(), end.clone()).await;
            match result {
                Some(talks_data) => {
                    let talks: Vec<AdminTalkTemplate> = talks_data
                        .iter()
                        .map(|talk| {
                            debug!("Construct AdminTalkTemplate for Talk Id {}", &talk.id);
                            debug!("AdminTalkTemplate {:?}", &talk);
                            let (media_link, org_name, org_link) = sanitize_talk_media_org(&talk);

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
                    debug!("AdminTalksTemplate talks : {:?}", &talks);

                    let talks_res = AdminTalksTemplate { talks }.render();
                    match talks_res {
                        Ok(res) => {
                            info!("Talks askama template rendered.");
                            Html(res)
                        }
                        Err(err) => {
                            error!("Failed to render admin/talks.html. {}", err);
                            get_500_internal_server_error()
                        }
                    }
                }
                None => {
                    error!(
                        "Failed to find talks with Talk Id started at {} and ended at {}.",
                        &start.0, &end.0
                    );
                    get_500_internal_server_error()
                }
            }
        }
        None => get_404_not_found().await,
    }
}

/// get_admin_talks
/// Serve get_talks HTML file and return point for /admin/talks/add cancel button
/// Return lite version of get_base_admin_talks with talks data only
#[debug_handler]
pub async fn get_admin_talks(
    State(app_state): State<AppState>,
    pagination: Query<TalkPagination>,
) -> Html<String> {
    match app_state.talk_usecase.lock().await.clone() {
        Some(data) => {
            debug!("Pagination {:?}", &pagination);
            let (start, end) = setup_pagination(pagination);

            // Construct TalksTemplate Struct
            let result = data.talk_repo.find_talks(start.clone(), end.clone()).await;
            match result {
                Some(talks_data) => {
                    let talks: Vec<AdminTalkTemplate> = talks_data
                        .iter()
                        .map(|talk| {
                            debug!("Construct AdminTalkTemplate for Talk Id {}", &talk.id);
                            debug!("AdminTalkTemplate {:?}", &talk);
                            let (media_link, org_name, org_link) = sanitize_talk_media_org(&talk);

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
                    debug!("AdminTalksTemplate talks : {:?}", &talks);

                    let talks_res = AdminGetTalksTemplate { talks }.render();
                    match talks_res {
                        Ok(res) => {
                            info!("Talks askama template rendered.");
                            Html(res)
                        }
                        Err(err) => {
                            error!("Failed to render admin/get_talks.html. {}", err);
                            get_500_internal_server_error()
                        }
                    }
                }
                None => {
                    error!(
                        "Failed to find talks with Talk Id started at {} and ended at {}.",
                        &start.0, &end.0
                    );
                    get_500_internal_server_error()
                }
            }
        }
        None => get_404_not_found().await,
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
    match app_state.talk_usecase.lock().await.clone() {
        Some(data) => {
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

            let result = data
                .talk_repo
                .find(TalkId {
                    id: id.clone().unwrap(),
                })
                .await;

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
                            return Html(res);
                        }
                        Err(err) => {
                            error!("Failed to render admin/get_talk.html. {}", err);
                            return get_500_internal_server_error();
                        }
                    }
                }
                None => {
                    info!("Failed to find Talk with Id {}.", &path);
                    return get_404_not_found().await;
                }
            }
        }
        None => get_404_not_found().await,
    }
}

/// get_add_admin_talk
/// Serve GET add talk HTML file in a form format.
#[debug_handler]
pub async fn get_add_admin_talk(State(app_state): State<AppState>) -> Html<String> {
    match app_state.talk_usecase.lock().await.clone() {
        Some(data) => {
            // Calculate new Talk Id
            let result = data.get_new_id().await;

            match result {
                Some(talk_id) => {
                    debug!(
                        "Construct AdminGetAddTalkTemplate for Talk Id {}",
                        &talk_id.id
                    );
                    let add_talk = AdminGetAddTalkTemplate {
                        id: talk_id.id,
                        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
                    }
                    .render();
                    debug!("AdminGetAddTalkTemplate : {:?}", &add_talk);

                    match add_talk {
                        Ok(res) => {
                            info!("Talks askama template rendered.");
                            return Html(res);
                        }
                        Err(err) => {
                            error!("Failed to render admin/get_add_talk.html. {}", err);
                            return get_500_internal_server_error();
                        }
                    }
                }
                None => {
                    info!("Failed to add Talk.");
                    return get_404_not_found().await;
                }
            }
        }
        None => get_404_not_found().await,
    }
}

/// get_edit_admin_talk
/// Serve GET edit talk HTML file to edit a talk
#[debug_handler]
pub async fn get_edit_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
) -> Html<String> {
    match app_state.talk_usecase.lock().await.clone() {
        Some(data) => {
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

            let result = data
                .talk_repo
                .find(TalkId {
                    id: id.clone().unwrap(),
                })
                .await;

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
                            return Html(res);
                        }
                        Err(err) => {
                            error!("Failed to render admin/get_edit_talk.html. {}", err);
                            return get_500_internal_server_error();
                        }
                    }
                }
                None => {
                    info!("Failed to find Talk with Id {}.", &path);
                    return get_404_not_found().await;
                }
            }
        }
        None => get_404_not_found().await,
    }
}

/// get_delete_admin_talk
/// Serve GET delete talk HTML file to delete a talk
#[debug_handler]
pub async fn get_delete_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
) -> Html<String> {
    match app_state.talk_usecase.lock().await.clone() {
        Some(data) => {
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

            let result = data
                .talk_repo
                .find(TalkId {
                    id: id.clone().unwrap(),
                })
                .await;

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
                            return Html(res);
                        }
                        Err(err) => {
                            error!("Failed to render admin/get_delete_talk.html. {}", err);
                            return get_500_internal_server_error();
                        }
                    }
                }
                None => {
                    info!("Failed to find Talk with Id {}.", &path);
                    return get_404_not_found().await;
                }
            }
        }
        None => get_404_not_found().await,
    }
}

/// Initiate pagination check. Set default if not manually requested
fn setup_pagination(pagination: Query<TalkPagination>) -> (TalkStartPage, TalkEndPage) {
    let start = match pagination.0.start {
        Some(val) => val,
        None => {
            debug!("Set default start to 0");
            TalkStartPage(0)
        }
    };
    let end = match pagination.0.end {
        Some(val) => val,
        None => {
            debug!("Set default end to 10");
            TalkEndPage(10)
        }
    };
    (start, end)
}

/// sanitize media and org part of Talk fields
fn sanitize_talk_media_org(talk_data: &Talk) -> (String, String, String) {
    let empty_value = "".to_string();

    let media_link = match &talk_data.media_link {
        Some(val) => val.clone(),
        None => empty_value.clone(),
    };
    let org_name = match &talk_data.org_name {
        Some(val) => val.clone(),
        None => empty_value.clone(),
    };
    let org_link = match &talk_data.org_link {
        Some(val) => val.clone(),
        None => empty_value.clone(),
    };

    (media_link, org_name, org_link)
}
