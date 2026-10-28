use crate::handler::auth::is_auth_verified;
use crate::handler::status::{
    get_401_unauthorized, get_404_not_found, get_500_internal_server_error,
};
use crate::model::axum::AppState;
use crate::model::blogs::BlogsParams;
use crate::model::tags::TagsListParams;
use crate::model::templates::BlogMetadataTemplate;
use crate::model::templates_admin::{
    AdminBlogsTemplate, AdminGetAddBlogTemplate, AdminGetBlogTemplate, AdminGetDeleteBlogTemplate,
    AdminGetEditBlogTemplate, AdminListBlogsTemplate,
};
use crate::utils::convert_tags_string_to_vec;
use askama::Template;
use axum::{
    debug_handler,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Html,
};
use tracing::{debug, error, info, warn};

/// get_base_admin_blogs
/// Serve GET (base) admin blogs HTML file
/// Under endpoint /admin/blogs
/// It's the base of Admin Blogs feature
#[debug_handler]
pub async fn get_base_admin_blogs(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    let blogs_res = AdminBlogsTemplate {}.render();
    match blogs_res {
        Ok(res) => {
            info!("Admin Blogs askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render admin/blogs/blogs.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_admin_blogs
/// Serve GET (base) admin blogs HTML file
/// Serve get_blogs HTML file and return point for /admin/blogs/add cancel button
/// Return lite version of get_base_admin_blogs with blogs data only
#[debug_handler]
pub async fn get_admin_blogs_list(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    params: Query<BlogsParams>,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    let mut blog_uc = app_state.blog_usecase.lock().await.clone();

    let sanitized_params = params.sanitize();
    let blogs_result = blog_uc.find_blogs(sanitized_params.clone()).await;

    // Early check result. If empty, return 500 error
    let Some(blogs_data) = blogs_result else {
        error!(
            "Failed to find admin blogs with Blog Id started at {} and ended at {}.",
            sanitized_params.start.unwrap(),
            sanitized_params.end.unwrap()
        );
        return get_500_internal_server_error();
    };

    let blogs: Vec<BlogMetadataTemplate> = blogs_data
        .iter()
        .map(|b| b.as_blog_metadata().as_template())
        .collect();
    let active_tags = convert_tags_string_to_vec(&sanitized_params.tags.clone().unwrap());

    let blogs_res = match (AdminListBlogsTemplate { blogs, active_tags }).render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/blogs/list_blogs.html. {}", err);
            return get_500_internal_server_error();
        }
    };
    info!("Admin Blogs askama template rendered.");
    Html(blogs_res)
}

/// get_admin_blog
/// Serve GET blog HTML file and return point for several cancelation endpoints
/// Returned single blog
#[debug_handler]
pub async fn get_admin_blog(
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

    let mut blog_uc = app_state.blog_usecase.lock().await.clone();
    let blog_result = blog_uc.find(id).await;

    // Early check result. If empty, return 404 error
    let Some(blog_data) = blog_result else {
        info!("Failed to find Blog with Id {}.", &id);
        return get_404_not_found().await;
    };

    let blog_tmpl = AdminGetBlogTemplate {
        blog: blog_data.as_blog_metadata().as_template(),
    };
    let blog_res = match blog_tmpl.render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/blogs/get_blog.html. {}", err);
            return get_500_internal_server_error();
        }
    };

    info!("Admin Blog askama template rendered.");
    Html(blog_res)
}

/// get_add_admin_blog
/// Serve GET add blog HTML file in a form format.
#[debug_handler]
pub async fn get_add_admin_blog(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    // Calculate new Blog Id
    let db_result = {
        let mut blog_uc = app_state.blog_usecase.lock().await.clone();
        blog_uc.get_new_id().await
    };

    let Some(blog_id) = db_result else {
        error!("Failed to get new Blog ID.");
        return get_500_internal_server_error();
    };
    debug!("Construct AdminGetAddBlogTemplate for Blog Id {}", &blog_id);

    // Get the available Tags through the Tag usecase (cache or database).
    let mut tag_uc = app_state.tag_usecase.lock().await.clone();
    let tags_result = tag_uc
        .find_tags(TagsListParams {
            start: Some(0),
            end: Some(1000),
        })
        .await;

    let Some(complete_tags_data) = tags_result else {
        error!("Failed to get find all available Tags.");
        return get_500_internal_server_error();
    };
    let avail_tags = complete_tags_data
        .tags
        .iter()
        .map(|tag| tag.name.clone())
        .collect();

    let add_blog_tmpl = AdminGetAddBlogTemplate {
        id: blog_id,
        avail_tags,
    };
    let add_blog = match add_blog_tmpl.render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/blogs/get_add_blog.html. {}", err);
            return get_500_internal_server_error();
        }
    };
    info!("AdminGetAddBlog askama template rendered.");
    Html(add_blog)
}

/// get_edit_admin_blog
/// Serve GET edit blog HTML file to edit a blog
#[debug_handler]
#[allow(clippy::type_complexity)]
pub async fn get_edit_admin_blog(
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

    // Get Blog data through the usecase (cache or database).
    let blog_result = {
        let mut blog_uc = app_state.blog_usecase.lock().await.clone();
        blog_uc.find(id).await
    };

    let Some(blog_data) = blog_result else {
        info!("Failed to find Blog with Id {}.", &path);
        return get_404_not_found().await;
    };
    debug!("Blog {:?}", &blog_data);

    // Get the available Tags through the Tag usecase (cache or database).
    // Tags not selected on the Blog are rendered as the unselected choices.
    let mut tag_uc = app_state.tag_usecase.lock().await.clone();
    let tags_result = tag_uc
        .find_tags(TagsListParams {
            start: Some(0),
            end: Some(1000),
        })
        .await;

    let Some(complete_tags_data) = tags_result else {
        error!("Failed to get all Tags");
        return get_500_internal_server_error();
    };
    let unselected_tags: Vec<String> = complete_tags_data
        .tags
        .iter()
        // Get tags not available in the current Blog Tags
        .filter(|t| !blog_data.tags.clone().unwrap().contains(&t.name))
        // Grab only tag name
        .map(|t| t.name.clone())
        .collect();

    debug!(
        "Construct AdminGetEditBlogTemplate for Blog Id {}",
        &blog_data.id
    );

    let edit_blog_tmpl = AdminGetEditBlogTemplate {
        id,
        name: blog_data.name.unwrap(),
        body: blog_data.body.unwrap(),
        blog_tags: blog_data.tags.unwrap(),
        avail_tags: unselected_tags,
    };
    let edit_blog = match edit_blog_tmpl.render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/blogs/get_edit_blog.html. {}", err);
            return get_500_internal_server_error();
        }
    };

    info!("AdminGetEditBlog askama template rendered.");
    Html(edit_blog)
}

/// get_delete_admin_blog
/// Serve GET delete blog HTML file to delete a blog
#[debug_handler]
pub async fn get_delete_admin_blog(
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

    // Get Blog data through the usecase (cache or database).
    let blog_result = {
        let mut blog_uc = app_state.blog_usecase.lock().await.clone();
        blog_uc.find(id).await
    };

    // If empty, return 404 error
    let Some(blog_data) = blog_result else {
        info!("Failed to find Blog with Id {}.", &path);
        return get_404_not_found().await;
    };

    let delete_blog = match (AdminGetDeleteBlogTemplate { id: blog_data.id }).render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render admin/blogs/get_delete_blog.html. {}", err);
            return get_500_internal_server_error();
        }
    };

    info!("AdminGetDeleteBlog askama template rendered.");
    Html(delete_blog)
}
