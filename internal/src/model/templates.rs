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
    pub filename: &'a str,
    pub body: &'a str,
}

#[derive(Template, Debug)]
#[template(path = "version.html")]
pub struct Version<'a> {
    pub version: &'a str,
    pub environment: &'a str,
    pub build_hash: &'a str,
    pub build_date: &'a str,
}

#[derive(Template, Debug)]
#[template(path = "404_not_found.html")]
pub struct NotFound;

#[derive(Template, Debug)]
#[template(path = "500_internal_server_error.html")]
pub struct InternalServerError;
