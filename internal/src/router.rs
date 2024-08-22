use crate::config::Config;
use crate::model::{data::*, templates::*};
use crate::utils::read_version_manifest;
use actix_files::NamedFile;
use actix_web::{web, Responder, Result};
// use actix_web_lab::respond::Html;
use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use log::{debug, error, info};
use tower_http::services::ServeFile;

/// Note: In axum [example](https://docs.rs/axum/latest/axum/response/index.html#building-responses)
/// They show an example to return Html<&'static str>
/// Instaed of Html<String>. But using static give me a headache :")

/// get_profile
/// Serve Profile/Biography HTML file
pub async fn get_profile() -> Html<String> {
    let profile = Profile.render().expect("Failed to render profile.html");
    Html(profile)
}

// get_404_not_found
// Serve 404 Not found HTML file
pub async fn get_404_not_found() -> Html<String> {
    let html = NotFound.render().expect("Failed to render not_found.html");
    Html(html)
}

pub async fn get_blogs(State(app_state): State<AppState>) -> Html<String> {
    // Copy data to Template struct
    let blogs_template: Vec<Blog> = app_state
        .blogs_data
        .blogs
        .iter()
        .map(|blog| Blog {
            id: &blog.id,
            name: &blog.name,
            filename: &blog.filename,
            body: &blog.body,
        })
        .collect();

    let blogs = Blogs {
        blogs: &blogs_template,
    }
    .render()
    .expect("Failed to render blogs.html");
    info!("Blogs Template created");
    Html(blogs)
}

pub async fn get_blog(Path(path): Path<String>, State(app_state): State<AppState>) -> Html<String> {
    let blog_id = path;
    let blog_data = app_state
        .blogs_data
        .blogs
        .iter()
        .filter(|blog| blog.id == blog_id)
        .next()
        .expect("Failed to get blog name with id {blog_id}");

    let blog = Blog {
        id: &blog_id,
        name: &blog_data.name,
        filename: &blog_data.filename,
        body: &blog_data.body,
    }
    .render()
    .expect("Failed to render blog.html");
    Html(blog)
}

pub async fn get_version(State(app_state): State<AppState>) -> Html<String> {
    let version_json = read_version_manifest().expect("Failed to get version manifest");
    let version = Version {
        version: version_json.version.as_str(),
        environment: app_state.config.environment.as_str(),
        build_hash: version_json.build_hash.as_str(),
        build_date: version_json.build_date.as_str(),
    }
    .render()
    .expect("Failed to render version.html");
    info!("Version Template created");

    Html(version)
}
