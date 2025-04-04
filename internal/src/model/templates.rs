use askama::Template;

#[derive(Template, Debug)]
#[template(path = "profile.html")]
pub struct ProfileTemplate;

#[derive(Template, Debug)]
#[template(path = "blogs.html")]
pub struct BlogsTemplate<'a> {
    pub blogs: &'a Vec<BlogsTemplateBlog<'a>>,
}

#[derive(Debug)]
pub struct BlogsTemplateBlog<'a> {
    pub id: &'a str,
    pub name: &'a str,
}

#[derive(Template, Debug)]
#[template(path = "blog.html")]
pub struct BlogTemplate<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub filename: &'a str,
    pub body: &'a str,
}

#[derive(Template, Debug)]
#[template(path = "version.html")]
pub struct VersionTemplate<'a> {
    pub version: &'a str,
    pub environment: &'a str,
    pub build_hash: &'a str,
    pub build_date: &'a str,
}

#[derive(Template, Debug)]
#[template(path = "talks.html")]
pub struct TalksTemplate;

#[derive(Template, Debug)]
#[template(path = "404_not_found.html")]
pub struct NotFoundTemplate;

#[derive(Template, Debug)]
#[template(path = "500_internal_server_error.html")]
pub struct InternalServerErrorTemplate;
