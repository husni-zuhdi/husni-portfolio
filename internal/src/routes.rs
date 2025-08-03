use crate::handler::{
    admin::talks::{displays, operations},
    blogs::{get_blog, get_blogs},
    talks::get_talks,
};
use crate::model::axum::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn blogs_route() -> Router<AppState> {
    Router::new()
        .route("/", get(get_blogs))
        .route("/:blog_id", get(get_blog))
}

pub fn talks_route() -> Router<AppState> {
    Router::new().route("/", get(get_talks))
}

pub fn admin_talks_route() -> Router<AppState> {
    Router::new()
        .route("/", get(displays::get_base_admin_talks))
        .route("/get", get(displays::get_admin_talks))
        .route("/add", get(displays::get_add_admin_talk))
        .route("/add", post(operations::post_add_admin_talk))
        .route("/:talk_id", get(displays::get_admin_talk))
        .route("/:talk_id/edit", get(displays::get_edit_admin_talk))
        .route("/:talk_id/edit", put(operations::put_edit_admin_talk))
        .route("/:talk_id/delete", get(displays::get_delete_admin_talk))
        .route(
            "/:talk_id/delete",
            delete(operations::delete_delete_admin_talk),
        )
}
