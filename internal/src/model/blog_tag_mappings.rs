use serde::{Deserialize, Serialize};

/// Blog Tag Mapping
/// Corelate a blog id with a tag id
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogTagMapping {
    pub blog_id: i64,
    pub tag_id: i64,
}

/// Blog Tag Mappings
/// Vector of blog id and it's tag id
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogTagMappings {
    pub maps: Vec<BlogTagMapping>,
}

/// BlogTagMappingCommandStatus
/// Status of Tag Command Operations:
/// - Stored
/// - Updated
/// - Deleted
/// I think you should wrap this with Option so you can check if it `None`
/// then check the value of the status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlogTagMappingCommandStatus {
    Stored,
    Updated,
    Deleted,
}
