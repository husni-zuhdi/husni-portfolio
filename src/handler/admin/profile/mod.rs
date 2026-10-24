pub mod displays;
pub mod operations;

use crate::model::axum::AppState;
use crate::model::profile::{Layout, ProfileMeta, ProfilePage, ProfileSection};
use crate::repo::profile::ProfileDisplayRepo;
use std::str::FromStr;
use tracing::{debug, warn};
use urlencoding::decode;

/// Rebuild the profile page cache from the database after a mutation:
/// invalidate the cached page, fetch sections and meta from the database,
/// then insert the fresh page.
pub async fn rebuild_profile_page_cache(app_state: &AppState) {
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();
    if !is_cache_enabled {
        return;
    }

    let profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let _ = profile_cache_uc_opt
        .clone()
        .unwrap()
        .profile_operation_repo
        .invalidate_page()
        .await;

    let sections = profile_db_uc.find_sections().await;
    let meta = profile_db_uc.find_meta().await;
    if let (Some(sections), Some(meta)) = (sections, meta) {
        debug!("Re-caching profile page");
        let _ = profile_cache_uc_opt
            .clone()
            .unwrap()
            .profile_operation_repo
            .insert_page(ProfilePage { sections, meta })
            .await;
    } else {
        warn!("Failed to rebuild profile page cache. Database data are empty.");
    }
}

/// Take request body String from PUT and POST operations to create
/// a Profile Section
pub fn process_profile_section_body(body: String) -> Option<ProfileSection> {
    // Initialize fields
    let mut section_id = 0_i64;
    let mut section_title = String::new();
    let mut section_layout = String::new();
    let mut section_body = String::new();

    let req_fields: Vec<&str> = body.split("&").collect();
    for req_field in req_fields {
        let Some((key, value)) = req_field.split_once("=") else {
            warn!("Malformed request field {}", req_field);
            return None;
        };
        let value_decoded = decode(value).unwrap();
        debug!("Request field key/value {:?}/{:?}", key, value_decoded);
        match key {
            "section_id" => {
                let section_id_res = value_decoded.parse::<i64>();
                match section_id_res {
                    Ok(val) => section_id = val,
                    Err(err) => {
                        warn!("Failed to parse section_id with error, {err}");
                        return None;
                    }
                }
            }
            "section_title" => section_title = value_decoded.to_string(),
            "section_layout" => section_layout = value_decoded.to_string(),
            "section_body" => section_body = value_decoded.to_string(),
            _ => {
                warn!("Unrecognized key/value: {:?}/{:?}", key, value_decoded);
            }
        }
    }

    // Unknown layout values fall back to `Stacked`
    let layout = match Layout::from_str(&section_layout) {
        Ok(layout) => layout,
        Err(_) => {
            warn!(
                "Unknown section layout {}. Default to stacked.",
                &section_layout
            );
            Layout::Stacked
        }
    };

    Some(ProfileSection {
        id: section_id,
        // Position is determined by the repo on insertion
        position: 0,
        title: section_title,
        body: section_body,
        layout,
    })
}

/// Take request body String from PUT to update the Profile Meta
pub fn process_profile_meta_body(body: String) -> Option<ProfileMeta> {
    let mut meta_id = 1_i64;
    let mut description = String::new();

    let req_fields: Vec<&str> = body.split("&").collect();
    for req_field in req_fields {
        let Some((key, value)) = req_field.split_once("=") else {
            warn!("Malformed request field {}", req_field);
            return None;
        };
        let value_decoded = decode(value).unwrap();
        debug!("Request field key/value {:?}/{:?}", key, value_decoded);
        match key {
            "meta_id" => {
                let meta_id_res = value_decoded.parse::<i64>();
                match meta_id_res {
                    Ok(val) => meta_id = val,
                    Err(err) => {
                        warn!("Failed to parse meta_id with error, {err}");
                        return None;
                    }
                }
            }
            "meta_description" => description = value_decoded.to_string(),
            _ => {
                warn!("Unrecognized key/value: {:?}/{:?}", key, value_decoded);
            }
        }
    }

    Some(ProfileMeta {
        id: meta_id,
        description,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_process_profile_section_body() {
        let body =
            "section_id=7&section_title=Overview&section_layout=stacked&section_body=Hello%20World"
                .to_string();
        let section = process_profile_section_body(body).unwrap();
        assert_eq!(section.id, 7);
        assert_eq!(section.title, "Overview");
        assert_eq!(section.body, "Hello World");
        assert_eq!(section.layout, Layout::Stacked);
        assert_eq!(section.position, 0);
    }

    #[test]
    fn test_process_profile_section_body_cards_layout() {
        let body =
            "section_id=1&section_title=&section_layout=cards&section_body=Some%20text".to_string();
        let section = process_profile_section_body(body).unwrap();
        assert_eq!(section.id, 1);
        assert_eq!(section.title, "");
        assert_eq!(section.body, "Some text");
        assert_eq!(section.layout, Layout::Cards);
    }

    #[test]
    fn test_process_profile_section_body_unknown_layout_defaults_to_stacked() {
        let body = "section_id=1&section_title=T&section_layout=fancy&section_body=B".to_string();
        let section = process_profile_section_body(body).unwrap();
        assert_eq!(section.layout, Layout::Stacked);
    }

    #[test]
    fn test_process_profile_section_body_malformed_field() {
        let body = "section_id".to_string();
        assert!(process_profile_section_body(body).is_none());
    }

    #[test]
    fn test_process_profile_section_body_invalid_id() {
        let body =
            "section_id=abc&section_title=T&section_layout=stacked&section_body=B".to_string();
        assert!(process_profile_section_body(body).is_none());
    }

    #[test]
    fn test_process_profile_meta_body() {
        let body = "meta_id=1&meta_description=Husni%20Zuhdi%20Profile".to_string();
        let meta = process_profile_meta_body(body).unwrap();
        assert_eq!(meta.id, 1);
        assert_eq!(meta.description, "Husni Zuhdi Profile");
    }

    #[test]
    fn test_process_profile_meta_body_invalid_id() {
        let body = "meta_id=x&meta_description=D".to_string();
        assert!(process_profile_meta_body(body).is_none());
    }
}
