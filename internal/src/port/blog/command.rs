use crate::model::blog::{Blog, BlogBody, BlogDeleted, BlogFilename, BlogId, BlogName, BlogSource};

pub trait BlogQueryCommand {
    fn update(
        &mut self,
        id: BlogId,
        name: Option<BlogName>,
        filename: Option<BlogFilename>,
        source: Option<BlogSource>,
        body: Option<BlogBody>,
    ) -> Blog;
    fn delete(&mut self, id: BlogId) -> BlogDeleted;
}
