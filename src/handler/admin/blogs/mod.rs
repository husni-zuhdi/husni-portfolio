pub mod displays;
pub mod operations;
pub mod tags;

use crate::model::blogs::Blog;
use crate::utils::remove_whitespace;
use tracing::{debug, warn};
use urlencoding::decode;

/// Take request body String from PUT and POST operations to create a new blog.
///
/// Decodes URL-encoded form fields and collects multiple `blog_tag` values.
/// Empty tag values are ignored.
fn process_blog_body(body: String) -> Blog {
    let mut blog_id = 0_i64;
    let mut blog_name = String::new();
    let mut blog_body = String::new();
    let mut blog_tags = Vec::new();

    for req_field in body.split('&') {
        let Some((key, value)) = req_field.split_once('=') else {
            warn!("Skipping request field without '=': {:?}", req_field);
            continue;
        };
        let value_decoded = decode(value).unwrap_or_default();
        debug!("Request field key/value {:?}/{:?}", key, value_decoded);
        match key {
            "blog_id" => {
                blog_id = value_decoded
                    .parse::<i64>()
                    .expect("Failed to parse path from request body");
            }
            "blog_name" => blog_name = value_decoded.to_string(),
            "blog_body" => blog_body = value_decoded.to_string(),
            "blog_tag" => {
                let clean_tag = remove_whitespace(&value_decoded);
                if !clean_tag.is_empty() && !blog_tags.contains(&clean_tag) {
                    blog_tags.push(clean_tag);
                }
            }
            _ => {
                warn!("Unrecognized key/value: {:?}/{:?}", key, value_decoded);
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_process_blog_body_parses_fields_and_tags() {
        let body = "blog_id=42&blog_name=Hello%20World&blog_body=This%20is%20a%20body&blog_tag=rust&blog_tag=go".to_string();
        let blog = process_blog_body(body);
        assert_eq!(blog.id, 42);
        assert_eq!(blog.name, Some("Hello World".to_string()));
        assert_eq!(blog.body, Some("This is a body".to_string()));
        assert_eq!(blog.tags, Some(vec!["rust".to_string(), "go".to_string()]));
    }

    #[test]
    fn test_process_blog_body_ignores_empty_tags() {
        let body =
            "blog_id=1&blog_name=Test&blog_body=Body&blog_tag=&blog_tag=rust&blog_tag=".to_string();
        let blog = process_blog_body(body);
        assert_eq!(blog.tags, Some(vec!["rust".to_string()]));
    }

    #[test]
    fn test_process_blog_body_deduplicates_tags() {
        let body =
            "blog_id=1&blog_name=Test&blog_body=Body&blog_tag=rust&blog_tag=rust".to_string();
        let blog = process_blog_body(body);
        assert_eq!(blog.tags, Some(vec!["rust".to_string()]));
    }

    #[test]
    fn test_process_blog_body_handles_missing_equals() {
        let body = "blog_id=1&blog_name=Test&orphan&blog_body=Body".to_string();
        let blog = process_blog_body(body);
        assert_eq!(blog.id, 1);
        assert_eq!(blog.name, Some("Test".to_string()));
        assert_eq!(blog.body, Some("Body".to_string()));
    }

    #[test]
    fn test_process_blog_body_handles_no_tags() {
        let body = "blog_id=1&blog_name=Test&blog_body=Body".to_string();
        let blog = process_blog_body(body);
        assert_eq!(blog.tags, Some(vec![]));
    }
}
