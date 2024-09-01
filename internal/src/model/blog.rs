use crate::api::github::get_gh_blogs;
use crate::utils::{capitalize, md_to_html};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;

/// BlogId
/// Identifier of Blog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogId(pub String);

impl BlogId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for BlogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// BlogName
/// Name of the Blog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogName(pub String);

impl BlogName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for BlogName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// BlogFilename
/// Filename of the Blog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogFilename(pub String);

impl BlogFilename {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for BlogFilename {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// BlogBody
/// HTML body of the Blog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogBody(pub String);

impl BlogBody {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for BlogBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// BlogDeleted
/// Blog Deleted or not
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogDeleted(pub bool);

/// BlogType
/// Type of Blog source
/// Can be:
/// - FileSystem: Blog markdown come from filesystem
/// - Github: Blog markdown come from github repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlogSource {
    FileSystem,
    Github,
}

impl Display for BlogSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::FileSystem => {
                write!(f, "FileSystem")
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Blog {
    pub id: BlogId,
    pub name: BlogName,
    pub source: BlogSource,
    pub filename: BlogFilename,
    pub body: BlogBody,
}

impl Blog {
    pub fn update_name(&mut self, new_name: BlogName) {
        self.name = new_name
    }
    pub fn update_source(&mut self, new_source: BlogSource) {
        self.source = new_source
    }
    pub fn update_filename(&mut self, new_filename: BlogFilename) {
        self.filename = new_filename
    }
    pub fn update_body(&mut self, new_body: BlogBody) {
        self.body = new_body
    }
}

/// BlogStartPage
/// Start page of Blog Pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogStartPage(pub i32);

/// BlogEndPage
/// End page of Blog Pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogEndPage(pub i32);

// /// Blogs
// /// Vector of Blog in range of start page and end page
// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub struct Blogs {
//     pub blogs: Vec<Blog>,
//     pub blog_start_page: BlogStartPage,
//     pub blog_end_page: BlogEndPage,
// }
//
// impl Default for Blogs {
//     fn default() -> Self {
//         let dir = Some("./statics/blogs/".to_string());
//         Self::from_dir(dir)
//     }
// }
//
// impl Blogs {
//     /// Async function to get BlogsData from github
//     /// Borrowed `owner`, `repo`, and `branch` String
//     pub async fn with_gh(owner: &String, repo: &String, branch: &String) -> Self {
//         let dir = Some("./statics/blogs/".to_string());
//         let mut blog_data = Self::from_dir(dir).blogs;
//         let mut gh_blog_data =
//             get_gh_blogs(owner.to_string(), repo.to_string(), branch.to_string())
//                 .await
//                 .expect("Failed to get github blog data");
//         blog_data.append(&mut gh_blog_data);
//         Self {
//             blogs: blog_data,
//             blog_start_page: BlogStartPage(0),
//             blog_end_page: BlogEndPage(10),
//         }
//     }
//
//     pub fn from_dir(dir: Option<String>) -> Self {
//         let directory = dir.clone().expect("Failed to get directory");
//         let static_path = fs::read_dir(directory.as_str()).unwrap();
//
//         let blogs_paths: Vec<String> = static_path
//             .filter_map(|blog_path| {
//                 let path = blog_path.ok().expect("Failed to get blog path").path();
//                 if path.is_file() {
//                     path.file_name()
//                         .expect("Failed to get filename")
//                         .to_str()
//                         .map(|s| s.to_owned())
//                 } else {
//                     None
//                 }
//             })
//             .collect();
//
//         let blogs: Vec<Blog> = blogs_paths
//             .iter()
//             .map(|blog_path| {
//                 let (id, name_init) = blog_path
//                     .split_once("-")
//                     .expect("Failed to split filename into id and name");
//                 let name_formated = name_init.replace("_", " ");
//                 let (name_lower, _) = name_formated
//                     .split_once(".")
//                     .expect("Failed to remove file extension");
//                 let name = capitalize(name_lower);
//                 let fullpath = format!("{}{}", directory, blog_path);
//
//                 info!("markdown loaded: {}", fullpath);
//
//                 let body = md_to_html(fullpath).expect("Failed to convert markdown to html");
//                 Blog {
//                     id: BlogId(id.to_string()),
//                     name: BlogName(name.to_string()),
//                     source: BlogSource::FileSystem,
//                     filename: BlogFilename(blog_path.to_owned()),
//                     body: BlogBody(body),
//                 }
//             })
//             .collect();
//
//         debug!("Blogs: {:?}", blogs);
//
//         Self {
//             blogs,
//             blog_start_page: BlogStartPage(0),
//             blog_end_page: BlogEndPage(10),
//         }
//     }
// }
//
// #[cfg(test)]
// mod test {
//     use super::*;
//     use std::env::current_dir;
//     use std::io::Write;
//     use test_log::test;
//
//     #[test]
//     fn test_blogs_data_from_dir() {
//         // Preparation
//         let test_id = "999";
//         let test_name = "Test blog";
//         let test_body = "# Testing Blog for Unit Test";
//         let test_body_html = "<h1>Testing Blog for Unit Test</h1>";
//         let test_filename = "999-test_blog.md";
//         let test_path = "../statics/blogs/999-test_blog.md";
//
//         // Get current directory
//         debug!(
//             "Curent Directory: {}",
//             current_dir().expect("Failed to get current dir").display()
//         );
//
//         // Create a blog markdown
//         let mut md_file = fs::File::create(test_path).expect("Failed to create File Write buffer");
//         md_file
//             .write_all(test_body.as_bytes())
//             .expect("Failed to write buffer to");
//
//         // Call create_blogs function
//         let dir = Some("../statics/blogs/".to_string());
//         let blogs = Blogs::from_dir(dir);
//
//         // Check blogs data
//         debug!("Check BlogsData: {:?}", blogs);
//
//         let blog_test = blogs
//             .blogs
//             .iter()
//             .filter(|blog| blog.id == BlogId(test_id.to_string()))
//             .next()
//             .expect("Failed to get test blog data");
//
//         // Compare if new blog markdown is available
//         assert_eq!(blog_test.id, BlogId(test_id.to_string()));
//         assert_eq!(blog_test.name, BlogName(test_name.to_string()));
//         assert_eq!(blog_test.body, BlogBody(test_body_html.to_string()));
//         assert_eq!(blog_test.filename, BlogFilename(test_filename.to_string()));
//
//         // Delete test blog markdown
//         fs::remove_file(test_path).expect("Failed to delete test blog markdown");
//     }
// }
