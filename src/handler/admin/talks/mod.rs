pub mod displays;
pub mod operations;

use crate::model::talks::Talk;
use tracing::{debug, warn};
use urlencoding::decode;

// Take request body String from PUT and POST operations to create a new Talk
fn process_talk_body(body: String) -> Option<Talk> {
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
                let talk_id_res = value_decoded.parse::<i64>();
                match talk_id_res {
                    Ok(val) => talk_id = val,
                    Err(err) => {
                        warn!("Failed to parse talk_id with error, {err}");
                        return None;
                    }
                }
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

    Some(Talk {
        id: talk_id,
        name: talk_name,
        date: talk_date,
        media_link: Some(talk_media_link),
        org_name: Some(talk_org_name),
        org_link: Some(talk_org_link),
    })
}
