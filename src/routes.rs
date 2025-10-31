use crate::handler::{
    admin::{
        blogs::{
            displays as bd, operations as bo,
            tags::{displays as btd, operations as bto},
        },
        displays as add,
        talks::{displays as td, operations as to},
    },
    auth::{displays as ad, operations as ao},
};
use crate::handler::{blogs, profile, status, talks, version};
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
        .route("/login", get(ad::get_login))
        .route("/login", post(ao::post_login))
        .route("/logout", delete(ao::delete_logout))
        .route("/etc/passwd", get(status::get_418_i_am_a_teapot))
        .nest("/blogs", blogs_route())
        .nest("/talks", talks_route())
        .nest("/admin", admin_route())
        .nest_service("/statics", get_service(ServeDir::new("./statics/favicon/")))
        .nest_service(
            "/theme.js",
            get_service(ServeFile::new("./statics/theme.js")),
        )
        .nest_service(
            "/styles.css",
            get_service(ServeFile::new("./statics/styles.css")),
        )
        .with_state(app_state)
        .layer(ServiceBuilder::new().layer(CompressionLayer::new()))
        .fallback(get(status::get_404_not_found))
}

fn blogs_route() -> Router<AppState> {
    Router::new()
        .route("/", get(blogs::get_blogs))
        .route("/:blog_id", get(blogs::get_blog))
}

fn talks_route() -> Router<AppState> {
    Router::new().route("/", get(talks::get_talks))
}

fn admin_route() -> Router<AppState> {
    Router::new()
        .route("/", get(add::get_base_admin))
        .nest("/talks", admin_talks_route())
        .nest("/blogs", admin_blogs_route())
}

fn admin_talks_route() -> Router<AppState> {
    Router::new()
        .route("/", get(td::get_base_admin_talks))
        .route("/list", get(td::get_admin_talks_list))
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
        .route("/list", get(bd::get_admin_blogs_list))
        .route("/add", get(bd::get_add_admin_blog))
        .route("/add", post(bo::post_add_admin_blog))
        .route("/:blog_id", get(bd::get_admin_blog))
        .route("/:blog_id/edit", get(bd::get_edit_admin_blog))
        .route("/:blog_id/edit", put(bo::put_edit_admin_blog))
        .route("/:blog_id/delete", get(bd::get_delete_admin_blog))
        .route("/:blog_id/delete", delete(bo::delete_delete_admin_blog))
        .nest("/tags", admin_blogs_tags_route())
}

fn admin_blogs_tags_route() -> Router<AppState> {
    Router::new()
        .route("/", get(btd::get_base_admin_tags))
        .route("/list", get(btd::get_admin_tags_list))
        .route("/search", get(btd::get_admin_tags_search))
        .route("/add", get(btd::get_add_admin_tag))
        .route("/add", post(bto::post_add_admin_tag))
        .route("/:tag_id", get(btd::get_admin_tag))
        .route("/:tag_id/edit", get(btd::get_edit_admin_tag))
        .route("/:tag_id/edit", put(bto::put_edit_admin_tag))
        .route("/:tag_id/delete", get(btd::get_delete_admin_tag))
        .route("/:tag_id/delete", delete(bto::delete_delete_admin_tag))
}
