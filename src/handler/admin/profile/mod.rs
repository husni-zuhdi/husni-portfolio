pub mod displays;
pub mod operations;

use crate::model::profile::{Layout, ProfileMeta, ProfileSection};
use std::str::FromStr;
use tracing::{debug, warn};
use urlencoding::decode;

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
        // Tolerate malformed percent-encoding like the blog form parser:
        // a bad escape must not crash the request.
        let value_decoded = decode(value).unwrap_or_default();
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
        // Tolerate malformed percent-encoding like the blog form parser:
        // a bad escape must not crash the request.
        let value_decoded = decode(value).unwrap_or_default();
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
    fn test_process_profile_section_body_tolerates_malformed_encoding() {
        let body =
            "section_id=1&section_title=A%zzT&section_layout=stacked&section_body=B".to_string();
        let section = process_profile_section_body(body).unwrap();
        // urlencoding decodes invalid escapes leniently: the bad escape
        // passes through instead of crashing the request.
        assert_eq!(section.title, "A%zzT");
        assert_eq!(section.body, "B", "later fields must still decode");
    }

    #[test]
    fn test_process_profile_meta_body_tolerates_malformed_encoding() {
        let body = "meta_id=1&meta_description=%zzD".to_string();
        let meta = process_profile_meta_body(body).unwrap();
        assert_eq!(meta.description, "%zzD");
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
