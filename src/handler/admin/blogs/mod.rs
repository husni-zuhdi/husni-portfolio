pub mod displays;
pub mod operations;
pub mod tags;

use crate::model::blogs::Blog;
use crate::utils::remove_whitespace;
use tracing::{debug, warn};
use urlencoding::decode;

// Take request body String from PUT and POST operations to create a new blog
fn process_blog_body(body: String) -> Blog {
    // Initialize fields
    let mut blog_id = 0_i64;
    let mut blog_name = String::new();
    let mut blog_body = String::new();
    let mut blog_tags = vec![String::new()];

    let req_fields: Vec<&str> = body.split("&").collect();
    for req_field in req_fields {
        let (key, value) = req_field.split_once("=").unwrap();
        let value_decoded = decode(value).unwrap();
        debug!("Request field key/value {:?}/{:?}", key, value_decoded);
        match key {
            "blog_id" => {
                blog_id = value_decoded
                    .parse::<i64>()
                    .expect("Failed to parse path from request body")
            }
            "blog_name" => blog_name = value_decoded.to_string(),
            "blog_body" => blog_body = value_decoded.to_string(),
            "blog_tag" => {
                let clean_tag = remove_whitespace(&value_decoded);
                blog_tags.push(clean_tag);
            }
            _ => {
                warn!("Unrecognized key/value: {:?}/{:?}", key, value_decoded);
                continue;
            }
        }
    }

    Blog {
        id: blog_id,
        name: Some(blog_name),
        body: Some(blog_body),
        tags: Some(blog_tags),
        source: None,
        filename: None,
    }
}
