use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::blogs::{BlogEndPage, BlogId, BlogStartPage, BlogsParams};
use crate::model::{
    axum::AppState,
    templates::{BlogMetadataTemplate, BlogTemplate, BlogsTemplate},
};
use crate::utils::remove_whitespace;
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
        tags: Some(tags),
    };

    // Construct BlogsTemplate Struct
    // TODO: implement tags on handler and database adapter
    let result = data.blog_repo.find_blogs(query_params).await;
    match result {
        Some(blogs_data) => {
            let blogs: Vec<BlogMetadataTemplate> = blogs_data
                .iter()
                .map(|blog| {
                    debug!("Construct BlogsTemplateBlog for Blog Id {}", &blog.id);
                    BlogMetadataTemplate {
                        id: blog.id.id,
                        name: blog.name.clone(),
                        tags: blog.tags.clone(),
                    }
                })
                .collect();
            debug!("BlogsTemplate blogs : {:?}", &blogs);

            let blogs_res = BlogsTemplate { blogs }.render();
            match blogs_res {
                Ok(res) => {
                    info!("Blogs askama template rendered.");
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render get_blogs.html. {}", err);
                    get_500_internal_server_error()
                }
            }
        }
        None => {
            error!(
                "Failed to find blogs with Blog Id started at {} and ended at {}.",
                &start.0, &end.0
            );
            get_500_internal_server_error()
        }
    }
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
            // TIL i forgot this kind of early return exist in rust.
            // TBH I forgot to use `return` keyword lmao
            return get_404_not_found().await;
        }
    }

    // Locking Mutex
    let data = app_state.blog_usecase.lock().await;

    // Construct BlogTemplate Struct
    let result = data
        .blog_repo
        .find(BlogId {
            id: id.clone().unwrap(),
        })
        .await;
    match result {
        Some(blog_data) => {
            let tags_string = blog_data.tags.unwrap();

            let tags = tags_string.iter().map(|tag| tag.as_str()).collect();
            let blog = BlogTemplate {
                id: &id.clone().unwrap(),
                name: &blog_data.name.unwrap().as_str(),
                filename: &blog_data.filename.unwrap().as_str(),
                body: &blog_data.body.unwrap().as_str(),
                tags: &tags,
            }
            .render();

            match blog {
                Ok(res) => {
                    info!("Blog ID {} askama template rendered.", &path);
                    Html(res)
                }
                Err(err) => {
                    error!("Failed to render blog.html. {}", err);
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
