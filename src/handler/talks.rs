use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use askama::Template;
use axum::response::Html;

use crate::model::talks::TalksParams;
use crate::model::{
    axum::AppState,
    templates::{TalkTemplate, TalksTemplate},
};
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
    match app_state.talk_db_usecase.lock().await.clone() {
        Some(data) => {
            // Setup Pagination
            debug!("Params {:?}", &params);
            let start = match params.start {
                Some(val) => val,
                None => {
                    debug!("Set default start to 0");
                    0_i64
                }
            };
            let end = match params.end {
                Some(val) => val,
                None => {
                    debug!("Set default end to 10");
                    10_i64
                }
            };

            // Construct TalksTemplate Struct
            let empty_value = "".to_string();
            let result = data
                .talk_display_repo
                .find_talks(TalksParams {
                    start: Some(start),
                    end: Some(end),
                })
                .await;
            match result {
                Some(talks_data) => {
                    let talks: Vec<TalkTemplate> = talks_data
                        .talks
                        .iter()
                        .map(|talk| {
                            debug!("Construct TalkTemplate for Talk Id {}", &talk.id);
                            debug!("TalkTemplate {:?}", &talk);
                            let media_link = match &talk.media_link {
                                Some(val) => val.clone(),
                                None => empty_value.clone(),
                            };
                            let org_name = match &talk.org_name {
                                Some(val) => val.clone(),
                                None => empty_value.clone(),
                            };
                            let org_link = match &talk.org_link {
                                Some(val) => val.clone(),
                                None => empty_value.clone(),
                            };
                            TalkTemplate {
                                id: talk.id,
                                name: talk.name.clone(),
                                date: talk.date.clone(),
                                media_link,
                                org_name,
                                org_link,
                            }
                        })
                        .collect();
                    debug!("TalksTemplate talks : {:?}", &talks);

                    let talks_res = TalksTemplate { talks }.render();
                    match talks_res {
                        Ok(res) => {
                            info!("Talks askama template rendered.");
                            Html(res)
                        }
                        Err(err) => {
                            error!("Failed to render get_talks.html. {}", err);
                            get_500_internal_server_error()
                        }
                    }
                }
                None => {
                    error!(
                        "Failed to find talks with Talk Id started at {} and ended at {}.",
                        start, end
                    );
                    get_500_internal_server_error()
                }
            }
        }
        None => {
            // No Talks in Memory yet and I don't think it's worth to be implement
            // Andd I also think to remove the memory database at all
            get_404_not_found().await
        }
    }
}
