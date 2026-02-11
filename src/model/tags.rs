use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::model::templates_admin::{AdminBlogTagsListTemplate, AdminGetTagTemplate};

/// Tag
/// Just tag id and it's name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub fn data_size(&self) -> u32 {
        (size_of_val(&self.id) + size_of_val(&self.name)) as u32
    }
}

/// Tags
/// Vector of tag id and it's name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TagCommandStatus {
    Stored,
    Updated,
    Deleted,
    CacheInserted,
    CacheInvalidated,
}

/// TagsParams
/// Axum Query struct for `/admin/blogs/tags/list` query parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
                debug!("TagsListParams: set default end to 10");
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagsSearchParams {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub query: String,
}

impl TagsSearchParams {
    /// Sanitize TagSearchParams by checking negative value and set unknown to
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
                debug!("TagsListParams: set default end to 10");
                100_i64
            }
        };

        let pattern = Regex::new(r"[^a-zA-Z0-9\s]+").unwrap();
        let sanitized_query = pattern.replace(&self.query, "").to_string();
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
}
