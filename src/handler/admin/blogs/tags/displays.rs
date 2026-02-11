use crate::handler::auth::{process_login_header, verify_jwt};
use crate::handler::status::get_401_unauthorized;
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
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
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
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
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    // Locking Mutex
    let tags_db_uc = app_state.tag_db_usecase.lock().await.clone().unwrap();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_cache_enabled = tags_cache_uc_opt.is_some();

    let sanitized_params = params.sanitize();

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_display_repo
            .find_tags(sanitized_params.clone())
            .await
    } else {
        None
    };

    // If cache hit, return early
    if let Some(res) = cache_result {
        let tags_res = res.to_admin_list_template().render();
        if tags_res.is_err() {
            error!(
                "Failed to render admin/blogs/tags/list_tags.html. {}",
                tags_res.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("AdminBlogTagsList askama template rendered.");
        return Html(tags_res.unwrap());
    }

    // If not, get data from database
    let db_result = tags_db_uc
        .tag_display_repo
        .find_tags(sanitized_params.clone())
        .await;

    if db_result.is_none() {
        error!(
            "Failed to find admin blog tags with Tag Id started at {} and ended at {}.",
            sanitized_params.start.unwrap(),
            sanitized_params.end.unwrap()
        );
        return get_500_internal_server_error();
    }

    // Insert cache
    if is_cache_enabled {
        for tag in db_result.clone().unwrap().tags {
            debug!("Caching tag {}", &tag.id);
            let _ = tags_cache_uc_opt
                .clone()
                .unwrap()
                .tag_operation_repo
                .insert(tag)
                .await;
        }
    }

    // Render Admin Blog Tags List
    let tags_res = db_result.unwrap().to_admin_list_template().render();
    if tags_res.is_err() {
        error!(
            "Failed to render admin/blogs/tags/list_tags.html. {}",
            tags_res.unwrap_err()
        );
        return get_500_internal_server_error();
    }
    info!("Admin Blogs askama template rendered.");
    Html(tags_res.unwrap())
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
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    // Locking Mutex
    let tags_db_uc = app_state.tag_db_usecase.lock().await.clone().unwrap();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_cache_enabled = tags_cache_uc_opt.is_some();

    let sanitized_params = params.sanitize();

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_display_repo
            .search_tags(sanitized_params.clone())
            .await
    } else {
        None
    };

    // If cache hit, return early
    if let Some(res) = cache_result {
        let tags_res = res.to_admin_list_template().render();
        if tags_res.is_err() {
            error!(
                "Failed to render admin/blogs/tags/list_tags.html. {}",
                tags_res.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("AdminBlogTagsLists askama template rendered.");
        return Html(tags_res.unwrap());
    }

    // If not, get data from database
    let db_result = tags_db_uc
        .tag_display_repo
        .search_tags(sanitized_params.clone())
        .await;

    if db_result.is_none() {
        error!(
            "Failed to find admin blog tags with Tag Id started at {}, ended at {}, and query {}.",
            sanitized_params.start.unwrap(),
            sanitized_params.end.unwrap(),
            sanitized_params.query
        );
        return get_500_internal_server_error();
    }

    // Insert cache
    if is_cache_enabled {
        for tag in db_result.clone().unwrap().tags {
            debug!("Caching tag {}", &tag.id);
            let _ = tags_cache_uc_opt
                .clone()
                .unwrap()
                .tag_operation_repo
                .insert(tag)
                .await;
        }
    }

    // Render Admin Blog Tags List
    let tags_res = db_result.unwrap().to_admin_list_template().render();
    if tags_res.is_err() {
        error!(
            "Failed to render admin/blogs/tags/list_tags.html. {}",
            tags_res.unwrap_err()
        );
        return get_500_internal_server_error();
    }
    info!("AdminBlogTagsList askama template rendered.");
    Html(tags_res.unwrap())
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
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    let tags_db_uc = app_state.tag_db_usecase.lock().await.clone().unwrap();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_cache_enabled = tags_cache_uc_opt.is_some();

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
        tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_display_repo
            .find(id.clone().unwrap())
            .await
    } else {
        None
    };

    // If cache hit, return early
    if let Some(res) = cache_result {
        let tag_res = res.to_admin_template().render();
        if tag_res.is_err() {
            error!(
                "Failed to render admin/blogs/tags/get_tag.html. {}",
                tag_res.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("AdminGetTag askama template rendered.");
        return Html(tag_res.unwrap());
    }

    // If not, get data from database
    let db_result = tags_db_uc.tag_display_repo.find(id.clone().unwrap()).await;

    // Early check db result. If empty, return 404 error
    if db_result.is_none() {
        info!("Failed to find Tag with Id {}.", &id.unwrap());
        return get_404_not_found().await;
    }

    // Insert cache
    if is_cache_enabled {
        debug!("Caching tag {}", &id.clone().unwrap());
        let _ = tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_operation_repo
            .insert(db_result.clone().unwrap())
            .await;
    }

    // Render Tag
    let tag = db_result.unwrap().to_admin_template().render();
    if tag.is_err() {
        error!(
            "Failed to render admin/blogs/tags/get_tag.html. {}",
            tag.unwrap_err()
        );
        return get_500_internal_server_error();
    }
    info!("AdminGetTag askama template rendered.");
    Html(tag.unwrap())
}

/// get_add_admin_tag
/// /// Serve GET add tag HTML file in a form format.
#[debug_handler]
pub async fn get_add_admin_tag(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let (user_agent, token) = process_login_header(headers).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
        return get_401_unauthorized().await;
    }

    // Locking Mutex
    let tag_uc = app_state.tag_db_usecase.lock().await.clone().unwrap();
    // Calculate new Blog Id
    let db_result = tag_uc.tag_operation_repo.get_new_id().await;

    let Some(id) = db_result else {
        error!("Failed to get new Tag ID.");
        return get_500_internal_server_error();
    };
    debug!("Construct AdminGetAddTagTemplate for Tag Id {}", &id);

    let add_tag = AdminGetAddTagTemplate { id }.render();
    debug!("AdminGetAddTagTemplate : {:?}", &add_tag);

    if add_tag.is_err() {
        error!(
            "Failed to render admin/blogs/get_add_tag.html. {}",
            add_tag.unwrap_err()
        );
        return get_500_internal_server_error();
    }

    info!("AdminGetAddTag askama template rendered.");
    Html(add_tag.unwrap())
}

/// get_edit_admin_tag
/// Serve GET edit tag HTML file to edit a tag
#[debug_handler]
pub async fn get_edit_admin_tag(
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

    let tag_uc = app_state.tag_db_usecase.lock().await.clone().unwrap();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_cache_enabled = tags_cache_uc_opt.is_some();

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
        tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_display_repo
            .find(id.clone().unwrap())
            .await
    } else {
        None
    };

    // If cache hit, return early
    if let Some(res) = cache_result {
        let edit_tag = AdminGetEditTagTemplate {
            id: res.id,
            name: res.name,
        }
        .render();
        if edit_tag.is_err() {
            error!(
                "Failed to render admin/blogs/tags/get_edit_tag.html. {}",
                edit_tag.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("AdminGetEditTag askama template rendered.");
        return Html(edit_tag.unwrap());
    }

    // If not, get data from database
    let db_result = tag_uc.tag_display_repo.find(id.clone().unwrap()).await;

    if db_result.is_none() {
        info!("Failed to find Tag with Id {}.", &path);
        return get_404_not_found().await;
    };

    // Insert cache
    if is_cache_enabled {
        debug!("Caching tag {}", &id.clone().unwrap());
        let _ = tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_operation_repo
            .insert(db_result.clone().unwrap())
            .await;
    }

    let edit_tag = AdminGetEditTagTemplate {
        id: id.clone().unwrap(),
        name: db_result.clone().unwrap().name.clone(),
    }
    .render();
    if edit_tag.is_err() {
        error!(
            "Failed to render admin/blogs/tags/get_edit_tag.html. {}",
            edit_tag.unwrap_err()
        );
        return get_500_internal_server_error();
    }

    info!("AdminGetEditTag askama template rendered.");
    Html(edit_tag.unwrap())
}

/// get_delete_admin_tag
/// Serve GET tag blog HTML file to delete a tag
#[debug_handler]
pub async fn get_delete_admin_tag(
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

    // Locking Mutex
    let tags_db_uc = app_state.tag_db_usecase.lock().await.clone().unwrap();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_cache_enabled = tags_cache_uc_opt.is_some();

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
        tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_display_repo
            .find(id.clone().unwrap())
            .await
    } else {
        None
    };

    // If cache hit, return early
    if let Some(res) = cache_result {
        let delete_tag = AdminGetDeleteTagTemplate { id: res.id }.render();
        if delete_tag.is_err() {
            error!(
                "Failed to render admin/blogs/tags/get_delete_tag.html. {}",
                delete_tag.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("AdminGetDeleteTag askama template rendered.");
        return Html(delete_tag.unwrap());
    }

    // If not, get data from database
    let db_result = tags_db_uc.tag_display_repo.find(id.clone().unwrap()).await;

    if db_result.is_none() {
        info!("Failed to find Tag with Id {}.", &path);
        return get_404_not_found().await;
    };

    // Insert cache
    if is_cache_enabled {
        debug!("Caching tag {}", &id.clone().unwrap());
        let _ = tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_operation_repo
            .insert(db_result.clone().unwrap())
            .await;
    }

    let delete_tag = AdminGetDeleteTagTemplate {
        id: id.clone().unwrap(),
    }
    .render();
    if delete_tag.is_err() {
        error!(
            "Failed to render admin/blogs/tags/get_delete_tag.html. {}",
            delete_tag.unwrap_err()
        );
        return get_500_internal_server_error();
    }

    info!("AdminGetDeleteTag askama template rendered.");
    Html(delete_tag.unwrap())
}
