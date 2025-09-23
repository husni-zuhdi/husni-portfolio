use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::axum::AppState;
use crate::model::blogs::{BlogEndPage, BlogStartPage, BlogsParams};
use crate::model::templates::BlogMetadataTemplate;
use crate::model::templates_admin::{
    AdminBlogsTemplate, AdminGetAddBlogTemplate, AdminGetBlogTemplate, AdminGetBlogsTemplate,
    AdminGetDeleteBlogTemplate, AdminGetEditBlogTemplate,
};
use crate::utils::remove_whitespace;

use askama::Template;
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::response::Html;
use tracing::{debug, error, info, warn};

/// get_base_admin_blogs
/// Serve GET (base) admin blogs HTML file
/// Under endpoint /admin/blogs
/// It's the base of Admin Blogs feature
#[debug_handler]
pub async fn get_base_admin_blogs(
    State(app_state): State<AppState>,
    params: Query<BlogsParams>,
) -> Html<String> {
    // Locking Mutex
    let data = app_state.blog_usecase.lock().await;

    // Setup Pagination
    debug!("Query Parameters {:?}", &params);
    let start = match params.0.start {
        Some(val) => val,
        None => {
            debug!("Set default start to 0");
            BlogStartPage(0)
        }
    };
    let end = match params.0.end {
        Some(val) => val,
        None => {
            debug!("Set default end to 10");
            BlogEndPage(10)
        }
    };
    let tags: String = match params.0.tags {
        Some(val) => remove_whitespace(&val),
        None => {
            debug!("Set default tags to empty");
            "".to_string()
        }
    };

    let query_params = BlogsParams {
        start: Some(start.clone()),
        end: Some(end.clone()),
        tags: Some(tags.clone()),
    };

    // Construct BlogsTemplate Struct
    let result = data.blog_repo.find_blogs(query_params).await;
    match result {
        Some(blogs_data) => {
            let blogs: Vec<BlogMetadataTemplate> = blogs_data
                .iter()
                .map(|blog| {
                    debug!("Construct BlogMetadataTemplate for Blog Id {}", &blog.id);
                    BlogMetadataTemplate {
                        id: blog.id,
                        name: blog.name.clone(),
                        tags: blog.tags.clone(),
                    }
                })
                .collect();
            debug!("AdminBlogsTemplate blogs : {:?}", &blogs);

            let active_tags: Vec<String> = tags.clone().split(",").map(|t| t.to_string()).collect();

            let blogs_res = AdminBlogsTemplate { blogs, active_tags }.render();
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
        None => {
            error!(
                "Failed to find admin blogs with Blog Id started at {} and ended at {}.",
                &start.0, &end.0
            );
            get_500_internal_server_error()
        }
    }
}

/// get_admin_blogs
/// Serve GET (base) admin blogs HTML file
/// Serve get_blogs HTML file and return point for /admin/blogs/add cancel button
/// Return lite version of get_base_admin_blogs with blogs data only
#[debug_handler]
pub async fn get_admin_blogs(
    State(app_state): State<AppState>,
    params: Query<BlogsParams>,
) -> Html<String> {
    // Locking Mutex
    let data = app_state.blog_usecase.lock().await;

    // Setup Pagination
    debug!("Query Parameters {:?}", &params);
    let start = match params.0.start {
        Some(val) => val,
        None => {
            debug!("Set default start to 0");
            BlogStartPage(0)
        }
    };
    let end = match params.0.end {
        Some(val) => val,
        None => {
            debug!("Set default end to 10");
            BlogEndPage(10)
        }
    };
    let tags: String = match params.0.tags {
        Some(val) => remove_whitespace(&val),
        None => {
            debug!("Set default tags to empty");
            "".to_string()
        }
    };

    let query_params = BlogsParams {
        start: Some(start.clone()),
        end: Some(end.clone()),
        tags: Some(tags.clone()),
    };

    // Construct BlogsTemplate Struct
    let result = data.blog_repo.find_blogs(query_params).await;
    match result {
        Some(blogs_data) => {
            let blogs: Vec<BlogMetadataTemplate> = blogs_data
                .iter()
                .map(|blog| {
                    debug!("Construct BlogMetadataTemplate for Blog Id {}", &blog.id);
                    BlogMetadataTemplate {
                        id: blog.id,
                        name: blog.name.clone(),
                        tags: blog.tags.clone(),
                    }
                })
                .collect();
            debug!("AdminBlogsTemplate blogs : {:?}", &blogs);

            let active_tags: Vec<String> = tags.clone().split(",").map(|t| t.to_string()).collect();

            let blogs_res = AdminGetBlogsTemplate { blogs, active_tags }.render();
            match blogs_res {
                Ok(res) => {
                    info!("Admin Blogs askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/blogs/get_blogs.html. {}", err);
                    get_500_internal_server_error()
                }
            }
        }
        None => {
            error!(
                "Failed to find admin blogs with Blog Id started at {} and ended at {}.",
                &start.0, &end.0
            );
            get_500_internal_server_error()
        }
    }
}

/// get_admin_blog
/// Serve GET blog HTML file and return point for several cancelation endpoints
/// Returned single blog
#[debug_handler]
pub async fn get_admin_blog(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
) -> Html<String> {
    let data = app_state.blog_usecase.lock().await.clone();
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

    let result = data.blog_repo.find(id.clone().unwrap()).await;

    match result {
        Some(blog_data) => {
            debug!(
                "Construct BlogMetadataTemplate for Blog Id {}",
                &blog_data.id
            );
            let blog_metadata = BlogMetadataTemplate {
                id: blog_data.id,
                name: blog_data.name.clone().unwrap(),
                tags: blog_data.tags.clone().unwrap(),
            };
            debug!("BlogMetadataTemplate blogs : {:?}", &blog_metadata);

            let blogs_res = AdminGetBlogTemplate {
                blog: blog_metadata,
            }
            .render();
            match blogs_res {
                Ok(res) => {
                    info!("Admin Blog askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/blogs/get_blog.html. {}", err);
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

/// get_add_admin_blog
/// Serve GET add blog HTML file in a form format.
#[debug_handler]
pub async fn get_add_admin_blog(State(app_state): State<AppState>) -> Html<String> {
    // Locking Mutex
    let blog_uc = app_state.blog_usecase.lock().await;

    // Calculate new Blog Id
    let result = blog_uc.blog_repo.get_new_id().await;

    let Some(blog_id) = result else {
        error!("Failed to get new Blog ID.");
        return get_500_internal_server_error();
    };
    debug!("Construct AdminGetAddBlogTemplate for Blog Id {}", &blog_id);

    let tag_uc = app_state.tag_usecase.lock().await;
    if tag_uc.is_none() {
        error!("Failed to get lock Tag Usecase Mutex.");
        return get_500_internal_server_error();
    };

    let tag_result = tag_uc.clone().unwrap().tag_repo.find_all_tags().await;
    if tag_result.is_none() {
        error!("Failed to get find all available Tags.");
        return get_500_internal_server_error();
    }

    let avail_tags = tag_result
        .unwrap()
        .tags
        .iter()
        .map(|tag| tag.name.clone())
        .collect();

    let add_blog = AdminGetAddBlogTemplate {
        id: blog_id,
        avail_tags,
    }
    .render();
    debug!("AdminGetAddBlogTemplate : {:?}", &add_blog);

    match add_blog {
        Ok(res) => {
            info!("Blogs askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render admin/blogs/get_add_blog.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_edit_admin_blog
/// Serve GET edit blog HTML file to edit a blog
#[debug_handler]
pub async fn get_edit_admin_blog(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
) -> Html<String> {
    let blog_uc = app_state.blog_usecase.lock().await.clone();
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

    let blog_result = blog_uc.blog_repo.find(id.clone().unwrap()).await;

    let Some(blog_data) = blog_result else {
        info!("Failed to find Blog with Id {}.", &path);
        return get_404_not_found().await;
    };
    debug!("Blog {:?}", &blog_data);

    let tag_uc = app_state.tag_usecase.lock().await.clone();
    if tag_uc.is_none() {
        error!("Failed to lock Tag Usecase Mutex.");
        return get_500_internal_server_error();
    }

    let tags_result = tag_uc.unwrap().tag_repo.find_all_tags().await;
    let Some(complete_tags_data) = tags_result else {
        error!("Failed to get all Tags");
        return get_500_internal_server_error();
    };
    let avail_tags: Vec<String> = complete_tags_data
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

    let edit_blog = AdminGetEditBlogTemplate {
        id: id.clone().unwrap(),
        name: blog_data.name.unwrap().clone(),
        body: blog_data.body.unwrap().clone(),
        blog_tags: blog_data.tags.unwrap().clone(),
        avail_tags,
    }
    .render();
    debug!("AdminGetEditBlogTemplate : {:?}", &edit_blog);

    match edit_blog {
        Ok(res) => {
            info!("Talks askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render admin/blogs/get_edit_blog.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_delete_admin_blog
/// Serve GET delete blog HTML file to delete a blog
#[debug_handler]
pub async fn get_delete_admin_blog(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
) -> Html<String> {
    // Locking Mutex
    let data = app_state.blog_usecase.lock().await.clone();
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

    let result = data.blog_repo.find(id.clone().unwrap()).await;

    match result {
        Some(blog_data) => {
            debug!(
                "Construct AdminGetDeleteBlogTemplate for Blog Id {}",
                &blog_data.id
            );
            debug!("Blog {:?}", &blog_data);

            let delete_blog = AdminGetDeleteBlogTemplate { id: id.unwrap() }.render();
            debug!("AdminGetDeleteBlogTemplate : {:?}", &delete_blog);

            match delete_blog {
                Ok(res) => {
                    info!("Talks askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render admin/blogs/get_delete_blog.html. {}", err);
                    get_500_internal_server_error()
                }
            }
        }
        None => {
            info!("Failed to find Blog with Id {}.", &path);
            get_404_not_found().await
        }
    }
}
