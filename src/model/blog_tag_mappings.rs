use serde::{Deserialize, Serialize};

/// Blog Tag Mapping
/// Corelate a blog id with a tag id
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlogTagMapping {
    pub blog_id: i64,
    pub tag_id: i64,
}

impl BlogTagMapping {
    /// Calculate size of Tag in u32
    /// Useful for weighing data size
    pub const fn data_size(&self) -> u32 {
        (size_of_val(&self.blog_id) + size_of_val(&self.tag_id)) as u32
    }
}

/// Blog Tag Mappings
/// Vector of blog id and it's tag id
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlogTagMappings {
    pub maps: Vec<BlogTagMapping>,
}

/// BlogTagMappingCommandStatus
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
pub enum BlogTagMappingCommandStatus {
    Stored,
    Updated,
    Deleted,
    CacheInserted,
    CacheInvalidated,
}
