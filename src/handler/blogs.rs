use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::blogs::{Blog, BlogsParams};
use crate::model::{
    axum::AppState,
    templates::{BlogMetadataTemplate, BlogsTemplate},
};
use crate::utils::convert_tags_string_to_vec;
use askama::Template;
use axum::{
    debug_handler,
    extract::{Path, Query, State},
    response::Html,
};
use tracing::{debug, error, info, warn};

/// get_blogs
/// Serve get_blogs HTML file
/// List our blogs id and name
#[debug_handler]
pub async fn get_blogs(
    State(app_state): State<AppState>,
    params: Query<BlogsParams>,
) -> Html<String> {
    let mut blog_uc = app_state.blog_usecase.lock().await.clone();

    let sanitized_params = params.sanitize();
    let blogs_result = blog_uc.find_blogs(sanitized_params.clone()).await;

    // Early check db result. If empty, return 500 error
    let Some(blogs_data) = blogs_result else {
        error!(
            "Failed to find blogs with Blog Id started at {} and ended at {}.",
            sanitized_params.start.unwrap(),
            sanitized_params.end.unwrap()
        );
        return get_500_internal_server_error();
    };

    render_blogs_list_html(blogs_data, sanitized_params.tags.clone().unwrap())
}

fn render_blogs_list_html(blogs_data: Vec<Blog>, tags: String) -> Html<String> {
    let blogs: Vec<BlogMetadataTemplate> = blogs_data
        .iter()
        .map(|b| b.as_blog_metadata().as_template())
        .collect();
    let active_tags = convert_tags_string_to_vec(&tags);

    let blogs_res = match (BlogsTemplate { blogs, active_tags }).render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render get_blogs.html. {}", err);
            return get_500_internal_server_error();
        }
    };
    info!("Blogs askama template rendered.");
    Html(blogs_res)
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

    let mut blog_uc = app_state.blog_usecase.lock().await.clone();
    // The usecase resolves the Blog through the cache or the database; a `None`
    // here means the Blog does not exist.
    let blog_result = blog_uc.find(id.clone().unwrap()).await;

    // Early check result. If empty, return 404 error
    let Some(blog_data) = blog_result else {
        info!("Failed to find Blog with Id {}.", &id.unwrap());
        return get_404_not_found().await;
    };

    let blog = match blog_data.as_template().render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render blog.html. {}", err);
            return get_500_internal_server_error();
        }
    };
    info!("Blog ID {} askama template rendered.", &path);
    Html(blog)
}
