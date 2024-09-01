use crate::model::blog::{
    Blog, BlogBody, BlogDeleted, BlogEndPage, BlogFilename, BlogId, BlogName, BlogSource,
    BlogStartPage,
};
use crate::port::blog::{command::BlogQueryCommand, query::BlogQueryPort};
use crate::repo::blog::BlogRepo;

#[derive(Clone)]
pub struct BlogUseCase {
    pub blog_repo: Box<dyn BlogRepo + Send>,
}

impl BlogQueryPort for BlogUseCase {
    fn find(&self, id: BlogId) -> Blog {
        self.blog_repo.find(id)
    }
    fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage) -> Vec<Blog> {
        self.blog_repo.find_blogs(start, end)
    }
    fn find_all(&self) -> Vec<Blog> {
        self.blog_repo.find_all()
    }
}

impl BlogQueryCommand for BlogUseCase {
    fn update(
        &mut self,
        id: BlogId,
        name: Option<BlogName>,
        filename: Option<BlogFilename>,
        source: Option<BlogSource>,
        body: Option<BlogBody>,
    ) -> Blog {
        self.blog_repo.update(id, name, filename, source, body)
    }
    fn delete(&mut self, id: BlogId) -> BlogDeleted {
        self.blog_repo.delete(id)
    }
}

impl BlogUseCase {
    pub fn new(blog_repo: Box<dyn BlogRepo + Send>) -> BlogUseCase {
        BlogUseCase { blog_repo }
    }
}
