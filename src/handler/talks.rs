use crate::handler::status::get_500_internal_server_error;
use crate::repo::talks::TalkDisplayRepo;
use askama::Template;
use axum::response::Html;

use crate::model::axum::AppState;
use crate::model::talks::TalksParams;
use axum::debug_handler;
use axum::extract::{Query, State};
use tracing::{debug, error, info};

/// get_talks
/// Serve talks HTML file
#[debug_handler]
pub async fn get_talks(
    State(app_state): State<AppState>,
    params: Query<TalksParams>,
) -> Html<String> {
    // Setup usecases
    let talk_db_uc = app_state
        .talk_db_usecase
        .lock()
        .await
        .clone()
        .expect("Failed to lock Talk DB Usecase");
    let talk_cache_uc_opt = app_state.talk_cache_usecase.lock().await;
    let cache_is_enabled = talk_cache_uc_opt.is_some();

    // Setup Pagination
    debug!("Params {:?}", &params);
    let start = match params.start {
        Some(val) if val >= 0 => val,
        _ => {
            debug!("Set default start to 0");
            0_i64
        }
    };
    let end = match params.end {
        Some(val) if val >= 0 => val,
        _ => {
            debug!("Set default end to 10");
            10_i64
        }
    };
    let params = TalksParams {
        start: Some(start),
        end: Some(end),
    };

    // Get Data from Cache
    let cache_result = if cache_is_enabled {
        talk_cache_uc_opt
            .clone()
            .unwrap()
            .find_talks(params.clone())
            .await
    } else {
        None
    };
    // If cache hit, return early
    if let Some(res) = cache_result {
        let talks_res = res.to_template().render();
        if talks_res.is_err() {
            error!(
                "Failed to render get_talks.html. {}",
                talks_res.unwrap_err()
            );
            return get_500_internal_server_error();
        }

        info!("Talks askama template rendered.");
        return Html(talks_res.unwrap());
    }
    // If not, get data from database
    let db_result = talk_db_uc
        .talk_display_repo
        .find_talks(params.clone())
        .await;

    // Early check db result. If empty, return 500 error
    if db_result.is_none() {
        error!(
            "Failed to find talks with Talk Id started at {} and ended at {}.",
            start, end
        );
        return get_500_internal_server_error();
    }

    // Insert cache
    if cache_is_enabled {
        for talk in db_result.clone().unwrap().talks {
            debug!("Caching talk {}", &talk.id);
            let _ = talk_cache_uc_opt
                .clone()
                .unwrap()
                .talk_operation_repo
                .insert(talk)
                .await;
        }
    }

    // Render Talks
    let talks_res = db_result.unwrap().to_template().render();
    if talks_res.is_err() {
        error!(
            "Failed to render get_talks.html. {}",
            talks_res.unwrap_err()
        );
        return get_500_internal_server_error();
    }
    info!("Talks askama template rendered.");
    Html(talks_res.unwrap())
}
