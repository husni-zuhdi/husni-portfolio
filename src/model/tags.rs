use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::model::templates_admin::{AdminBlogTagsListTemplate, AdminGetTagTemplate};

/// Tag
/// Just tag id and it's name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

impl Tag {
    /// Convert Tag to (Askama) AdminTagTemplate
    pub fn to_admin_template(&self) -> AdminGetTagTemplate {
        debug!(
            "Tag: Constructing AdminGetTagTemplate for Tag Id {}",
            self.id
        );
        AdminGetTagTemplate {
            id: self.id,
            name: self.name.clone(),
        }
    }
    /// Calculate size of Tag in u32
    /// Useful for weighing data size
    pub const fn data_size(&self) -> u32 {
        (size_of_val(&self.id) + size_of_val(&self.name)) as u32
    }
}

/// Tags
/// Vector of tag id and it's name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tags {
    pub tags: Vec<Tag>,
}

impl Tags {
    /// Convert Talks to (Askama) AdminListTalksTemplate
    pub fn to_admin_list_template(&self) -> AdminBlogTagsListTemplate {
        AdminBlogTagsListTemplate {
            tags: self.tags.clone(),
        }
    }
    /// Convert Tags to Vector of String
    pub fn to_vector_string(&self) -> Vec<String> {
        self.tags.iter().map(|tag| tag.name.clone()).collect()
    }
    /// Convert Tags to String separated by comma
    pub fn to_formatted_string(&self) -> String {
        self.to_vector_string().join(", ")
    }
}

/// TagCommandStatus
/// Status of Tag Command Operations:
/// - Stored
/// - Updated
/// - Deleted
/// - CacheInserted
/// - CacheInvalidated
///
/// I think you should wrap this with Option so you can check if it `None`
/// then check the value of the status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TagCommandStatus {
    Stored,
    Updated,
    Deleted,
    CacheInserted,
    CacheInvalidated,
}

/// TagsParams
/// Axum Query struct for `/admin/blogs/tags/list` query parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagsListParams {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

impl TagsListParams {
    /// Sanitize TagListParams by checking negative value and set unknown to
    /// the default values
    pub fn sanitize(&self) -> Self {
        let start = match self.start {
            Some(val) if val >= 0 => val,
            _ => {
                debug!("TagsListParams: set default start to 0");
                0_i64
            }
        };
        let end = match self.end {
            Some(val) if val >= 0 => val,
            _ => {
                debug!("TagsListParams: set default end to 100");
                100_i64
            }
        };

        Self {
            start: Some(start),
            end: Some(end),
        }
    }
}

/// TagsSearchParams
/// Axum Query struct for `/admin/blogs/tags/search` query parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagsSearchParams {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub query: String,
}

impl TagsSearchParams {
    /// Sanitize TagSearchParams by checking negative value and set unknown to
    /// the default values
    pub fn sanitize(&self) -> Self {
        let start = self.start.unwrap_or(0);
        let end = self.end.unwrap_or(100);

        let sanitized_query = self.sanitize_query();
        if sanitized_query != self.query {
            warn!(
                "Query {} contain non-alphanumeric, dash, and whitespace chars",
                self.query
            );
        }
        Self {
            start: Some(start),
            end: Some(end),
            query: sanitized_query,
        }
    }
    /// Sanitize query of TagsSearchParams
    fn sanitize_query(&self) -> String {
        let pattern = Regex::new(r"[^a-zA-Z0-9\s]+").unwrap();
        pattern.replace(&self.query, "").to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn sample_tag() -> Tag {
        Tag {
            id: 1,
            name: "kubernetes".to_string(),
        }
    }

    fn sample_tags() -> Tags {
        Tags {
            tags: vec![
                sample_tag(),
                Tag {
                    id: 2,
                    name: "rust".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_tag_to_admin_template() {
        let template = sample_tag().to_admin_template();
        assert_eq!(template.id, 1);
        assert_eq!(template.name, "kubernetes");
    }

    #[test]
    fn test_tag_data_size() {
        assert!(sample_tag().data_size() > 0);
    }

    #[test]
    fn test_tags_to_admin_list_template() {
        let template = sample_tags().to_admin_list_template();
        assert_eq!(template.tags.len(), 2);
        assert_eq!(template.tags[0].name, "kubernetes");
        assert_eq!(template.tags[1].name, "rust");
    }

    #[test]
    fn test_tags_to_vector_string() {
        assert_eq!(
            sample_tags().to_vector_string(),
            vec!["kubernetes".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn test_tags_to_formatted_string() {
        assert_eq!(sample_tags().to_formatted_string(), "kubernetes, rust");
    }

    #[test]
    fn test_tags_list_params_sanitize_defaults() {
        let params = TagsListParams {
            start: None,
            end: None,
        };
        assert_eq!(
            params.sanitize(),
            TagsListParams {
                start: Some(0),
                end: Some(100)
            }
        );
    }

    #[test]
    fn test_tags_list_params_sanitize_negative_values() {
        let params = TagsListParams {
            start: Some(-1),
            end: Some(-2),
        };
        assert_eq!(
            params.sanitize(),
            TagsListParams {
                start: Some(0),
                end: Some(100)
            }
        );
    }

    #[test]
    fn test_tags_list_params_sanitize_valid_values() {
        let params = TagsListParams {
            start: Some(3),
            end: Some(7),
        };
        assert_eq!(
            params.sanitize(),
            TagsListParams {
                start: Some(3),
                end: Some(7)
            }
        );
    }

    #[test]
    fn test_tags_search_params_sanitize_defaults() {
        let params = TagsSearchParams {
            start: None,
            end: None,
            query: String::new(),
        };
        assert_eq!(
            params.sanitize(),
            TagsSearchParams {
                start: Some(0),
                end: Some(100),
                query: String::new()
            }
        );
    }

    #[test]
    fn test_tags_search_params_sanitize_preserves_start_and_end() {
        // Regression: start used to be derived from end.
        let params = TagsSearchParams {
            start: Some(10),
            end: Some(50),
            query: "rust".to_string(),
        };
        assert_eq!(
            params.sanitize(),
            TagsSearchParams {
                start: Some(10),
                end: Some(50),
                query: "rust".to_string()
            }
        );
    }

    #[test]
    fn test_tags_search_params_sanitize_query() {
        let params = TagsSearchParams {
            start: None,
            end: None,
            query: "  rust @@@ dev !!".to_string(),
        };
        let sanitized = params.sanitize();
        // Regex::replace only replaces the first non-alphanumeric run.
        assert_eq!(sanitized.query, "  rust  dev !!");
    }

    #[test]
    fn test_tags_search_params_sanitize_query_empty() {
        let params = TagsSearchParams {
            start: Some(-1),
            end: None,
            query: "!!!".to_string(),
        };
        let sanitized = params.sanitize();
        assert_eq!(sanitized.start, Some(-1));
        assert_eq!(sanitized.end, Some(100));
        assert_eq!(sanitized.query, "");
    }
}
