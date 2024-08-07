use crate::utils::{capitalize, md_to_html};
use log::{debug, info};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileData;

#[derive(Debug, Clone, PartialEq)]
pub struct BlogsData {
    pub blogs: Vec<BlogData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlogData {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VersionData {
    pub version: String,
    pub build_hash: String,
    pub build_date: String,
}

impl Default for BlogsData {
    fn default() -> Self {
        let dir = Some("./statics/blogs/".to_string());
        Self::from_dir(dir)
    }
}

impl BlogsData {
    pub fn from_dir(dir: Option<String>) -> Self {
        let directory = dir.clone().expect("Failed to get directory");
        let static_path = fs::read_dir(directory.as_str()).unwrap();

        let blogs_paths: Vec<String> = static_path
            .filter_map(|blog_path| {
                let path = blog_path.ok().expect("Failed to get blog path").path();
                if path.is_file() {
                    path.file_name()
                        .expect("Failed to get filename")
                        .to_str()
                        .map(|s| s.to_owned())
                } else {
                    None
                }
            })
            .collect();

        let blogs: Vec<BlogData> = blogs_paths
            .iter()
            .map(|blog_path| {
                let (id, name_init) = blog_path
                    .split_once("-")
                    .expect("Failed to split filename into id and name");
                let name_formated = name_init.replace("_", " ");
                let (name_lower, _) = name_formated
                    .split_once(".")
                    .expect("Failed to remove file extension");
                let name = capitalize(name_lower);
                let fullpath = format!("{}{}", directory, blog_path);

                info!("markdown loaded: {}", fullpath);

                let body = md_to_html(fullpath).expect("Failed to convert markdown to html");
                BlogData {
                    id: id.to_string(),
                    name: name.to_string(),
                    filename: blog_path.to_owned(),
                    body,
                }
            })
            .collect();

        debug!("Blogs: {:?}", blogs);

        BlogsData { blogs }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env::current_dir;
    use std::io::Write;
    use test_log::test;

    #[test]
    fn test_blogs_data_from_dir() {
        // Preparation
        let test_id = "999";
        let test_name = "Test blog";
        let test_body = "# Testing Blog for Unit Test";
        let test_body_html = "<h1>Testing Blog for Unit Test</h1>";
        let test_filename = "999-test_blog.md";
        let test_path = "../statics/blogs/999-test_blog.md";

        // Get current directory
        debug!(
            "Curent Directory: {}",
            current_dir().expect("Failed to get current dir").display()
        );

        // Create a blog markdown
        let mut md_file = fs::File::create(test_path).expect("Failed to create File Write buffer");
        md_file
            .write_all(test_body.as_bytes())
            .expect("Failed to write buffer to");

        // Call create_blogs function
        let dir = Some("../statics/blogs/".to_string());
        let blogs = BlogsData::from_dir(dir);

        // Check blogs data
        debug!("Check BlogsData: {:?}", blogs);

        let blog_test = blogs
            .blogs
            .iter()
            .filter(|blog| blog.id == test_id)
            .next()
            .expect("Failed to get test blog data");

        // Compare if new blog markdown is available
        assert_eq!(blog_test.id.as_str(), test_id);
        assert_eq!(blog_test.name.as_str(), test_name);
        assert_eq!(blog_test.body.as_str(), test_body_html);
        assert_eq!(blog_test.filename.as_str(), test_filename);

        // Delete test blog markdown
        fs::remove_file(test_path).expect("Failed to delete test blog markdown");
    }
}
