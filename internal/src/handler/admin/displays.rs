use crate::handler::status::get_500_internal_server_error;
use crate::model::templates_admin::AdminTemplate;
use askama::Template;
use axum::debug_handler;
use axum::response::Html;
use tracing::{error, info};

/// get_base_admin
/// Serve GET (base) admin HTML file
/// Under endpoint /admin
#[debug_handler]
pub async fn get_base_admin() -> Html<String> {
    let admin_res = AdminTemplate {}.render();
    match admin_res {
        Ok(res) => {
            info!("Talks askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render admin/admin.html. {}", err);
            get_500_internal_server_error()
        }
    }
}
