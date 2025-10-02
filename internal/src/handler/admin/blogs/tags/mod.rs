pub mod displays;
pub mod operations;

use crate::model::tags::Tag;
use tracing::{debug, warn};
use urlencoding::decode;
// Take request tag String from PUT and POST operations to create a new tag
fn process_tag_body(body: String) -> Tag {
    // Initialize fields
    let mut tag_id = 0_i64;
    let mut tag_name = String::new();

    let req_fields: Vec<&str> = body.split("&").collect();
    for req_field in req_fields {
        let (key, value) = req_field.split_once("=").unwrap();
        let value_decoded = decode(value).unwrap();
        debug!("Request field key/value {:?}/{:?}", key, value_decoded);
        match key {
            "tag_id" => {
                tag_id = value_decoded
                    .parse::<i64>()
                    .expect("Failed to parse path from request body")
            }
            "tag_name" => tag_name = value_decoded.to_string(),
            _ => {
                warn!("Unrecognized key/value: {:?}/{:?}", key, value_decoded);
                continue;
            }
        }
    }

    Tag {
        id: tag_id,
        name: tag_name,
    }
}
