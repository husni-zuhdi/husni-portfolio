use crate::model::blog::{Blog, BlogEndPage, BlogId, BlogStartPage};

pub trait BlogQueryPort {
    fn find(&self, id: BlogId) -> Blog;
    fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage) -> Vec<Blog>;
    fn find_all(&self) -> Vec<Blog>;
}
