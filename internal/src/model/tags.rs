use serde::{Deserialize, Serialize};

/// Tag
/// Just tag id and it's name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

/// Tags
/// Vector of tag id and it's name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tags {
    pub tags: Vec<Tag>,
}

/// TagCommandStatus
/// Status of Tag Command Operations:
/// - Stored
/// - Updated
/// - Deleted
/// I think you should wrap this with Option so you can check if it `None`
/// then check the value of the status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TagCommandStatus {
    Stored,
    Updated,
    Deleted,
}
