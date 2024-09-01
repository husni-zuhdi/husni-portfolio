use serde::{Deserialize, Serialize};

/// Version
/// Store version, build hash, and buld date
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Version {
    pub version: String,
    pub build_hash: String,
    pub build_date: String,
}
