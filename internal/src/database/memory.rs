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
                debug!("Blog HTML {:?}.", &blog.body);
                Some(blog.clone())
            }
            None => {
                info!("Blog {} not found. Return None", &id);
                None
            }
        }
    }
    async fn find_blogs(&self, query_params: BlogsParams) -> Option<Vec<BlogMetadata>> {
        let start = query_params.start.unwrap();
        let end = query_params.end.unwrap();
        let tags: Vec<String> = query_params
            .tags
            .unwrap()
            .split(",")
            .map(|tag| tag.to_string())
            .collect();

        let start_seq = if start.0 as usize > self.blogs.len() {
            warn!("BlogStartPage is greater than Blogs count. Will reset to 0.");
            0_i64
        } else {
            start.0
        };

        let end_seq = if (end.0 as usize > self.blogs.len()) && self.blogs.len() > 10 {
            warn!("BlogEndPage is greater than Blogs count. Will reset to Blogs count or 10, whichever is lesser.");
            10_i64
        } else if (end.0 as usize > self.blogs.len()) && self.blogs.len() < 10 {
            warn!("BlogEndPage is greater than Blogs count. Will reset to Blogs count or 10, whichever is lesser.");
            self.blogs.len() as i64
        } else if start.0 as usize > end.0 as usize {
            warn!("BlogStartPage is greater than BlogEndPage. Will reset to 10.");
            self.blogs.len() as i64
        } else {
            end.0
        };

        let result: &Vec<&Blog> = &self
            .blogs
            .iter()
            .filter(|blog| &blog.id.id >= &start_seq && &blog.id.id < &end_seq)
            .filter(|blog| {
                // Basically we need an OR operation to determine which tags
                // To be displayed. It's an OR operation because I want to show
                // multiple tags instead of find specific articles with the
                // matched tags
                let mut are_tags_matched = true;
                for tag in &tags {
                    match &blog.tags {
                        Some(blog_tags) => {
                            if !blog_tags.contains(tag) {
                                debug!("Tag: {} is not available in blog {}", &tag, &blog.id.id);
                                are_tags_matched = are_tags_matched || false;
                            }
                        }
                        None => continue,
                    }
                }
                are_tags_matched
            })
            .collect();
        // let result = &self.blogs[start_seq..end_seq];
        if result.is_empty() {
            info!(
                "Blogs started at {} and ended at {} were not found. Return None",
                &start.0, &end.0
            );
            None
        } else {
            debug!("Vector of BlogMetadata is created.");
            Some(
                result
                    .iter()
                    .map(|blog| BlogMetadata {
                        id: blog.id.clone(),
                        name: blog.name.clone().unwrap(),
                        filename: blog.filename.clone().unwrap(),
                        tags: blog.tags.clone().unwrap(),
                    })
                    .collect(),
            )
        }
    }
    async fn check_id(&self, id: BlogId) -> Option<BlogCommandStatus> {
        let result = self.blogs.iter().filter(|blog| &blog.id == &id).next();
        match result {
            Some(blog) => {
                info!("Blog {} is in Memory.", &blog.id);
                Some(BlogCommandStatus::Stored)
            }
            None => {
                info!("Blog {} is not in Memory.", &id);
                None
            }
        }
    }
    async fn add(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        self.blogs.push(blog.clone());
        info!("Blog {} added.", &blog.id);
        debug!("Blog HTML {:?}.", &blog.body);
        Some(BlogCommandStatus::Stored)
    }
    async fn delete(&mut self, id: BlogId) -> Option<BlogCommandStatus> {
        let result = self.blogs.iter().position(|blog| &blog.id == &id);
        match result {
            Some(val) => {
                info!("Deleting Blog with Id {}", &val);
                self.blogs.remove(val);
                info!("Deleted Blog with Id {}", &val);
                Some(BlogCommandStatus::Deleted)
            }
            None => {
                error!("Failed to delete Blog with Id {}. Blog not found.", &id);
                None
            }
        }
    }
    async fn update(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        let result: Option<&mut Blog> = self
            .blogs
            .iter_mut()
            .filter(|blog| &blog.id == &blog.id)
            .next();

        match result {
            Some(in_mem_blog) => {
                match blog.name {
                    Some(updated_name) => {
                        debug!(
                            "Update Blog {} name from {:?} to {}",
                            &blog.id, &in_mem_blog.name, &updated_name
                        );
                        in_mem_blog.name = Some(updated_name)
                    }
                    None => (),
                }
                match blog.filename {
                    Some(updated_filename) => {
                        debug!(
                            "Update Blog {} filename from {:?} to {}",
                            &blog.id, &in_mem_blog.filename, &updated_filename
                        );
                        in_mem_blog.filename = Some(updated_filename)
                    }
                    None => (),
                }
                match blog.source {
                    Some(updated_source) => {
                        debug!(
                            "Update Blog {} source from {:?} to {}",
                            &blog.id, &in_mem_blog.source, &updated_source
                        );
                        in_mem_blog.source = Some(updated_source)
                    }
                    None => (),
                }
                match blog.body {
                    Some(updated_body) => {
                        debug!(
                            "Update Blog {} body from {:?} to {}",
                            &blog.id, &in_mem_blog.body, &updated_body
                        );
                        in_mem_blog.body = Some(updated_body)
                    }
                    None => (),
                }
                match blog.tags {
                    Some(updated_tags) => {
                        debug!(
                            "Update Blog {} tags from {:?} to {:?}",
                            &blog.id, &in_mem_blog.tags, &updated_tags
                        );
                        in_mem_blog.tags = Some(updated_tags)
                    }
                    None => (),
                }
                Some(BlogCommandStatus::Updated)
            }
            None => {
                error!(
                    "Failed to update Blog with Id {}. Blog not found.",
                    &blog.id
                );
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
