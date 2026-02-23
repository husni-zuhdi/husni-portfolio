use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::blogs::BlogsParams;
use crate::model::{
    axum::AppState,
    templates::{BlogMetadataTemplate, BlogsTemplate},
};
use crate::utils::convert_tags_string_to_vec;
use askama::Template;
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::response::Html;
use tracing::{debug, error, info, warn};

/// get_blogs
/// Serve get_blogs HTML file
/// List our blogs id and name
#[debug_handler]
pub async fn get_blogs(
    State(app_state): State<AppState>,
    params: Query<BlogsParams>,
) -> Html<String> {
    // Locking Mutex
    let blog_db_uc = app_state.blog_db_usecase.lock().await;
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
        let blogs_res = BlogsTemplate { blogs, active_tags }.render();
        if blogs_res.is_err() {
            error!("Failed to render blogs.html. {}", blogs_res.unwrap_err());
            return get_500_internal_server_error();
        }

        info!("Blogs askama template rendered.");
        return Html(blogs_res.unwrap());
    }

    // If not, get data from database
    let db_result = blog_db_uc
        .blog_display_repo
        .find_blogs(sanitized_params.clone())
        .await;

    // Early check db result. If empty, return 500 error
    if db_result.is_none() {
        error!(
            "Failed to find blogs with Blog Id started at {} and ended at {}.",
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
    }

    let blogs: Vec<BlogMetadataTemplate> = db_result
        .unwrap()
        .iter()
        .map(|b| b.as_blog_metadata().as_template())
        .collect();
    let active_tags = convert_tags_string_to_vec(&sanitized_params.tags.clone().unwrap());

    let blogs_res = BlogsTemplate { blogs, active_tags }.render();

    if blogs_res.is_err() {
        error!(
            "Failed to render get_blogs.html. {}",
            blogs_res.unwrap_err()
        );
        return get_500_internal_server_error();
    }
    info!("Blogs askama template rendered.");
    Html(blogs_res.unwrap())
}

/// get_blog
/// Serve get_blog HTML file
/// Render our blog
#[debug_handler]
pub async fn get_blog(Path(path): Path<String>, State(app_state): State<AppState>) -> Html<String> {
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
    }

    // Locking Mutex
    let blog_db_uc = app_state.blog_db_usecase.lock().await;
    let blog_cache_uc_opt = app_state.blog_cache_usecase.lock().await;
    let cache_is_enabled = blog_cache_uc_opt.is_some();

    // Get Data from Cache
    let cache_result = if cache_is_enabled {
        blog_cache_uc_opt
            .clone()
            .unwrap()
            .blog_display_repo
            .find(id.clone().unwrap())
            .await
    } else {
        None
    };
    // If cache hit, return early
    if let Some(res) = cache_result {
        let blog_res = res.as_template().render();
        if blog_res.is_err() {
            error!("Failed to render blog.html. {}", blog_res.unwrap_err());
            return get_500_internal_server_error();
        }

        info!("Blog askama template rendered.");
        return Html(blog_res.unwrap());
    }
    // If not, get data from database
    let db_result = blog_db_uc.blog_display_repo.find(id.clone().unwrap()).await;

    // Early check db result. If empty, return 404 error
    if db_result.is_none() {
        info!("Failed to find Blog with Id {}.", &id.unwrap());
        return get_404_not_found().await;
    }

    // Insert cache
    if cache_is_enabled {
        debug!("Caching blog {}", &id.clone().unwrap());
        let _ = blog_cache_uc_opt
            .clone()
            .unwrap()
            .blog_operation_repo
            .insert(db_result.clone().unwrap())
            .await;
    }

    let blog = db_result.unwrap().as_template().render();

    if blog.is_err() {
        error!("Failed to render blog.html. {}", blog.unwrap_err());
        return get_500_internal_server_error();
    }
    info!("Blog ID {} askama template rendered.", &path);
    Html(blog.unwrap())
}
