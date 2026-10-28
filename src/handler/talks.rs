use crate::handler::status::get_500_internal_server_error;
use askama::Template;
use axum::response::Html;

use crate::model::axum::AppState;
use crate::model::talks::TalksParams;
use axum::debug_handler;
use axum::extract::{Query, State};
use tracing::{error, info};

/// get_talks
/// Serve talks HTML file
#[debug_handler]
pub async fn get_talks(
    State(app_state): State<AppState>,
    params: Query<TalksParams>,
) -> Html<String> {
    let mut talk_uc = app_state.talk_usecase.lock().await.clone();

    // Sanitize Params
    let sanitized_params = params.sanitize();
    let talks_result = talk_uc.find_talks(sanitized_params.clone()).await;

    // Early check result. If empty, return 500 error
    let Some(talks_data) = talks_result else {
        error!(
            "Failed to find talks with Talk Id started at {} and ended at {}.",
            sanitized_params.start.unwrap(),
            sanitized_params.end.unwrap()
        );
        return get_500_internal_server_error();
    };

    // Render Talks
    let talks_res = match talks_data.sanitize().to_template().render() {
        Ok(html) => html,
        Err(err) => {
            error!("Failed to render get_talks.html. {}", err);
            return get_500_internal_server_error();
        }
    };
    info!("Talks askama template rendered.");
    Html(talks_res)
}
