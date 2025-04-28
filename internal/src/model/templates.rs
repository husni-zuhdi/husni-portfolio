use askama::Template;

#[derive(Template, Debug)]
#[template(path = "profile.html")]
pub struct ProfileTemplate;

#[derive(Template, Debug)]
#[template(path = "blogs.html")]
pub struct BlogsTemplate {
    pub blogs: Vec<BlogMetadataTemplate>,
}
// pub struct BlogsTemplate<'a> {
//     pub blogs: &'a Vec<BlogMetadataTemplate<'a>>,
// }

#[derive(Debug)]
pub struct BlogMetadataTemplate {
    pub id: i64,
    pub name: String,
    pub tags: Vec<String>,
}
// pub struct BlogMetadataTemplate<'a> {
//     pub id: &'a i64,
//     pub name: &'a str,
//     pub tags: &'a Vec<&'a str>,
// }

#[derive(Template, Debug)]
#[template(path = "blog.html")]
pub struct BlogTemplate<'a> {
    pub id: &'a i64,
    pub name: &'a str,
    pub filename: &'a str,
    pub body: &'a str,
    pub tags: &'a Vec<&'a str>,
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
pub struct TalksTemplate<'a> {
    pub talks: &'a Vec<TalkTemplate<'a>>,
}

#[derive(Debug)]
pub struct TalkTemplate<'a> {
    pub id: &'a i64,
    pub name: &'a str,
    pub date: &'a str,
    pub media_link: &'a str,
    pub org_name: &'a str,
    pub org_link: &'a str,
}

#[derive(Template, Debug)]
#[template(path = "404_not_found.html")]
pub struct NotFoundTemplate;

#[derive(Template, Debug)]
#[template(path = "500_internal_server_error.html")]
pub struct InternalServerErrorTemplate;
