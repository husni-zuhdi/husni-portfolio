use crate::model::blogs::*;
use crate::repo::blogs::BlogRepo;
use async_trait::async_trait;
use tracing::{debug, error, info, warn};

// TODO: remove in-memory database

#[derive(Clone)]
pub struct MemoryBlogRepo {
    pub blogs: Vec<Blog>,
}

#[async_trait]
impl BlogRepo for MemoryBlogRepo {
    async fn find(&self, id: BlogId) -> Option<Blog> {
        let result = self.blogs.iter().filter(|blog| &blog.id == &id).next();
        match result {
            Some(blog) => {
                info!("Blog {} processed.", &blog.id);
                debug!("Blog HTML {}.", &blog.body);
                Some(blog.clone())
            }
            None => {
                info!("Blog {} not found. Return None", &id);
                None
            }
        }
    }
    async fn find_blogs(
        &self,
        start: BlogStartPage,
        end: BlogEndPage,
    ) -> Option<Vec<BlogMetadata>> {
        let start_seq = if start.0 as usize > self.blogs.len() {
            warn!("BlogStartPage is greater than Blogs count. Will reset to 0.");
            0
        } else {
            start.0 as usize
        };

        let end_seq = if (end.0 as usize > self.blogs.len()) && self.blogs.len() > 10 {
            warn!("BlogEndPage is greater than Blogs count. Will reset to Blogs count or 10, whichever is lesser.");
            10
        } else if (end.0 as usize > self.blogs.len()) && self.blogs.len() < 10 {
            warn!("BlogEndPage is greater than Blogs count. Will reset to Blogs count or 10, whichever is lesser.");
            self.blogs.len()
        } else if start.0 as usize > end.0 as usize {
            warn!("BlogStartPage is greater than BlogEndPage. Will reset to 10.");
            self.blogs.len()
        } else {
            end.0 as usize
        };

        let result = &self.blogs[start_seq..end_seq];
        if result.is_empty() {
            info!(
                "Blogs started at {} and ended at {} were not found. Return None",
                &start.0, &end.0
            );
            None
        } else {
            let mut blogs: Vec<BlogMetadata> = Vec::new();
            for blog in result {
                blogs.push(BlogMetadata {
                    id: blog.id.clone(),
                    name: blog.name.clone(),
                    filename: blog.filename.clone(),
                })
            }
            Some(blogs)
        }
    }
    async fn check_id(&self, id: BlogId) -> Option<BlogStored> {
        let result = self.blogs.iter().filter(|blog| &blog.id == &id).next();
        match result {
            Some(blog) => {
                info!("Blog {} is in Memory.", &blog.id);
                Some(BlogStored(true))
            }
            None => {
                info!("Blog {} is not in Memory.", &id);
                Some(BlogStored(false))
            }
        }
    }
    async fn add(
        &mut self,
        id: BlogId,
        name: String,
        filename: String,
        source: BlogSource,
        body: String,
    ) -> Option<Blog> {
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
        Some(result)
    }
    async fn delete(&mut self, id: BlogId) -> Option<BlogDeleted> {
        let result = self.blogs.iter().position(|blog| &blog.id == &id);
        match result {
            Some(val) => {
                info!("Deleting Blog with Id {}", &val);
                self.blogs.remove(val);
                info!("Deleted Blog with Id {}", &val);
                Some(BlogDeleted(true))
            }
            None => {
                error!("Failed to delete Blog with Id {}. Blog not found.", &id);
                None
            }
        }
    }
    async fn update(
        &mut self,
        id: BlogId,
        name: Option<String>,
        filename: Option<String>,
        source: Option<BlogSource>,
        body: Option<String>,
    ) -> Option<Blog> {
        let result: Option<&mut Blog> = self.blogs.iter_mut().filter(|blog| &blog.id == &id).next();

        match result {
            Some(blog) => {
                match name {
                    Some(val) => {
                        debug!("Update Blog {} name from {} to {}", &id, &blog.name, &val);
                        blog.name = val
                    }
                    None => (),
                }
                match filename {
                    Some(val) => {
                        debug!(
                            "Update Blog {} filename from {} to {}",
                            &id, &blog.filename, &val
                        );
                        blog.filename = val
                    }
                    None => (),
                }
                match source {
                    Some(val) => {
                        debug!(
                            "Update Blog {} source from {} to {}",
                            &id, &blog.source, &val
                        );
                        blog.source = val
                    }
                    None => (),
                }
                match body {
                    Some(val) => {
                        debug!("Update Blog {} body from {} to {}", &id, &blog.body, &val);
                        blog.body = val
                    }
                    None => (),
                }
                Some(blog.clone())
            }
            None => {
                error!("Failed to update Blog with Id {}. Blog not found.", &id);
                None
            }
        }
    }
}

impl MemoryBlogRepo {
    pub fn new() -> MemoryBlogRepo {
        let blogs: Vec<Blog> = Vec::new();
        MemoryBlogRepo { blogs }
    }
}

impl Default for MemoryBlogRepo {
    fn default() -> Self {
        MemoryBlogRepo::new()
    }
}
