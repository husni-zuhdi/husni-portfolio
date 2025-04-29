use askama::Template;

#[derive(Template, Debug)]
#[template(path = "profile.html")]
pub struct ProfileTemplate;

#[derive(Template, Debug)]
#[template(path = "blogs.html")]
pub struct BlogsTemplate {
    pub blogs: Vec<BlogMetadataTemplate>,
    pub active_tags: Vec<String>,
}

#[derive(Debug)]
pub struct BlogMetadataTemplate {
    pub id: i64,
    pub name: String,
    pub tags: Vec<String>,
}

#[derive(Template, Debug)]
#[template(path = "blog.html")]
pub struct BlogTemplate {
    pub id: i64,
    pub name: String,
    pub filename: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Template, Debug)]
#[template(path = "version.html")]
pub struct VersionTemplate {
    pub version: String,
    pub environment: String,
    pub build_hash: String,
    pub build_date: String,
}

#[derive(Template, Debug)]
#[template(path = "talks.html")]
pub struct TalksTemplate {
    pub talks: Vec<TalkTemplate>,
}

#[derive(Debug)]
pub struct TalkTemplate {
    pub id: i64,
    pub name: String,
    pub date: String,
    pub media_link: String,
    pub org_name: String,
    pub org_link: String,
}

#[derive(Template, Debug)]
#[template(path = "404_not_found.html")]
pub struct NotFoundTemplate;

#[derive(Template, Debug)]
#[template(path = "500_internal_server_error.html")]
pub struct InternalServerErrorTemplate;
