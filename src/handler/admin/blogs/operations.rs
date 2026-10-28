use crate::handler::admin::blogs::displays::get_admin_blogs_list;
use crate::handler::admin::blogs::process_blog_body;
use crate::handler::auth::csrf::verify_csrf_token;
use crate::handler::auth::is_auth_verified;
use crate::handler::status::{
    get_401_unauthorized, get_404_not_found, get_500_internal_server_error,
};
use crate::model::axum::AppState;
use crate::model::blogs::{BlogCommandStatus, BlogsParams};
use crate::model::templates_admin::AdminBlogPreviewTemplate;
use crate::utils::convert_markdown_to_html;
use askama::Template;
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use tracing::{error, info};
use urlencoding::decode;

/// post_admin_blog_preview
/// Serve POST preview blog content endpoint.
/// Renders the markdown body without touching the database.
#[debug_handler]
pub async fn post_admin_blog_preview(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let mut blog_body = String::new();
    for req_field in body.split('&') {
        let Some((key, value)) = req_field.split_once('=') else {
            continue;
        };
        if key == "blog_body" {
            blog_body = decode(value).unwrap_or_default().to_string();
            break;
        }
    }

    let preview = AdminBlogPreviewTemplate {
        html: convert_markdown_to_html(&blog_body),
    }
    .render();

    match preview {
        Ok(html) => {
            info!("AdminBlogPreview askama template rendered.");
            Html(html)
        }
        Err(err) => {
            error!("Failed to render admin/blogs/preview.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// post_add_admin_blog
/// Serve POST add blog endpoint
#[debug_handler]
pub async fn post_add_admin_blog(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let blog = process_blog_body(body);

    // The usecase stores the Blog, refreshes the Blog cache, ensures the
    // selected Tags exist, and adds one Blog tag mapping per Tag.
    let add_result = {
        let mut blog_uc = app_state.blog_usecase.lock().await.clone();
        blog_uc.add(blog).await
    };

    match add_result {
        Some(BlogCommandStatus::Stored) => {}
        Some(_) => {
            error!("Adding the Blog returned an unexpected command status");
            return get_500_internal_server_error();
        }
        None => {
            return get_404_not_found().await;
        }
    }

    let query_params = BlogsParams {
        start: None,
        end: None,
        tags: None,
    };

    get_admin_blogs_list(State(app_state), headers, Query(query_params)).await
}

/// put_edit_admin_blog
/// Serve PUT edit blog HTML file
#[debug_handler]
pub async fn put_edit_admin_blog(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    // Sanitize `path`
    let Ok(_id) = path.parse::<i64>() else {
        return get_404_not_found().await;
    };

    // The Blog target id comes from the form body, mirroring the original
    // edit flow; the parsed path stays a sanity gate.
    let blog = process_blog_body(body);

    // The usecase updates the Blog, refreshes the Blog cache, ensures the
    // selected Tags exist, and syncs the Blog tag mappings to the tags.
    let edit_result = {
        let mut blog_uc = app_state.blog_usecase.lock().await.clone();
        blog_uc.update(blog).await
    };

    match edit_result {
        Some(BlogCommandStatus::Updated) => {}
        Some(_) | None => {
            return get_404_not_found().await;
        }
    }

    let query_params = BlogsParams {
        start: None,
        end: None,
        tags: None,
    };

    get_admin_blogs_list(State(app_state), headers, Query(query_params)).await
}

/// delete_delete_admin_blog
/// Serve DELETE delete blog HTML file
#[debug_handler]
pub async fn delete_delete_admin_blog(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        return get_404_not_found().await;
    };

    // The usecase deletes the Blog, drops the Blog cache, and deletes the
    // Blog's Blog tag mappings.
    let delete_result = {
        let mut blog_uc = app_state.blog_usecase.lock().await.clone();
        blog_uc.delete(id).await
    };

    match delete_result {
        Some(BlogCommandStatus::Deleted) => {}
        Some(_) | None => {
            return get_500_internal_server_error();
        }
    }

    let query_params = BlogsParams {
        start: None,
        end: None,
        tags: None,
    };

    get_admin_blogs_list(State(app_state), headers, Query(query_params)).await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Config;
    use crate::state::state_factory;

    fn unauthorized_html(html: &Html<String>) -> bool {
        html.0.contains("401 Unauthorized")
    }

    #[tokio::test]
    async fn test_post_admin_blog_preview_unauthorized() {
        let state = state_factory(Config::default()).await;
        let headers = HeaderMap::new();
        let body = "blog_body=%23%20Hello".to_string();
        let result = post_admin_blog_preview(State(state), headers, body).await;
        assert!(unauthorized_html(&result));
    }
}
