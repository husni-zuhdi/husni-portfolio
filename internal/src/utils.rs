use crate::model::version::Version;
use markdown::{to_html_with_options, Options};
use std::fs;
use std::io::BufReader;

/// md_to_html: Markdown to HTML
/// take String of filename
/// return String of converted markdown in html or String of error
pub fn md_to_html(filename: String) -> Result<String, String> {
    let body_md = fs::read_to_string(filename).expect("Failed to read markdown blog file");
    let html = to_html_with_options(&body_md, &Options::gfm())
        .expect("Failed to convert html with options");
    Ok(html)
}

/// read_version_manifest
/// read version manifest on root repository to get this configuration
/// * version
/// * git build hash
/// * build date
pub fn read_version_manifest() -> Result<Version, String> {
    let file = fs::File::open("version.json").expect("Failed to open version.json");
    let reader = BufReader::new(file);

    let json: Version = serde_json::from_reader(reader).expect("Failed to parse version.json");
    Ok(json)
}

/// capitalize
/// Capitalize the first character in s.
/// Take borrowed str of s
/// then return capitalized String
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_log::test;

    #[test]
    fn test_capitalize() {
        let test = "lorem ipsum dolor sit amet".to_string();
        let expected = "Lorem ipsum dolor sit amet".to_string();
        let result = capitalize(test.as_str());
        assert_eq!(result, expected);
    }
}
