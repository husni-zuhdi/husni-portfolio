use crate::model::blogs::{
    Blog, BlogBody, BlogFilename, BlogId, BlogMetadata, BlogName, BlogSource,
};
use crate::repo::api::ApiRepo;
use crate::utils::capitalize;
use async_trait::async_trait;
use markdown::{to_html_with_options, CompileOptions, Constructs, Options, ParseOptions};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct FilesystemApiUseCase {
    pub blogs_dir: String,
}

#[async_trait]
impl ApiRepo for FilesystemApiUseCase {
    async fn list_metadata(&self) -> Option<Vec<BlogMetadata>> {
        let read_dir = fs::read_dir(self.blogs_dir.clone());
        match read_dir {
            Ok(value) => {
                let metadatas = value
                    .filter_map(|blog_path| {
                        let blog_path_buf = blog_path.expect("Failed to get blog DirEntry").path();
                        Self::process_blog_path(&self, blog_path_buf)
                    })
                    // Collect Blog Metadata
                    .map(|blog_filename| Self::process_blog_metadata(&self, blog_filename))
                    .collect();
                Some(metadatas)
            }
            Err(err) => {
                error!(
                    "Failed to read directory. Returned empty Vector. Error: {}",
                    err
                );
                None
            }
        }
    }
    async fn fetch(&self, metadata: BlogMetadata) -> Option<Blog> {
        let result = Self::process_markdown(metadata.filename.0.clone());
        match result {
            Ok(body) => {
                debug!("Blog Body with Id {}: {}", &metadata.id.0, &body);

                Some(Blog {
                    id: metadata.id,
                    name: metadata.name,
                    source: BlogSource::Filesystem,
                    filename: metadata.filename,
                    body: BlogBody(body),
                })
            }
            Err(err) => {
                error!(
                    "Failed to process markdown to html for Blog Id {}. Error: {}",
                    &metadata.id.0, err
                );
                None
            }
        }
    }
}

impl FilesystemApiUseCase {
    pub async fn new(blogs_dir: String) -> FilesystemApiUseCase {
        FilesystemApiUseCase { blogs_dir }
    }
    /// Process Blog Path from a PathBuf
    /// Returned an Option String
    fn process_blog_path(&self, blog_path_buf: PathBuf) -> Option<String> {
        if blog_path_buf.is_file() {
            blog_path_buf
                .file_name()
                .expect("Failed to get filename")
                .to_str()
                .map(|str| str.to_owned())
        } else {
            None
        }
    }
    /// Process Blog Metadata from Blog Filename
    /// Returned BlogMetadata
    fn process_blog_metadata(&self, blog_filename: String) -> BlogMetadata {
        let (id, name_init) = blog_filename
            .split_once("-")
            .expect("Failed to split filename into id and name");
        let name_lower = name_init
            .replace("_", " ")
            .split_once(".")
            .expect("Failed to remove file extension.")
            .0
            .to_string();
        let name = capitalize(&name_lower);
        let filename = format!("{}{}", self.blogs_dir, &blog_filename);
        info!("Blog Metadata with Id {} has been processed.", &id);
        debug!("Blog Name with Id {}: {}", &id, &name);
        debug!("Blog Filename with Id {}: {}", &id, &filename);

        BlogMetadata {
            id: BlogId(id.to_string()),
            name: BlogName(name),
            filename: BlogFilename(filename),
        }
    }
    /// Process Markdown
    /// take String of filename and convert markdown file into html with option
    /// return String of converted markdown in html or String of error
    fn process_markdown(filename: String) -> Result<String, String> {
        let body_md =
            fs::read_to_string(filename.clone()).expect("Failed to read markdown blog file");
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
}
