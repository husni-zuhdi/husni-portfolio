use crate::model::templates::{BlogMetadataTemplate, BlogTemplate};
use crate::utils::{convert_markdown_to_html, remove_whitespace};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// BlogCommandStatus
/// Status of Blog Command Operations:
/// - Stored
/// - Updated
/// - Deleted
///
/// I think you should wrap this with Option so you can check if it `None`
/// then check the value of the status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlogCommandStatus {
    Stored,
    Updated,
    Deleted,
    CacheInserted,
    CacheInvalidated,
}

/// BlogType
/// Type of Blog source
/// Can be:
/// - Filesystem: Blog markdown come from filesystem
/// - Github: Blog markdown come from github repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlogSource {
    Filesystem,
    Github,
}

impl Display for BlogSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Filesystem => {
                write!(f, "Filesystem")
            }
            Self::Github => {
                write!(f, "Github")
            }
        }
    }
}

/// Blog
/// Blog data with fields:
/// - id: Blog Identifier
/// - name: Blog name
/// - source: Blog source
/// - filename: Blog Filename or Source
/// - body: Blog HTML body
/// - tags: Blog tags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Blog {
    pub id: i64,
    pub name: Option<String>,
    pub source: Option<BlogSource>,
    pub filename: Option<String>,
    pub body: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl Blog {
    /// Convert `Blog` to `BlogMetadata`
    pub fn as_blog_metadata(&self) -> BlogMetadata {
        BlogMetadata {
            id: self.id,
            name: self.name.clone().unwrap(),
            filename: self.filename.clone().unwrap_or_default(),
            tags: self.tags.clone().unwrap(),
        }
    }
    /// Convert to `BlogTemplate`
    pub fn as_template(&self) -> BlogTemplate {
        BlogTemplate {
            id: self.id,
            filename: self.filename.clone().unwrap_or_default(),
            name: self.name.clone().unwrap(),
            body: convert_markdown_to_html(&self.body.clone().unwrap()),
            tags: self.tags.clone().unwrap(),
        }
    }
    /// Calculate size of Tag in u32
    /// Useful for weighing data size
    pub const fn data_size(&self) -> u32 {
        (size_of_val(&self.id)
            + size_of_val(&self.name)
            + size_of_val(&self.source)
            + size_of_val(&self.filename)
            + size_of_val(&self.body)
            + size_of_val(&self.tags)) as u32
    }
}

/// BlogsParams
/// Axum Query struct for `/blogs` query parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlogsParams {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub tags: Option<String>,
}

impl BlogsParams {
    /// Sanitize BlogsParams
    pub fn sanitize(&self) -> Self {
        let start = self.start.unwrap_or(0);
        let end = self.end.unwrap_or(100);
        let tags: String = self
            .tags
            .as_ref()
            .map(|val| remove_whitespace(val))
            .unwrap_or_default();
        Self {
            start: Some(start),
            end: Some(end),
            tags: Some(tags),
        }
    }
}

/// BlogMetadata
/// Minimum Metadata to query Blog
/// filename can be full filename in filesystem or url to github blog content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlogMetadata {
    pub id: i64,
    pub name: String,
    pub filename: String,
    pub tags: Vec<String>,
}

impl BlogMetadata {
    /// Convert to Blog. Take `body` to convert a metadata into minimal blog
    pub fn as_blog(&self, body: String) -> Blog {
        Blog {
            id: self.id,
            name: Some(self.name.clone()),
            filename: Some(self.filename.clone()),
            body: Some(body),
            tags: Some(self.tags.clone()),
            source: None,
        }
    }
    /// Convert to BlogMetadata template
    pub fn as_template(&self) -> BlogMetadataTemplate {
        BlogMetadataTemplate {
            id: self.id,
            name: self.name.clone(),
            tags: self.tags.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn sample_blog() -> Blog {
        Blog {
            id: 1,
            name: Some("My Blog".to_string()),
            source: Some(BlogSource::Filesystem),
            filename: Some("my-blog.md".to_string()),
            body: Some("# Hello".to_string()),
            tags: Some(vec!["rust".to_string()]),
        }
    }

    fn sample_metadata() -> BlogMetadata {
        BlogMetadata {
            id: 2,
            name: "Meta Blog".to_string(),
            filename: "meta-blog.md".to_string(),
            tags: vec!["go".to_string()],
        }
    }

    #[test]
    fn test_blog_source_display_filesystem() {
        assert_eq!(BlogSource::Filesystem.to_string(), "Filesystem");
    }

    #[test]
    fn test_blog_source_display_github() {
        assert_eq!(BlogSource::Github.to_string(), "Github");
    }

    #[test]
    fn test_blog_as_blog_metadata() {
        let metadata = sample_blog().as_blog_metadata();
        assert_eq!(metadata.id, 1);
        assert_eq!(metadata.name, "My Blog");
        assert_eq!(metadata.filename, "my-blog.md");
        assert_eq!(metadata.tags, vec!["rust".to_string()]);
    }

    #[test]
    fn test_blog_as_template() {
        let template = sample_blog().as_template();
        assert_eq!(template.id, 1);
        assert_eq!(template.name, "My Blog");
        assert_eq!(template.filename, "my-blog.md");
        assert!(template.body.contains("<h1>Hello</h1>"));
        assert_eq!(template.tags, vec!["rust".to_string()]);
    }

    #[test]
    fn test_blog_data_size() {
        assert!(sample_blog().data_size() > 0);
    }

    #[test]
    fn test_blog_metadata_as_blog() {
        let blog = sample_metadata().as_blog("body".to_string());
        assert_eq!(blog.id, 2);
        assert_eq!(blog.name, Some("Meta Blog".to_string()));
        assert_eq!(blog.filename, Some("meta-blog.md".to_string()));
        assert_eq!(blog.body, Some("body".to_string()));
        assert_eq!(blog.tags, Some(vec!["go".to_string()]));
        assert_eq!(blog.source, None);
    }

    #[test]
    fn test_blog_metadata_as_template() {
        let template = sample_metadata().as_template();
        assert_eq!(template.id, 2);
        assert_eq!(template.name, "Meta Blog");
        assert_eq!(template.tags, vec!["go".to_string()]);
    }

    #[test]
    fn test_blogs_params_sanitize_defaults() {
        let params = BlogsParams {
            start: None,
            end: None,
            tags: None,
        };
        let sanitized = params.sanitize();
        assert_eq!(sanitized.start, Some(0));
        assert_eq!(sanitized.end, Some(100));
        assert_eq!(sanitized.tags, Some(String::new()));
    }

    #[test]
    fn test_blogs_params_sanitize_preserves_start_and_end() {
        // Regression: start used to be derived from end.
        let params = BlogsParams {
            start: Some(10),
            end: Some(50),
            tags: Some("rust".to_string()),
        };
        let sanitized = params.sanitize();
        assert_eq!(sanitized.start, Some(10));
        assert_eq!(sanitized.end, Some(50));
    }

    #[test]
    fn test_blogs_params_sanitize_passes_through_negative_values() {
        let params = BlogsParams {
            start: Some(-1),
            end: Some(-5),
            tags: None,
        };
        let sanitized = params.sanitize();
        assert_eq!(sanitized.start, Some(-1));
        assert_eq!(sanitized.end, Some(-5));
    }

    #[test]
    fn test_blogs_params_sanitize_tags_remove_whitespace() {
        let params = BlogsParams {
            start: Some(0),
            end: Some(100),
            tags: Some(" rust,   dev ".to_string()),
        };
        let sanitized = params.sanitize();
        assert_eq!(sanitized.tags, Some("rust,dev".to_string()));
    }
}
