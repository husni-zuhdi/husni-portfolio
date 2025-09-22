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
    pub date: String,
    pub media_link: Option<String>,
    pub org_name: Option<String>,
    pub org_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Talks {
    pub talks: Vec<Talk>,
}

/// TalksParams
/// Axum parameters query for pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TalksParams {
    pub start: Option<i64>,
    pub end: Option<i64>,
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
