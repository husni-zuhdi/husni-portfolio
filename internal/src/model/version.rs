use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;

/// Version
/// Store version, build hash, and buld date
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Version {
    pub version: String,
    pub build_hash: String,
    pub build_date: String,
}

impl Version {
    /// read_version_manifest
    /// read version manifest on root repository to get this configuration
    /// * version
    /// * git build hash
    /// * build date
    pub fn new() -> Result<Self, String> {
        let file = fs::File::open("version.json").expect("Failed to open version.json");
        let reader = BufReader::new(file);

        let version: Version =
            serde_json::from_reader(reader).expect("Failed to parse version.json");
        Ok(version)
    }
}
