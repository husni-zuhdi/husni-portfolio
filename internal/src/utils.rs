use markdown::to_html;
use std::fs;

/// md_to_html: Markdown to HTML
/// take String of filename
/// return String of converted markdown in html or String of error
pub fn md_to_html(filename: String) -> Result<String, String> {
    let body_md = fs::read_to_string(filename).expect("Failed to read markdown blog file");
    Ok(to_html(&body_md))
}
