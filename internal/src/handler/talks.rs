use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use askama::Template;
use axum::response::Html;

use crate::model::talks::{TalkEndPage, TalkPagination, TalkStartPage};
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
    pagination: Query<TalkPagination>,
) -> Html<String> {
    match app_state.talk_usecase.lock().await.clone() {
        Some(data) => {
            // Setup Pagination
            debug!("Pagination {:?}", &pagination);
            let start = match pagination.0.start {
                Some(val) => val,
                None => {
                    debug!("Set default start to 0");
                    TalkStartPage(0)
                }
            };
            let end = match pagination.0.end {
                Some(val) => val,
                None => {
                    debug!("Set default end to 10");
                    TalkEndPage(10)
                }
            };

            // Construct TalksTemplate Struct
            let empty_value = "";
            let result = data.talk_repo.find_talks(start.clone(), end.clone()).await;
            match result {
                Some(talks_data) => {
                    let talks: Vec<TalkTemplate> = talks_data
                        .iter()
                        .map(|talk| {
                            debug!("Construct TalkTemplate for Talk Id {}", &talk.id);
                            debug!("TalkTemplate {:?}", &talk);
                            let media_link = match &talk.media_link {
                                Some(val) => val.as_str(),
                                None => empty_value,
                            };
                            let org_name = match &talk.org_name {
                                Some(val) => val.as_str(),
                                None => empty_value,
                            };
                            let org_link = match &talk.org_link {
                                Some(val) => val.as_str(),
                                None => empty_value,
                            };
                            TalkTemplate {
                                id: &talk.id.id,
                                name: &talk.name.as_str(),
                                date: &talk.date.as_str(),
                                media_link: &media_link,
                                org_name: &org_name,
                                org_link: &org_link,
                            }
                        })
                        .collect();
                    debug!("TalksTemplate talks : {:?}", &talks);

                    let talks_res = TalksTemplate { talks: &talks }.render();
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
                        &start.0, &end.0
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
