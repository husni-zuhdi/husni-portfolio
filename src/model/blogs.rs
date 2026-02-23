use crate::model::templates::{BlogMetadataTemplate, BlogTemplate};
use crate::utils::{convert_markdown_to_html, remove_whitespace};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tracing::debug;

/// BlogCommandStatus
/// Status of Blog Command Operations:
/// - Stored
/// - Updated
/// - Deleted
///
/// I think you should wrap this with Option so you can check if it `None`
/// then check the value of the status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlogCommandStatus {
    Stored,
    Updated,
    Deleted,
    CacheInserted,
    CacheInvalidated,
}

/// BlogType
/// Type of Blog source
/// Can be:
/// - Filesystem: Blog markdown come from filesystem
/// - Github: Blog markdown come from github repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlogSource {
    Filesystem,
    Github,
}

impl Display for BlogSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Filesystem => {
                write!(f, "Filesystem")
            }
            Self::Github => {
                write!(f, "Github")
            }
        }
    }
}

/// Blog
/// Blog data with fields:
/// - id: Blog Identifier
/// - name: Blog name
/// - source: Blog source
/// - filename: Blog Filename or Source
/// - body: Blog HTML body
/// - tags: Blog tags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Blog {
    pub id: i64,
    pub name: Option<String>,
    pub source: Option<BlogSource>,
    pub filename: Option<String>,
    pub body: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl Blog {
    /// Convert `Blog` to `BlogMetadata`
    pub fn as_blog_metadata(&self) -> BlogMetadata {
        BlogMetadata {
            id: self.id,
            name: self.name.clone().unwrap(),
            filename: self.filename.clone().unwrap_or("".to_string()),
            tags: self.tags.clone().unwrap(),
        }
    }
    /// Convert to `BlogTemplate`
    pub fn as_template(&self) -> BlogTemplate {
        BlogTemplate {
            id: self.id,
            filename: self.filename.clone().unwrap_or("".to_string()),
            name: self.name.clone().unwrap(),
            body: convert_markdown_to_html(self.body.clone().unwrap()),
            tags: self.tags.clone().unwrap(),
        }
    }
    /// Calculate size of Tag in u32
    /// Useful for weighing data size
    pub fn data_size(&self) -> u32 {
        (size_of_val(&self.id)
            + size_of_val(&self.name)
            + size_of_val(&self.source)
            + size_of_val(&self.filename)
            + size_of_val(&self.body)
            + size_of_val(&self.tags)) as u32
    }
}

/// BlogsParams
/// Axum Query struct for `/blogs` query parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogsParams {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub tags: Option<String>,
}

impl BlogsParams {
    /// Sanitize BlogsParams
    pub fn sanitize(&self) -> Self {
        let start = match self.start {
            Some(val) => val,
            None => {
                debug!("Set default start to 0");
                0_i64
            }
        };
        let end = match self.end {
            Some(val) => val,
            None => {
                debug!("Set default end to 100");
                100_i64
            }
        };
        let tags: String = match &self.tags {
            Some(val) => remove_whitespace(val),
            None => {
                debug!("Set default tags to empty");
                "".to_string()
            }
        };
        Self {
            start: Some(start),
            end: Some(end),
            tags: Some(tags),
        }
    }
}

/// BlogMetadata
/// Minimum Metadata to query Blog
/// filename can be full filename in filesystem or url to github blog content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogMetadata {
    pub id: i64,
    pub name: String,
    pub filename: String,
    pub tags: Vec<String>,
}

impl BlogMetadata {
    /// Convert to Blog. Take `body` to convert a metadata into minimal blog
    pub fn as_blog(&self, body: String) -> Blog {
        Blog {
            id: self.id,
            name: Some(self.name.clone()),
            filename: Some(self.filename.clone()),
            body: Some(body),
            tags: Some(self.tags.clone()),
            source: None,
        }
    }
    /// Convert to BlogMetadata template
    pub fn as_template(&self) -> BlogMetadataTemplate {
        BlogMetadataTemplate {
            id: self.id,
            name: self.name.clone(),
            tags: self.tags.clone(),
        }
    }
}
