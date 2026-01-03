use crate::handler::auth::{process_login_header, verify_jwt};
use crate::handler::status::get_401_unauthorized;
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::axum::AppState;
use crate::model::tags::{TagsListParams, TagsSearchParams};
use crate::model::templates_admin::{
    AdminBlogTagsListTemplate, AdminBlogTagsTemplate, AdminGetAddTagTemplate,
    AdminGetDeleteTagTemplate, AdminGetEditTagTemplate, AdminGetTagTemplate,
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
    let data = app_state.tag_db_usecase.lock().await.clone().unwrap();

    // Setup Pagination
    debug!("Query Parameters {:?}", &params);
    let start = match params.start {
        Some(val) => Some(val),
        None => {
            debug!("Set default start to 0");
            Some(0_i64)
        }
    };
    let end = match params.end {
        Some(val) => Some(val),
        None => {
            debug!("Set default end to 100");
            Some(100_i64)
        }
    };
    let query_params = TagsListParams { start, end };

    // Construct AdminBlogTagsTemplate Struct
    let result = data.tag_repo.find_tags(query_params).await;
    match result {
        Some(tags_data) => {
            let tags_res = AdminBlogTagsListTemplate {
                tags: tags_data.tags,
            }
            .render();
            match tags_res {
                Ok(res) => {
                    info!("Admin Blogs askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/blogs/tags/list_tags.html. {}", err);
                    get_500_internal_server_error()
                }
            }
        }
        None => {
            error!(
                "Failed to find admin blog tags with Tag Id started at {} and ended at {}.",
                start.unwrap(),
                end.unwrap()
            );
            get_500_internal_server_error()
        }
    }
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
    let data = app_state.tag_db_usecase.lock().await.clone().unwrap();

    // Setup Pagination
    debug!("Query Parameters {:?}", &params);
    let start = match params.start {
        Some(val) => Some(val),
        None => {
            debug!("Set default start to 0");
            Some(0_i64)
        }
    };
    let end = match params.end {
        Some(val) => Some(val),
        None => {
            debug!("Set default end to 100");
            Some(100_i64)
        }
    };
    let query_params = TagsSearchParams {
        start,
        end,
        query: params.query.clone(),
    };

    // Construct AdminBlogTagsTemplate Struct
    let result = data.tag_repo.search_tags(query_params).await;
    match result {
        Some(tags_data) => {
            let tags_res = AdminBlogTagsListTemplate {
                tags: tags_data.tags,
            }
            .render();
            match tags_res {
                Ok(res) => {
                    info!("Admin Blogs askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/blogs/tags/list_tags.html. {}", err);
                    get_500_internal_server_error()
                }
            }
        }
        None => {
            error!(
                "Failed to find admin blog tags with Tag Id started at {} and ended at {}.",
                start.unwrap(),
                end.unwrap()
            );
            get_500_internal_server_error()
        }
    }
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

    let data = app_state.tag_db_usecase.lock().await.clone().unwrap();
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

    let result = data.tag_repo.find(id.unwrap()).await;

    match result {
        Some(tag) => {
            let tag_res = AdminGetTagTemplate {
                id: tag.id,
                name: tag.name,
            }
            .render();
            match tag_res {
                Ok(res) => {
                    info!("Admin Tag askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/blogs/tags/get_tag.html. {}", err);
                    get_500_internal_server_error()
                }
            }
        }
        None => {
            info!("Failed to find Tag with Id {}.", &path);
            get_404_not_found().await
        }
    }
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
    let result = tag_uc.tag_repo.get_new_id().await;

    let Some(id) = result else {
        error!("Failed to get new Blog ID.");
        return get_500_internal_server_error();
    };
    debug!("Construct AdminGetAddTagTemplate for Tag Id {}", &id);

    let add_tag = AdminGetAddTagTemplate { id }.render();
    debug!("AdminGetAddTagTemplate : {:?}", &add_tag);

    match add_tag {
        Ok(res) => {
            info!("Blogs askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render admin/blogs/get_add_tag.html. {}", err);
            get_500_internal_server_error()
        }
    }
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

    let tag_result = tag_uc.tag_repo.find(id.clone().unwrap()).await;

    let Some(tag_data) = tag_result else {
        info!("Failed to find Tag with Id {}.", &path);
        return get_404_not_found().await;
    };

    debug!(
        "Construct AdminGetEditTagTemplate for Tag Id {}",
        &tag_data.id
    );

    let edit_tag = AdminGetEditTagTemplate {
        id: id.clone().unwrap(),
        name: tag_data.name.clone(),
    }
    .render();
    debug!("AdminGetEditTagTemplate : {:?}", &edit_tag);

    match edit_tag {
        Ok(res) => {
            info!("Talks askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!(
                "Failed to render admin/blogs/tags/get_edit_tag.html. {}",
                err
            );
            get_500_internal_server_error()
        }
    }
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
    let data = app_state.tag_db_usecase.lock().await.clone().unwrap();
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

    let result = data.tag_repo.find(id.clone().unwrap()).await;

    match result {
        Some(tag) => {
            debug!("Construct AdminGetDeleteTagTemplate for Tag Id {}", &tag.id);

            let delete_tag = AdminGetDeleteTagTemplate {
                id: id.clone().unwrap(),
            }
            .render();
            debug!("AdminGetDeleteTagTemplate : {:?}", &delete_tag);

            match delete_tag {
                Ok(res) => {
                    info!("Talks askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!(
                        "Failed to render admin/blogs/tags/get_delete_tag.html. {}",
                        err
                    );
                    get_500_internal_server_error()
                }
            }
        }
        None => {
            info!("Failed to find Tag with Id {}.", &path);
            get_404_not_found().await
        }
    }
}
