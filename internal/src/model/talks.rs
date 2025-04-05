use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// TalkId
/// Identifier of a Talk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TalkId {
    pub id: i64,
}

impl Display for TalkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Talk
/// Talk data with fields:
/// - id: Talk Identifier
/// - name: Talk Name
/// - media_link: (Optional) Talk media (video/record) link
/// - org_link: (Optional) Talk organisation link
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Talk {
    pub id: TalkId,
    pub name: String,
    pub media_link: Option<String>,
    pub org_link: Option<String>,
}

/// TalkStartPage
/// Start page of Talk Pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TalkStartPage(pub i32);

/// TalkEndPage
/// End page of Talk Pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TalkEndPage(pub i32);

/// TalkPagination
/// Axum Query struct for Talk Pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TalkPagination {
    pub start: Option<TalkStartPage>,
    pub end: Option<TalkEndPage>,
}

/// TalkCommandStatus
/// Status of Talk Command Operations:
/// - Stored
/// - Updated
/// - Deleted
/// I think you should wrap this with Option so you can check if it `None`
/// then check the value of the status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TalkCommandStatus {
    Stored,
    Updated,
    Deleted,
}
