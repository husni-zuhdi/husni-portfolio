pub mod displays;
pub mod operations;

use crate::model::talks::{Talk, TalksParams};
use axum::extract::Query;
use tracing::{debug, warn};
use urlencoding::decode;

/// Initiate pagination check. Set default if not manually requested
fn sanitize_params(params: Query<TalksParams>) -> TalksParams {
    let start = match params.start {
        Some(val) if val >= 0 => val,
        _ => {
            debug!("Set default end to 0");
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
    TalksParams {
        start: Some(start),
        end: Some(end),
    }
}

/// sanitize media and org part of Talk fields
fn sanitize_talk_media_org(talk_data: &Talk) -> (String, String, String) {
    let empty_value = "".to_string();

    let media_link = match &talk_data.media_link {
        Some(val) => val.clone(),
        None => empty_value.clone(),
    };
    let org_name = match &talk_data.org_name {
        Some(val) => val.clone(),
        None => empty_value.clone(),
    };
    let org_link = match &talk_data.org_link {
        Some(val) => val.clone(),
        None => empty_value.clone(),
    };

    (media_link, org_name, org_link)
}

// Take request body String from PUT and POST operations to create a new Talk
fn process_talk_body(body: String) -> Talk {
    // Initialize fields
    let mut talk_id = 0_i64;
    let mut talk_name = String::new();
    let mut talk_media_link = String::new();
    let mut talk_date = String::new();
    let mut talk_org_name = String::new();
    let mut talk_org_link = String::new();

    let req_fields: Vec<&str> = body.split("&").collect();
    for req_field in req_fields {
        let (key, value) = req_field.split_once("=").unwrap();
        let value_decoded = decode(value).unwrap();
        debug!("Request field key/value {:?}/{:?}", key, value_decoded);
        match key {
            "talk_id" => {
                talk_id = value_decoded
                    .parse::<i64>()
                    .expect("Failed to parse path from request body")
            }
            "talk_name" => talk_name = value_decoded.to_string(),
            "talk_media_link" => talk_media_link = value_decoded.to_string(),
            "talk_date" => talk_date = value_decoded.to_string(),
            "talk_org_name" => talk_org_name = value_decoded.to_string(),
            "talk_org_link" => talk_org_link = value_decoded.to_string(),
            _ => {
                warn!("Unrecognized key/value: {:?}/{:?}", key, value_decoded);
                continue;
            }
        }
    }

    Talk {
        id: talk_id,
        name: talk_name,
        date: talk_date,
        media_link: Some(talk_media_link),
        org_name: Some(talk_org_name),
        org_link: Some(talk_org_link),
    }
}
