use askama::Template;

#[derive(Template, Debug)]
#[template(path = "profile.html")]
pub struct Profile;

#[derive(Template, Debug)]
#[template(path = "blogs.html")]
pub struct Blogs<'a> {
    pub blogs: &'a Vec<Blog<'a>>,
}

#[derive(Template, Debug)]
#[template(path = "blog.html")]
pub struct Blog<'a> {
    pub id: &'a str,
    pub name: &'a str,
}
