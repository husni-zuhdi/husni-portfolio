use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// BlogId
/// Identifier of Blog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogId {
    pub id: i64,
}

impl Display for BlogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Blog {
    pub id: BlogId,
    pub name: String,
    pub source: BlogSource,
    pub filename: String,
    pub body: String,
}

/// BlogStartPage
/// Start page of Blog Pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogStartPage(pub i32);

/// BlogEndPage
/// End page of Blog Pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogEndPage(pub i32);

/// BlogPagination
/// Axum Query struct for Blog Pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogPagination {
    pub start: Option<BlogStartPage>,
    pub end: Option<BlogEndPage>,
}

/// BlogMetadata
/// Minimum Metadata to query Blog
/// filename can be full filename in filesystem or url to github blog content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogMetadata {
    pub id: BlogId,
    pub name: String,
    pub filename: String,
}
