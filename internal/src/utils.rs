use markdown::{to_html_with_options, CompileOptions, Constructs, Options, ParseOptions};

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

/// remove_whitespace
/// Take borrow of str and remove whitespace
/// return cleaned String
pub fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Process Markdown
/// take String of markdown body and convert into html with Askama Options
/// return String of converted markdown in html
pub fn convert_markdown_to_html(body_md: String) -> String {
    to_html_with_options(
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
    .unwrap()
}

#[cfg(test)]
mod test {
    use super::*;
    use test_log::test;

    #[test]
    fn test_capitalize() {
        let test = "lorem ipsum dolor sit amet";
        let expected = "Lorem ipsum dolor sit amet".to_string();
        let result = capitalize(test);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_whitespace() {
        let test = "kubernetes, jenkins, grafana, ec2";
        let expected = "kubernetes,jenkins,grafana,ec2".to_string();
        let result = remove_whitespace(test);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_convert_markdown_to_html_header() {
        let header = r#"
# Hello World
## Heading level 2
### Heading level 3
#### Heading level 4"#
            .to_string();
        let expected = "<h1>Hello World</h1>\n<h2>Heading level 2</h2>\n<h3>Heading level 3</h3>\n<h4>Heading level 4</h4>".to_string();
        let result = convert_markdown_to_html(header);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_convert_markdown_to_html_text_style() {
        let text = r#"
**bold**
*italics*
*italics and later **bold***
~~strikethrough~~
[A link](http://example.com)"#
            .to_string();
        let expected = "<p><strong>bold</strong>\n<em>italics</em>\n<em>italics and later <strong>bold</strong></em>\n<del>strikethrough</del>\n<a href=\"http://example.com\">A link</a></p>".to_string();
        let result = convert_markdown_to_html(text);
        assert_eq!(result, expected);
    }
}
