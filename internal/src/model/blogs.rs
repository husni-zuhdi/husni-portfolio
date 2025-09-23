use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// BlogCommandStatus
/// Status of Blog Command Operations:
/// - Stored
/// - Updated
/// - Deleted
/// I think you should wrap this with Option so you can check if it `None`
/// then check the value of the status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlogCommandStatus {
    Stored,
    Updated,
    Deleted,
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

/// BlogsParams
/// Axum Query struct for `/blogs` query parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogsParams {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub tags: Option<String>,
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
    pub fn get_ref_tags(&self) -> Vec<&str> {
        self.tags.iter().map(|t| t.as_ref()).collect()
    }
}
