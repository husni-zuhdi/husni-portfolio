use markdown::{to_html_with_options, CompileOptions, Options, ParseOptions};
use regex::Regex;
use std::collections::HashSet;

/// Capitalize the first character in s.
/// Take borrowed str of s
/// then return capitalized String
#[must_use]
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or_else(String::new, |f| {
        f.to_uppercase().collect::<String>() + c.as_str()
    })
}

/// Take borrow of str and remove whitespace
/// return cleaned String
#[must_use]
pub fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Convert String to Vec<String> for Tags
/// Tags from axum query come in a String with comma separated format
#[must_use]
pub fn convert_tags_string_to_vec(s: &str) -> Vec<String> {
    s.split(',').map(std::string::ToString::to_string).collect()
}

/// Wrap every rendered `<table>` in a scrollable container so tables don't
/// break the layout on narrow screens.
fn wrap_tables(html: &str) -> String {
    html.replace("<table>", r#"<div class="table-wrapper"><table>"#)
        .replace("</table>", r#"</table></div>"#)
}

/// Convert fenced ` ```mermaid ` code blocks from the markdown output
/// (`<pre><code class="language-mermaid">...</code></pre>`) into
/// `<pre class="mermaid">...</pre>` so Pele can mount them.
fn convert_mermaid_code_blocks(html: &str) -> String {
    static MERMAID_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = MERMAID_RE.get_or_init(|| {
        Regex::new(r#"<pre><code class="language-mermaid">([\s\S]*?)</code></pre>"#).unwrap()
    });
    re.replace_all(html, r#"<pre class="mermaid">$1</pre>"#)
        .into_owned()
}

/// Process Markdown
/// take String of markdown body and convert into html with Askama Options
/// return String of converted markdown in html
///
/// # Panics
/// panic if failed to convert markdown to html with allow dangerous html option
#[must_use]
pub fn convert_markdown_to_html(body_md: &str) -> String {
    let html = to_html_with_options(
        body_md,
        &Options {
            parse: ParseOptions::gfm(),
            compile: CompileOptions {
                allow_dangerous_html: true,
                ..CompileOptions::gfm()
            },
        },
    )
    .unwrap();

    let html = wrap_tables(&html);
    let html = convert_mermaid_code_blocks(&html);

    ammonia::Builder::default()
        .link_rel(None)
        .url_relative(ammonia::UrlRelative::PassThrough)
        .add_clean_content_tags(&["script", "style"])
        .add_generic_attributes(&["class"])
        .add_tag_attributes("a", HashSet::from(["target"]))
        .clean(&html)
        .to_string()
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
    fn test_convert_markdown_to_html_link_with_target() {
        let text = r#"<a href="https://example.com" target="_blank">Example</a>"#.to_string();
        let result = convert_markdown_to_html(&text);
        assert_eq!(
            result,
            r#"<p><a href="https://example.com" target="_blank">Example</a></p>"#
        );
    }

    #[test]
    fn test_convert_markdown_to_html_header() {
        let header = r"
# Hello World
## Heading level 2
### Heading level 3
#### Heading level 4"
            .to_string();
        let expected = "<h1>Hello World</h1>\n<h2>Heading level 2</h2>\n<h3>Heading level 3</h3>\n<h4>Heading level 4</h4>".to_string();
        let result = convert_markdown_to_html(&header);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_convert_markdown_to_html_text_style() {
        let text = r"
**bold**
*italics*
*italics and later **bold***
~~strikethrough~~
[A link](http://example.com)"
            .to_string();
        let expected = "<p><strong>bold</strong>\n<em>italics</em>\n<em>italics and later <strong>bold</strong></em>\n<del>strikethrough</del>\n<a href=\"http://example.com\">A link</a></p>".to_string();
        let result = convert_markdown_to_html(&text);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_convert_markdown_to_html_table_wrapped() {
        let table = "| a | b |\n|---|---|\n| 1 | 2 |".to_string();
        let result = convert_markdown_to_html(&table);
        assert!(
            result.contains(r#"<div class="table-wrapper"><table>"#),
            "table should be wrapped in a scrollable container, got: {result}"
        );
        assert!(
            result.contains(r#"</table></div>"#),
            "table wrapper should be closed, got: {result}"
        );
    }

    #[test]
    fn test_convert_markdown_to_html_mermaid_code_block() {
        let md = "```mermaid\nflowchart LR\n  A --> B\n```".to_string();
        let result = convert_markdown_to_html(&md);
        assert!(
            result.contains(r#"<pre class="mermaid">"#),
            "mermaid code block should become pre.mermaid, got: {result}"
        );
        assert!(
            !result.contains("language-mermaid"),
            "mermaid code block should not keep language-mermaid class, got: {result}"
        );
    }

    #[test]
    fn test_convert_markdown_to_html_preserves_class_attribute() {
        let raw = r#"<div class="note">hello</div>"#.to_string();
        let result = convert_markdown_to_html(&raw);
        assert!(
            result.contains(r#"class="note""#),
            "class attribute should be preserved, got: {result}"
        );
    }
}
