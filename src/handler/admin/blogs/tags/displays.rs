use crate::handler::auth::is_auth_verified;
use crate::handler::status::{
    get_401_unauthorized, get_404_not_found, get_500_internal_server_error,
};
use crate::model::axum::AppState;
use crate::model::tags::{TagsListParams, TagsSearchParams};
use crate::model::templates_admin::{
    AdminBlogTagsTemplate, AdminGetAddTagTemplate, AdminGetDeleteTagTemplate,
    AdminGetEditTagTemplate,
};
use askama::Template;
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use tracing::{debug, error, info, warn};

/// get_base_admin_tags
/// Serve GET (base) admin tags HTML file
/// Under endpoint /admin/blogs/tags
#[debug_handler]
pub async fn get_base_admin_tags(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    let tags_res = AdminBlogTagsTemplate {}.render();
    match tags_res {
        Ok(res) => {
            info!("Admin Blogs askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render admin/blogs/tags/tags.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_admin_tags_list
/// Serve to list tags for Admin Blog
/// Under endpoint /admin/blogs/tags/list
/// Accepted parameters:
/// - start: initial tags pagination
/// - end: end of tags pagination
#[debug_handler]
pub async fn get_admin_tags_list(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    params: Query<TagsListParams>,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    let mut tag_uc = app_state.tag_usecase.lock().await.clone();
    let sanitized_params = params.sanitize();
    let tags_result = tag_uc.find_tags(sanitized_params.clone()).await;

    let Some(tags_data) = tags_result else {
        error!(
            "Failed to find admin blog tags with Tag Id started at {} and ended at {}.",
            sanitized_params.start.unwrap(),
            sanitized_params.end.unwrap()
        );
        return get_500_internal_server_error();
    };

    let tags_res = match tags_data.to_admin_list_template().render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/blogs/tags/list_tags.html. {}", err);
            return get_500_internal_server_error();
        }
    };

    info!("AdminBlogTagsList askama template rendered.");
    Html(tags_res)
}

/// get_admin_tags_search
/// Serve to search tags for Admin Blog
/// Under endpoint /admin/blogs/tags/search
/// Accepted parameters:
/// - start: initial tags pagination
/// - end: end of tags pagination
/// - query: string of search query
#[debug_handler]
pub async fn get_admin_tags_search(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    params: Query<TagsSearchParams>,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    let mut tag_uc = app_state.tag_usecase.lock().await.clone();
    let sanitized_params = params.sanitize();
    let tags_result = tag_uc.search_tags(sanitized_params.clone()).await;

    let Some(tags_data) = tags_result else {
        error!(
            "Failed to find admin blog tags with Tag Id started at {}, ended at {}, and query {}.",
            sanitized_params.start.unwrap(),
            sanitized_params.end.unwrap(),
            sanitized_params.query
        );
        return get_500_internal_server_error();
    };

    let tags_res = match tags_data.to_admin_list_template().render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/blogs/tags/list_tags.html. {}", err);
            return get_500_internal_server_error();
        }
    };

    info!("AdminBlogTagsList askama template rendered.");
    Html(tags_res)
}

/// get_admin_tag
/// Serve GET tag HTML file and return point for several cancelation endpoints
/// Returned single tag
#[debug_handler]
pub async fn get_admin_tag(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    let mut tag_uc = app_state.tag_usecase.lock().await.clone();
    let tag_result = tag_uc.find(id).await;

    // Early check result. If empty, return 404 error
    let Some(tag_data) = tag_result else {
        info!("Failed to find Tag with Id {}.", &id);
        return get_404_not_found().await;
    };

    let tag_res = match tag_data.to_admin_template().render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/blogs/tags/get_tag.html. {}", err);
            return get_500_internal_server_error();
        }
    };
    info!("AdminGetTag askama template rendered.");
    Html(tag_res)
}

/// get_add_admin_tag
/// Serve GET add tag HTML file in a form format.
#[debug_handler]
pub async fn get_add_admin_tag(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    // Calculate new Tag Id
    let db_result = {
        let mut tag_uc = app_state.tag_usecase.lock().await.clone();
        tag_uc.get_new_id().await
    };

    let Some(id) = db_result else {
        error!("Failed to get new Tag ID.");
        return get_500_internal_server_error();
    };
    debug!("Construct AdminGetAddTagTemplate for Tag Id {}", &id);

    let add_tag = match (AdminGetAddTagTemplate { id }).render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/blogs/get_add_tag.html. {}", err);
            return get_500_internal_server_error();
        }
    };

    info!("AdminGetAddTag askama template rendered.");
    Html(add_tag)
}

/// get_edit_admin_tag
/// Serve GET edit tag HTML file to edit a tag
#[debug_handler]
pub async fn get_edit_admin_tag(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    let mut tag_uc = app_state.tag_usecase.lock().await.clone();
    let tag_result = tag_uc.find(id).await;

    // Early check result. If empty, return 404 error
    let Some(tag_data) = tag_result else {
        info!("Failed to find Tag with Id {}.", &path);
        return get_404_not_found().await;
    };

    let edit_tag_tmpl = AdminGetEditTagTemplate {
        id: tag_data.id,
        name: tag_data.name,
    };
    let edit_tag = match edit_tag_tmpl.render() {
        Ok(html) => html,
        Err(err) => {
            error!(
                "Failed to render admin/blogs/tags/get_edit_tag.html. {}",
                err
            );
            return get_500_internal_server_error();
        }
    };

    info!("AdminGetEditTag askama template rendered.");
    Html(edit_tag)
}

/// get_delete_admin_tag
/// Serve GET tag blog HTML file to delete a tag
#[debug_handler]
pub async fn get_delete_admin_tag(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    let mut tag_uc = app_state.tag_usecase.lock().await.clone();
    let tag_result = tag_uc.find(id).await;

    // Early check result. If empty, return 404 error
    let Some(tag_data) = tag_result else {
        info!("Failed to find Tag with Id {}.", &path);
        return get_404_not_found().await;
    };

    let delete_tag = match (AdminGetDeleteTagTemplate { id: tag_data.id }).render() {
        Ok(html) => html,
        Err(err) => {
            error!(
                "Failed to render admin/blogs/tags/get_delete_tag.html. {}",
                err
            );
            return get_500_internal_server_error();
        }
    };

    info!("AdminGetDeleteTag askama template rendered.");
    Html(delete_tag)
}
