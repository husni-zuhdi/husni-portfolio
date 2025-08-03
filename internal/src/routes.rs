use crate::handler::{admin, blogs, talks};
use crate::model::axum::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn blogs_route() -> Router<AppState> {
    Router::new()
        .route("/", get(blogs::get_blogs))
        .route("/:blog_id", get(blogs::get_blog))
}

pub fn talks_route() -> Router<AppState> {
    Router::new().route("/", get(talks::get_talks))
}

pub fn admin_talks_route() -> Router<AppState> {
    Router::new()
        .route("/", get(admin::talks::displays::get_base_admin_talks))
        .route("/get", get(admin::talks::displays::get_admin_talks))
        .route("/add", get(admin::talks::displays::get_add_admin_talk))
        .route("/add", post(admin::talks::operations::post_add_admin_talk))
        .route("/:talk_id", get(admin::talks::displays::get_admin_talk))
        .route(
            "/:talk_id/edit",
            get(admin::talks::displays::get_edit_admin_talk),
        )
        .route(
            "/:talk_id/edit",
            put(admin::talks::operations::put_edit_admin_talk),
        )
        .route(
            "/:talk_id/delete",
            get(admin::talks::displays::get_delete_admin_talk),
        )
        .route(
            "/:talk_id/delete",
            delete(admin::talks::operations::delete_delete_admin_talk),
        )
}
