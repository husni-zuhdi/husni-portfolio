use crate::model::data::VersionData;
use log::debug;
use markdown::to_html;
use regex::Regex;
use std::fs;
use std::io::BufReader;

/// md_to_html: Markdown to HTML
/// take String of filename
/// return String of converted markdown in html or String of error
pub fn md_to_html(filename: Option<String>, body: Option<String>) -> Result<String, String> {
    let mut body_md = String::new();
    match filename {
        Some(val) => {
            body_md = fs::read_to_string(val).expect("Failed to read markdown blog file");
        }
        None => (),
    }

    match body {
        Some(val) => body_md = val,
        None => (),
    }
    // let body_md = fs::read_to_string(filename.unwrap()).expect("Failed to read markdown blog file");
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

/// replace_gh_link
/// Replace Github Blog relative links
/// with full github content links
/// Take String of markdown body
/// and String of github blog endpoint
/// then return String of updated body
pub fn replace_gh_link(body: String, gh_blog_link: String, gh_raw_blog_link: String) -> String {
    // Regex href=.\.\/ mean
    // find string with character 'href='
    // then followed by any character (I tried to use '"' but didn't work)
    // then followed by '.' (must use escape character)
    // then followed by '/' (must use escape character)
    let re_href = Regex::new(r"href=.\.\/").expect("Failed to build regex href");

    let replaced_str_href = format!("href=\"{}/", gh_blog_link);
    debug!("Replaced str: {}", &replaced_str_href);

    let res_href = re_href
        .replace_all(body.as_str(), replaced_str_href.as_str())
        .to_string();
    debug!("Replaced Body: {}", &res_href);

    // Regex src=.\.\/ mean
    // find string with character 'src='
    // then followed by any character (I tried to use '"' but didn't work)
    // then followed by '.' (must use escape character)
    // then followed by '/' (must use escape character)
    let re_src = Regex::new(r"src=.\.\/").expect("Failed to build regex src");

    let replaced_str_src = format!("src=\"{}/", gh_raw_blog_link);
    debug!("Replaced str: {}", &replaced_str_src);

    let res = re_src
        .replace_all(res_href.as_str(), replaced_str_src.as_str())
        .to_string();
    debug!("Replaced Body: {}", &res);

    res
}
