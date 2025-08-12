use crate::handler::{
    admin::blogs::{displays as bd, operations as bo},
    admin::talks::{displays as td, operations as to},
    blogs::{get_blog, get_blogs},
    talks::get_talks,
};
use crate::handler::{profile, status, version};
use crate::model::axum::AppState;
use axum::routing::get_service;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};

pub fn main_route(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(profile::get_profile))
        .route("/version", get(version::get_version))
        .route("/etc/passwd", get(status::get_418_i_am_a_teapot))
        .nest("/blogs", blogs_route())
        .nest("/talks", talks_route())
        .nest("/admin/talks", admin_talks_route())
        .nest("/admin/blogs", admin_blogs_route())
        .nest_service("/statics", get_service(ServeDir::new("./statics/favicon/")))
        .nest_service(
            "/statics/styles.css",
            get_service(ServeFile::new("./statics/styles.css")),
        )
        .with_state(app_state)
        .layer(ServiceBuilder::new().layer(CompressionLayer::new()))
        .fallback(get(status::get_404_not_found))
}

fn blogs_route() -> Router<AppState> {
    Router::new()
        .route("/", get(get_blogs))
        .route("/:blog_id", get(get_blog))
}

fn talks_route() -> Router<AppState> {
    Router::new().route("/", get(get_talks))
}

fn admin_talks_route() -> Router<AppState> {
    Router::new()
        .route("/", get(td::get_base_admin_talks))
        .route("/get", get(td::get_admin_talks))
        .route("/add", get(td::get_add_admin_talk))
        .route("/add", post(to::post_add_admin_talk))
        .route("/:talk_id", get(td::get_admin_talk))
        .route("/:talk_id/edit", get(td::get_edit_admin_talk))
        .route("/:talk_id/edit", put(to::put_edit_admin_talk))
        .route("/:talk_id/delete", get(td::get_delete_admin_talk))
        .route("/:talk_id/delete", delete(to::delete_delete_admin_talk))
}

fn admin_blogs_route() -> Router<AppState> {
    Router::new()
        .route("/", get(bd::get_base_admin_blogs))
        .route("/get", get(bd::get_admin_blogs))
        .route("/add", get(bd::get_add_admin_blog))
        .route("/add", post(bo::post_add_admin_blog))
        .route("/:blog_id", get(bd::get_admin_blog))
        .route("/:blog_id/delete", get(bd::get_delete_admin_blog))
        .route("/:blog_id/delete", delete(bo::delete_delete_admin_blog))
}
