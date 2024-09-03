use log::debug;
use markdown::{to_html_with_options, CompileOptions, Constructs, Options, ParseOptions};
use std::fs;

/// md_to_html: Markdown to HTML
/// take String of filename
/// return String of converted markdown in html or String of error
pub fn md_to_html(filename: String) -> Result<String, String> {
    let body_md = fs::read_to_string(filename.clone()).expect("Failed to read markdown blog file");
    debug!("Markdown Body for filename {}: {}", &filename, body_md);

    let html = to_html_with_options(
        &body_md,
        &Options {
            parse: ParseOptions {
                constructs: Constructs {
                    // In case you want to activeat frontmatter in the future
                    // frontmatter: true,
                    ..Constructs::gfm()
                },
                ..ParseOptions::gfm()
            },
            compile: CompileOptions::gfm(),
        },
    )
    .expect("Failed to convert html with options");
    Ok(html)
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
