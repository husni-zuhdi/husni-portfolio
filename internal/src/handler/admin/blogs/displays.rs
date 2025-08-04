use crate::handler::status::get_500_internal_server_error;
use crate::model::axum::AppState;
use crate::model::blogs::{BlogEndPage, BlogStartPage, BlogsParams};
use crate::model::templates::BlogMetadataTemplate;
use crate::model::templates_admin::AdminBlogsTemplate;
use crate::utils::remove_whitespace;

use askama::Template;
use axum::debug_handler;
use axum::extract::{Query, State};
use axum::response::Html;
use tracing::{debug, error, info};

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
                        id: blog.id.id,
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
