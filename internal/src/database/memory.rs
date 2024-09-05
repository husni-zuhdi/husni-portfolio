// use crate::api::github::get_gh_blogs;
use crate::model::blog::{
    Blog, BlogBody, BlogDeleted, BlogEndPage, BlogFilename, BlogId, BlogName, BlogSource,
    BlogStartPage, BlogStored,
};
use crate::repo::blog::BlogRepo;
use async_trait::async_trait;
use log::{debug, info, warn};

#[derive(Clone)]
pub struct MemoryBlogRepo {
    pub blogs: Vec<Blog>,
}

#[async_trait]
impl BlogRepo for MemoryBlogRepo {
    async fn find(&self, id: BlogId) -> Blog {
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
    async fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage) -> Vec<Blog> {
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
        } else {
            end.0 as usize
        };

        let result = &self.blogs[start_seq..end_seq];
        result.to_vec()
    }
    async fn check_id(&self, id: BlogId) -> BlogStored {
        let result = self.blogs.iter().filter(|blog| &blog.id == &id).next();
        match result {
            Some(blog) => {
                info!("Blog {} is in Memory.", &blog.id.0);
                BlogStored(true)
            }
            None => {
                info!("Blog {} is not in Memory.", &id.0);
                BlogStored(false)
            }
        }
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
        let blogs: Vec<Blog> = Vec::new();
        MemoryBlogRepo { blogs }
    }
}

impl Default for MemoryBlogRepo {
    fn default() -> Self {
        MemoryBlogRepo::new()
    }
}
