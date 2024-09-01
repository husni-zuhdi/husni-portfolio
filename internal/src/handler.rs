use crate::model::blog::BlogId;
use crate::model::{axum::AppState, templates::*};
use crate::utils::read_version_manifest;
use askama::Template;
use axum::extract::{Path, State};
use axum::response::Html;
use log::{debug, error, info, warn};

/// Note: In axum [example](https://docs.rs/axum/latest/axum/response/index.html#building-responses)
/// They show an example to return Html<&'static str>
/// Instaed of Html<String>. But using static give me a headache :")

/// get_profile
/// Serve Profile/Biography HTML file
pub async fn get_profile() -> Html<String> {
    let profile = ProfileTemplate.render();
    match profile {
        Ok(res) => {
            info!("Profile askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render profile.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_blogs
/// Serve get_blogs HTML file
/// List our blogs title and id
pub async fn get_blogs(State(app_state): State<AppState>) -> Html<String> {
    // Copy data to Template struct
    let blogs_data = app_state.blog_usecase.blog_repo.find_all();
    let blogs: Vec<BlogTemplate> = blogs_data
        .iter()
        .map(|blog| BlogTemplate {
            id: &blog.id.as_str(),
            name: &blog.name.as_str(),
            filename: &blog.filename.as_str(),
            body: &blog.body.as_str(),
        })
        .collect();
    debug!("Blogs: {:?}", &blogs);

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

/// get_blog
/// Serve get_blog HTML file
/// Render our blog
pub async fn get_blog(Path(path): Path<String>, State(app_state): State<AppState>) -> Html<String> {
    let state = app_state.blog_usecase.blog_repo.find(BlogId(path.clone()));

    let blog = BlogTemplate {
        id: path.clone().as_str(),
        name: &state.name.as_str(),
        filename: &state.filename.as_str(),
        body: &state.body.as_str(),
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

/// get_version
/// Serve get_version HTML file
pub async fn get_version(State(app_state): State<AppState>) -> Html<String> {
    let version_json = read_version_manifest().expect("Failed to get version manifest");
    let version = VersionTemplate {
        version: version_json.version.as_str(),
        environment: app_state.config.environment.as_str(),
        build_hash: version_json.build_hash.as_str(),
        build_date: version_json.build_date.as_str(),
    }
    .render();

    match version {
        Ok(res) => {
            info!("Version askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render version.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_404_not_found
/// Serve 404 Not found HTML file
pub async fn get_404_not_found() -> Html<String> {
    let not_found = NotFoundTemplate.render();
    match not_found {
        Ok(res) => {
            info!("NotFound askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render 404_not_found.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_500_internal_server_error
/// Serve 500 Internal Server Error HTML file
fn get_500_internal_server_error() -> Html<String> {
    let internal_server_error = InternalServerErrorTemplate.render();
    match internal_server_error {
        Ok(res) => {
            info!("InternalServerError askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render 500_internal_server_error.html. {}", err);
            Html("We're fucked up.".to_string())
        }
    }
}
