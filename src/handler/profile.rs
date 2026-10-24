use crate::handler::status::get_500_internal_server_error;
use crate::model::axum::AppState;
use crate::model::profile::{ProfileMeta, ProfilePage};
use crate::model::templates::ProfileTemplate;
use crate::repo::profile::ProfileDisplayRepo;
use askama::Template;
use axum::debug_handler;
use axum::extract::State;
use axum::response::Html;
use tracing::{debug, error, info};

/// get_profile
/// Serve Profile/Biography HTML file
/// The content comes from the Profile Sections in the database (or cache)
#[debug_handler]
pub async fn get_profile(State(app_state): State<AppState>) -> Html<String> {
    let profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    // Get Data from Cache
    let cache_sections = if is_cache_enabled {
        profile_cache_uc_opt.clone().unwrap().find_sections().await
    } else {
        None
    };
    let cache_meta = if is_cache_enabled {
        profile_cache_uc_opt.clone().unwrap().find_meta().await
    } else {
        None
    };

    // If cache hit, use the cached data
    let (sections, meta) = if let (Some(sections), Some(meta)) = (cache_sections, cache_meta) {
        debug!("Profile page cache hit.");
        (sections, meta)
    } else {
        // If not, get data from database
        let db_sections = profile_db_uc.find_sections().await;
        if db_sections.is_none() {
            error!("Failed to find profile sections.");
            return get_500_internal_server_error();
        }
        let db_meta = profile_db_uc.find_meta().await;

        // Insert cache
        if is_cache_enabled {
            debug!("Caching profile page");
            let _ = profile_cache_uc_opt
                .clone()
                .unwrap()
                .profile_operation_repo
                .insert_page(ProfilePage {
                    sections: db_sections.clone().unwrap(),
                    meta: db_meta.clone().unwrap_or(ProfileMeta {
                        id: 0,
                        description: String::new(),
                    }),
                })
                .await;
        }

        (
            db_sections.unwrap(),
            db_meta.unwrap_or(ProfileMeta {
                id: 0,
                description: String::new(),
            }),
        )
    };

    // Render Profile
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
