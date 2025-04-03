use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::blogs::{BlogEndPage, BlogId, BlogPagination, BlogStartPage};
use crate::model::{
    axum::AppState,
    templates::{BlogTemplate, BlogsTemplate, BlogsTemplateBlog},
};
use askama::Template;
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::response::Html;
use tracing::{debug, error, info};

/// get_blogs
/// Serve get_blogs HTML file
/// List our blogs id and name
#[debug_handler]
pub async fn get_blogs(
    State(app_state): State<AppState>,
    pagination: Query<BlogPagination>,
) -> Html<String> {
    // Locking Mutex
    let data = app_state.blog_usecase.lock().await;

    // Setup Pagination
    debug!("Pagination {:?}", &pagination);
    let start = match pagination.0.start {
        Some(val) => val,
        None => {
            debug!("Set default start to 0");
            BlogStartPage(0)
        }
    };
    let end = match pagination.0.end {
        Some(val) => val,
        None => {
            debug!("Set default end to 10");
            BlogEndPage(10)
        }
    };

    // Construct BlogsTemplate Struct
    let result = data.blog_repo.find_blogs(start.clone(), end.clone()).await;
    match result {
        Some(blogs_data) => {
            let blogs: Vec<BlogsTemplateBlog> = blogs_data
                .iter()
                .map(|blog| {
                    debug!("Construct BlogsTemplateBlog for Blog Id {}", &blog.id);
                    BlogsTemplateBlog {
                        id: &blog.id.as_str(),
                        name: &blog.name.as_str(),
                    }
                })
                .collect();
            debug!("BlogsTemplate blogs : {:?}", &blogs);

            let blogs_res = BlogsTemplate { blogs: &blogs }.render();
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
    // Locking Mutex
    let data = app_state.blog_usecase.lock().await;

    // Construct BlogTemplate Struct
    let result = data.blog_repo.find(BlogId { id: path.clone() }).await;
    match result {
        Some(blog_data) => {
            let blog = BlogTemplate {
                id: path.clone().as_str(),
                name: &blog_data.name.as_str(),
                filename: &blog_data.filename.as_str(),
                body: &blog_data.body.as_str(),
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
            error!("Failed to find Blog with Id {}.", &path);
            get_404_not_found().await
        }
    }
}
