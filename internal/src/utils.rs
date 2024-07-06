use crate::model::data::VersionData;
use markdown::to_html;
use std::fs;
use std::io::BufReader;

/// md_to_html: Markdown to HTML
/// take String of filename
/// return String of converted markdown in html or String of error
pub fn md_to_html(filename: String) -> Result<String, String> {
    let body_md = fs::read_to_string(filename).expect("Failed to read markdown blog file");
    Ok(to_html(&body_md))
}

/// read_version_manifest
/// read version manifest on root repository to get this configuration
/// * version
/// * git build hash
/// * build date
pub fn read_version_manifest() -> Result<VersionData, String> {
    let file = fs::File::open("version.json").expect("Failed to open version.json");
    let reader = BufReader::new(file);

    let json: VersionData = serde_json::from_reader(reader).expect("Failed to parse version.json");
    Ok(json)
}

/// Capitalizes the first character in s.
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
