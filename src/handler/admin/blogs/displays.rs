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
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
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

    // Locking Mutex
    let blog_cache_uc_opt = app_state.blog_cache_usecase.lock().await;
    let cache_is_enabled = blog_cache_uc_opt.is_some();

    let sanitized_params = params.sanitize();

    // Get Data from Cache
    let cache_result = if cache_is_enabled {
        blog_cache_uc_opt
            .clone()
            .unwrap()
            .blog_display_repo
            .find_blogs(sanitized_params.clone())
            .await
    } else {
        None
    };
    // If cache hit, return early
    if let Some(res) = cache_result {
        let blogs: Vec<BlogMetadataTemplate> = res
            .iter()
            .map(|b| b.as_blog_metadata().as_template())
            .collect();
        let active_tags = convert_tags_string_to_vec(&sanitized_params.tags.clone().unwrap());
        let blogs_res = AdminListBlogsTemplate { blogs, active_tags }.render();
        if blogs_res.is_err() {
            error!("Failed to render blogs.html. {}", blogs_res.unwrap_err());
            return get_500_internal_server_error();
        }

        info!("Blogs askama template rendered.");
        return Html(blogs_res.unwrap());
    }

    // If not, get data from database
    let db_result = app_state
        .blog_db_usecase
        .lock()
        .await
        .blog_display_repo
        .find_blogs(sanitized_params.clone())
        .await;

    // Early check db result. If empty, return 500 error
    if db_result.is_none() {
        error!(
            "Failed to find admin blogs with Blog Id started at {} and ended at {}.",
            sanitized_params.start.unwrap(),
            sanitized_params.end.unwrap()
        );
        return get_500_internal_server_error();
    }

    // Insert cache
    if cache_is_enabled {
        for blog in db_result.clone().unwrap() {
            debug!("Caching blog {}", &blog.id);
            let _ = blog_cache_uc_opt
                .clone()
                .unwrap()
                .blog_operation_repo
                .insert(blog)
                .await;
        }
        drop(blog_cache_uc_opt);
    }

    let blogs: Vec<BlogMetadataTemplate> = db_result
        .unwrap()
        .iter()
        .map(|b| b.as_blog_metadata().as_template())
        .collect();
    let active_tags = convert_tags_string_to_vec(&sanitized_params.tags.clone().unwrap());

    let blogs_res = AdminListBlogsTemplate { blogs, active_tags }.render();

    if blogs_res.is_err() {
        error!(
            "Failed to render admin/blogs/list_blogs.html. {}",
            blogs_res.unwrap_err()
        );
        return get_500_internal_server_error();
    }
    info!("Admin Blogs askama template rendered.");
    Html(blogs_res.unwrap())
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

    // Locking Mutex
    let blog_cache_uc_opt = app_state.blog_cache_usecase.lock().await;
    let cache_is_enabled = blog_cache_uc_opt.is_some();

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    // Get Data from Cache
    let cache_result = if cache_is_enabled {
        blog_cache_uc_opt
            .clone()
            .unwrap()
            .blog_display_repo
            .find(id)
            .await
    } else {
        None
    };
    // If cache hit, return early
    if let Some(res) = cache_result {
        let blog_res = AdminGetBlogTemplate {
            blog: res.as_blog_metadata().as_template(),
        }
        .render();
        if blog_res.is_err() {
            error!(
                "Failed to render admin/blogs/get_blog.html. {}",
                blog_res.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("Admin Blog askama template rendered.");
        return Html(blog_res.unwrap());
    }
    // If not, get data from database
    let db_result = app_state
        .blog_db_usecase
        .lock()
        .await
        .clone()
        .blog_display_repo
        .find(id)
        .await;

    // Early check db result. If empty, return 404 error
    if db_result.is_none() {
        info!("Failed to find Blog with Id {}.", &id);
        return get_404_not_found().await;
    }

    // Insert cache
    if cache_is_enabled {
        debug!("Caching blog {}", &id);
        let _ = blog_cache_uc_opt
            .clone()
            .unwrap()
            .blog_operation_repo
            .insert(db_result.clone().unwrap())
            .await;
        drop(blog_cache_uc_opt);
    }

    let blog = AdminGetBlogTemplate {
        blog: db_result.clone().unwrap().as_blog_metadata().as_template(),
    }
    .render();

    if blog.is_err() {
        error!("Failed to render blog.html. {}", blog.unwrap_err());
        return get_500_internal_server_error();
    }
    info!("Blog ID {} askama template rendered.", &path);
    Html(blog.unwrap())
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

    // Locking Mutex
    // Calculate new Blog Id
    let db_result = app_state
        .blog_db_usecase
        .lock()
        .await
        .blog_operation_repo
        .get_new_id()
        .await;

    let Some(blog_id) = db_result else {
        error!("Failed to get new Blog ID.");
        return get_500_internal_server_error();
    };
    debug!("Construct AdminGetAddBlogTemplate for Blog Id {}", &blog_id);

    let tag_db_uc = app_state.tag_db_usecase.lock().await;
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_cache_enabled = tags_cache_uc_opt.is_some();

    if tag_db_uc.is_none() {
        error!("Failed to get lock Tag Usecase Mutex.");
        return get_500_internal_server_error();
    };

    // Get Tags Data from Cache
    let tags_cache_result = if is_cache_enabled {
        tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_display_repo
            .find_tags(TagsListParams {
                start: Some(0),
                end: Some(1000),
            })
            .await
    } else {
        None
    };

    // If cache hit, return early
    if let Some(res) = tags_cache_result {
        let avail_tags = res.tags.iter().map(|tag| tag.name.clone()).collect();

        let add_blog = AdminGetAddBlogTemplate {
            id: blog_id,
            avail_tags,
        }
        .render();
        if add_blog.is_err() {
            error!(
                "Failed to render admin/blogs/get_add_blog.html. {}",
                add_blog.unwrap_err()
            );
            return get_500_internal_server_error();
        }
        info!("AdminGetAddBlog askama template rendered.");
        return Html(add_blog.unwrap());
    }

    let tags_db_result = tag_db_uc
        .clone()
        .unwrap()
        .tag_display_repo
        .find_tags(TagsListParams {
            start: Some(0),
            end: Some(1000),
        })
        .await;
    drop(tag_db_uc);

    if tags_db_result.is_none() {
        error!("Failed to get find all available Tags.");
        return get_500_internal_server_error();
    }

    let avail_tags = tags_db_result
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
    if add_blog.is_err() {
        error!(
            "Failed to render admin/blogs/get_add_blog.html. {}",
            add_blog.unwrap_err()
        );
        return get_500_internal_server_error();
    }
    info!("AdminGetAddBlog askama template rendered.");
    Html(add_blog.unwrap())
}

/// get_edit_admin_blog
/// Serve GET edit blog HTML file to edit a blog
#[debug_handler]
pub async fn get_edit_admin_blog(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    let blog_uc = app_state.blog_db_usecase.lock().await.clone();
    let blog_cache_uc_opt = app_state.blog_cache_usecase.lock().await.clone();
    let is_blog_cache_enabled = blog_cache_uc_opt.is_some();

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    // Get Blog Data from Cache
    let blog_cache_result = if is_blog_cache_enabled {
        blog_cache_uc_opt
            .clone()
            .unwrap()
            .blog_display_repo
            .find(id)
            .await
    } else {
        None
    };

    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_tags_cache_enabled = tags_cache_uc_opt.is_some();

    // Get Tags Data from Cache
    let tags_cache_result = if is_tags_cache_enabled {
        tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_display_repo
            .find_tags(TagsListParams {
                start: Some(0),
                end: Some(1000),
            })
            .await
    } else {
        None
    };

    // If cache hit, return early
    if blog_cache_result.is_some() && tags_cache_result.clone().is_some() {
        let unselected_tags: Vec<String> = tags_cache_result
            .unwrap()
            .tags
            .iter()
            // Get tags not available in the current Blog Tags
            .filter(|t| {
                !blog_cache_result
                    .clone()
                    .unwrap()
                    .tags
                    .unwrap()
                    .contains(&t.name)
            })
            // Grab only tag name
            .map(|t| t.name.clone())
            .collect();

        let edit_blog = AdminGetEditBlogTemplate {
            id,
            name: blog_cache_result.clone().unwrap().name.unwrap(),
            body: blog_cache_result.clone().unwrap().body.unwrap(),
            blog_tags: blog_cache_result.clone().unwrap().tags.unwrap(),
            avail_tags: unselected_tags,
        }
        .render();
        if edit_blog.is_err() {
            error!(
                "Failed to render admin/blogs/get_edit_blog.html. {}",
                edit_blog.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("AdminGetEditBlog askama template rendered.");
        return Html(edit_blog.unwrap());
    }

    // If not, get data from database
    let blog_db_result = blog_uc.blog_display_repo.find(id).await;

    let Some(blog_data) = blog_db_result else {
        info!("Failed to find Blog with Id {}.", &path);
        return get_404_not_found().await;
    };
    debug!("Blog {:?}", &blog_data);

    let tag_db_uc = app_state.tag_db_usecase.lock().await.clone();
    if tag_db_uc.is_none() {
        error!("Failed to lock Tag Usecase Mutex.");
        return get_500_internal_server_error();
    }

    let tags_result = tag_db_uc
        .unwrap()
        .tag_display_repo
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

    let edit_blog = AdminGetEditBlogTemplate {
        id,
        name: blog_data.name.unwrap(),
        body: blog_data.body.unwrap(),
        blog_tags: blog_data.tags.unwrap(),
        avail_tags: unselected_tags,
    }
    .render();
    debug!("AdminGetEditBlogTemplate : {:?}", &edit_blog);
    if edit_blog.is_err() {
        error!(
            "Failed to render admin/blogs/get_edit_blog.html. {}",
            edit_blog.unwrap_err()
        );
        return get_500_internal_server_error();
    }

    info!("AdminGetEditBlog askama template rendered.");
    Html(edit_blog.unwrap())
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

    // Locking Mutex
    let blog_db_uc = app_state.blog_db_usecase.lock().await.clone();
    let blogs_cache_uc_opt = app_state.blog_cache_usecase.lock().await.clone();
    let is_cache_enabled = blogs_cache_uc_opt.is_some();

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        blogs_cache_uc_opt
            .clone()
            .unwrap()
            .blog_display_repo
            .find(id)
            .await
    } else {
        None
    };

    // If cache hit, return early
    if let Some(res) = cache_result {
        let delete_blog = AdminGetDeleteBlogTemplate { id: res.id }.render();
        if delete_blog.is_err() {
            error!(
                "Failed to render admin/blogs/get_delete_blog.html. {}",
                delete_blog.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("AdminGetDeleteBlog askama template rendered.");
        return Html(delete_blog.unwrap());
    }

    // If not, get data from database
    let db_result = blog_db_uc.blog_display_repo.find(id).await;

    if db_result.is_none() {
        info!("Failed to find Blog with Id {}.", &path);
        return get_404_not_found().await;
    };

    // Insert cache
    if is_cache_enabled {
        debug!("Caching tag {}", &id);
        let _ = blogs_cache_uc_opt
            .clone()
            .unwrap()
            .blog_operation_repo
            .insert(db_result.unwrap())
            .await;
    }

    let delete_blog = AdminGetDeleteBlogTemplate { id }.render();
    if delete_blog.is_err() {
        error!(
            "Failed to render admin/blogs/get_delete_blog.html. {}",
            delete_blog.unwrap_err()
        );
        return get_500_internal_server_error();
    }

    info!("AdminGetDeleteBlog askama template rendered.");
    Html(delete_blog.unwrap())
}
