use crate::api::github::get_gh_blogs;
use crate::model::blog::{
    Blog, BlogBody, BlogDeleted, BlogEndPage, BlogFilename, BlogId, BlogName, BlogSource,
    BlogStartPage,
};
use crate::repo::blog::BlogRepo;
use crate::utils::{capitalize, md_to_html};
use async_trait::async_trait;
use log::{debug, info};
use std::fs;

#[derive(Clone)]
pub struct MemoryBlogRepo {
    pub blogs: Vec<Blog>,
}

#[async_trait]
impl BlogRepo for MemoryBlogRepo {
    async fn find(&mut self, id: BlogId) -> Blog {
        let result = self
            .blogs
            .iter()
            .filter(|blog| &blog.id == &id)
            .next()
            .unwrap();
        info!("Blog {} processed.", &result.id);
        debug!("Blog HTML {}.", &result.body);

        result.clone()
    }
    async fn find_blogs(&mut self, start: BlogStartPage, end: BlogEndPage) -> Vec<Blog> {
        let start_seq = start.0 as usize;
        let end_seq = end.0 as usize;
        let result = &self.blogs[start_seq..end_seq];
        result.to_vec()
    }
    async fn add(
        &mut self,
        id: BlogId,
        name: BlogName,
        filename: BlogFilename,
        source: BlogSource,
        body: BlogBody,
    ) -> Blog {
        let result = Blog {
            id,
            name,
            source,
            filename,
            body,
        };
        self.blogs.push(result.clone());
        info!("Blog {} added.", &result.id);
        debug!("Blog HTML {}.", &result.body);
        result
    }
    async fn delete(&mut self, id: BlogId) -> BlogDeleted {
        let index = self.blogs.iter().position(|blog| &blog.id == &id).unwrap();
        info!("Deleting Blog with Id {}", &index);

        self.blogs.remove(index);
        info!("Deleted Blog with Id {}", &index);
        BlogDeleted(true)
    }
    async fn update(
        &mut self,
        id: BlogId,
        name: Option<BlogName>,
        filename: Option<BlogFilename>,
        source: Option<BlogSource>,
        body: Option<BlogBody>,
    ) -> Blog {
        let result: &mut Blog = self
            .blogs
            .iter_mut()
            .filter(|blog| &blog.id == &id)
            .next()
            .unwrap();
        match name {
            Some(val) => {
                debug!("Update Blog {} name from {} to {}", &id, &result.name, &val);
                result.name = val
            }
            None => (),
        }
        match filename {
            Some(val) => {
                debug!(
                    "Update Blog {} filename from {} to {}",
                    &id, &result.filename, &val
                );
                result.filename = val
            }
            None => (),
        }
        match source {
            Some(val) => {
                debug!(
                    "Update Blog {} source from {} to {}",
                    &id, &result.source, &val
                );
                result.source = val
            }
            None => (),
        }
        match body {
            Some(val) => {
                debug!("Update Blog {} body from {} to {}", &id, &result.body, &val);
                result.body = val
            }
            None => (),
        }
        result.clone()
    }
}

impl MemoryBlogRepo {
    pub fn new() -> MemoryBlogRepo {
        let dir = Some("./statics/blogs/".to_string());
        Self::from_dir(dir)
    }

    /// Async function to get BlogsData from github
    /// Borrowed `owner`, `repo`, and `branch` String
    pub async fn from_github(owner: &String, repo: &String, branch: &String) -> Self {
        let dir = Some("./statics/blogs/".to_string());
        let mut blog_data = Self::from_dir(dir).blogs;
        let mut gh_blog_data =
            get_gh_blogs(owner.to_string(), repo.to_string(), branch.to_string())
                .await
                .expect("Failed to get github blog data");
        blog_data.append(&mut gh_blog_data);
        Self { blogs: blog_data }
    }

    /// Create MemoryBlogRepo from directory
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

        let blogs: Vec<Blog> = blogs_paths
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
                Blog {
                    id: BlogId(id.to_string()),
                    name: BlogName(name.to_string()),
                    source: BlogSource::FileSystem,
                    filename: BlogFilename(blog_path.to_owned()),
                    body: BlogBody(body),
                }
            })
            .collect();

        debug!("Blogs: {:?}", blogs);

        Self { blogs }
    }
}

impl Default for MemoryBlogRepo {
    fn default() -> Self {
        MemoryBlogRepo::new()
    }
}
