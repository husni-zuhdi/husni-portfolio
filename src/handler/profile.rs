use crate::handler::status::get_500_internal_server_error;
use crate::model::axum::AppState;
use crate::model::profile::ProfileMeta;
use crate::model::templates::ProfileTemplate;
use askama::Template;
use axum::debug_handler;
use axum::extract::State;
use axum::response::Html;
use tracing::{error, info};

/// get_profile
/// Serve Profile/Biography HTML file
/// The content comes from the Profile Sections in the database (or cache)
#[debug_handler]
pub async fn get_profile(State(app_state): State<AppState>) -> Html<String> {
    let mut profile_uc = app_state.profile_usecase.lock().await.clone();

    // The sections read is the cache unit: it answers cache-only when the
    // page is warm, otherwise it falls through and rebuilds the page cache.
    let sections = profile_uc.find_sections().await;
    if sections.is_none() {
        error!("Failed to find profile sections.");
        return get_500_internal_server_error();
    }
    let meta = profile_uc.find_meta().await.unwrap_or(ProfileMeta {
        id: 0,
        description: String::new(),
    });

    // Render Profile
    let sections = sections.unwrap();
    let profile_tmpl = ProfileTemplate {
        page_title: sections.page_title(),
        description: meta.description,
        sections: sections.to_template(),
    };
    let profile_res = profile_tmpl.render();
    match profile_res {
        Ok(res) => {
            info!("Profile askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render profile.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Config;
    use crate::state::state_factory;

    #[tokio::test]
    async fn test_get_profile_renders_seeded_sections() {
        let state = state_factory(Config::default()).await;
        let html = get_profile(State(state)).await;
        assert!(html
            .0
            .contains("Husni Naufal Zuhdi - Site Reliability Engineer"));
        assert!(html.0.contains("Work Experiences"));
        assert!(html.0.contains("Overview"));
    }

    #[tokio::test]
    async fn test_get_profile_renders_title_and_description() {
        let state = state_factory(Config::default()).await;
        let html = get_profile(State(state)).await;
        assert!(html
            .0
            .contains("<h1>Husni Naufal Zuhdi - Site Reliability Engineer</h1>"));
        assert!(html.0.contains("content=\"Husni Zuhdi Profile\""));
    }
}
