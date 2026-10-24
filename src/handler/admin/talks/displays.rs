use crate::handler::auth::is_auth_verified;
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
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
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
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
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
        let talks_res = match res.sanitize().to_admin_list_template().render() {
            Ok(html) => html,
            Err(err) => {
                error!("Failed to render get_talks.html. {}", err);
                return get_500_internal_server_error();
            }
        };

        info!("Talks askama template rendered.");
        return Html(talks_res);
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
    let talks_res = match db_result
        .unwrap()
        .sanitize()
        .to_admin_list_template()
        .render()
    {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/talks/list_talks.html. {}", err);
            return get_500_internal_server_error();
        }
    };

    info!("AdminListTalks askama template rendered.");
    Html(talks_res)
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
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
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
        let talk_tmpl = AdminGetTalkTemplate {
            talk: res.sanitize_talk_media_org().to_admin_template(),
        };
        let talk_res = match talk_tmpl.render() {
            Ok(html) => html,
            Err(err) => {
                error!("Failed to render admin/talks/get_talk.html. {}", err);
                return get_500_internal_server_error();
            }
        };

        info!("AdminGetTalk askama template rendered.");
        return Html(talk_res);
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
        drop(talks_cache_uc_opt);
    }

    // Render Talks
    let talk_tmpl = AdminGetTalkTemplate {
        talk: db_result
            .clone()
            .unwrap()
            .sanitize_talk_media_org()
            .to_admin_template(),
    };
    let talk = match talk_tmpl.render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/talks/get_talk.html. {}", err);
            return get_500_internal_server_error();
        }
    };
    info!("AdminGetTalk askama template rendered.");
    Html(talk)
}

/// get_add_admin_talk
/// Serve GET add talk HTML file in a form format.
#[debug_handler]
pub async fn get_add_admin_talk(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
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
    let add_talk_tmpl = AdminGetAddTalkTemplate {
        id: talk_id,
        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
    };
    let add_talk = match add_talk_tmpl.render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/talks/get_add_talk.html. {}", err);
            return get_500_internal_server_error();
        }
    };

    info!("AdminGetAddTalk askama template rendered.");
    Html(add_talk)
}

/// get_edit_admin_talk
/// Serve GET edit talk HTML file to edit a talk
#[debug_handler]
pub async fn get_edit_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
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
        let edit_talk_tmpl = AdminGetEditTalkTemplate {
            talk: res.sanitize_talk_media_org().to_admin_template(),
        };
        let edit_talk = match edit_talk_tmpl.render() {
            Ok(html) => html,
            Err(err) => {
                error!("Failed to render admin/talks/get_edit_talk.html. {}", err);
                return get_500_internal_server_error();
            }
        };

        info!("AdminGetEditTalk askama template rendered.");
        return Html(edit_talk);
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
    let edit_talk_tmpl = AdminGetEditTalkTemplate {
        talk: db_result
            .clone()
            .unwrap()
            .sanitize_talk_media_org()
            .to_admin_template(),
    };
    let edit_talk = match edit_talk_tmpl.render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/talks/get_edit_talk.html. {}", err);
            return get_500_internal_server_error();
        }
    };
    info!("AdminGetEditTalk askama template rendered.");
    Html(edit_talk)
}

/// get_delete_admin_talk
/// Serve GET delete talk HTML file to delete a talk
#[debug_handler]
pub async fn get_delete_admin_talk(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
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
        let delete_talk_tmpl = AdminGetDeleteTalkTemplate {
            id: res.sanitize_talk_media_org().id,
        };
        let delete_talk = match delete_talk_tmpl.render() {
            Ok(html) => html,
            Err(err) => {
                error!("Failed to render admin/talks/get_delete_talk.html. {}", err);
                return get_500_internal_server_error();
            }
        };
        info!("AdminGetDeleteTalk askama template rendered.");
        return Html(delete_talk);
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
        drop(talks_cache_uc_opt);
    }

    // Render Talks
    let delete_talk_tmpl = AdminGetDeleteTalkTemplate {
        id: db_result.clone().unwrap().id,
    };
    let delete_talk = match delete_talk_tmpl.render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/talks/get_delete_talk.html. {}", err);
            return get_500_internal_server_error();
        }
    };
    info!("AdminGetDeleteTalk askama template rendered.");
    Html(delete_talk)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::talks::Talks;
    use crate::model::templates_admin::AdminListTalksTemplate;

    fn sample_talk() -> crate::model::talks::Talk {
        crate::model::talks::Talk {
            id: 1,
            name: "Talk 1".to_string(),
            date: "2024-01-01".to_string(),
            media_link: Some("https://example.com/talk-1".to_string()),
            org_name: Some("Org 1".to_string()),
            org_link: Some("https://example.com/org-1".to_string()),
        }
    }

    /// Regression test: the standalone talk view (returned after an edit is
    /// accepted or canceled) must render the same table row markup as the
    /// talks list so the swapped-in fragment looks identical to the
    /// original list row.
    #[test]
    fn test_get_talk_row_matches_list_row_markup() {
        let talk = sample_talk();

        // Standalone row fragment (get_talk.html)
        let row = AdminGetTalkTemplate {
            talk: talk.clone().sanitize_talk_media_org().to_admin_template(),
        }
        .render()
        .unwrap();

        // The list renders the same talk via the shared partial
        let list = AdminListTalksTemplate {
            talks: Talks { talks: vec![talk] }.to_admin_list_template().talks,
        }
        .render()
        .unwrap();

        for html in [&row, &list] {
            assert!(html.contains("<tr id=\"talk_1_target\">"));
            assert!(html.contains("hx-get=\"/admin/talks/1/edit\""));
            assert!(html.contains("hx-swap=\"outerHTML\""));
            assert!(html.contains("hx-get=\"/admin/talks/1/delete\""));
        }

        // The list renders a table with headers; the row view must not
        // duplicate the table element itself
        assert!(list.contains("<table"));
        assert!(list.contains("<th>Date</th>"));
        assert!(!row.contains("<table"));
    }
}
