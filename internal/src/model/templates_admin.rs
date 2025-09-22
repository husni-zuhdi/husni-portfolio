use askama::Template;

use crate::model::{tags::Tag, templates::BlogMetadataTemplate};

#[derive(Template, Debug)]
#[template(path = "admin/talks/talks.html")]
pub struct AdminTalksTemplate {}

#[derive(Template, Debug)]
#[template(path = "admin/talks/get_add_talk.html")]
pub struct AdminGetAddTalkTemplate {
    pub id: i64,
    pub date: String,
}

#[derive(Template, Debug)]
#[template(path = "admin/talks/get_edit_talk.html")]
pub struct AdminGetEditTalkTemplate {
    pub talk: AdminTalkTemplate,
}

#[derive(Template, Debug)]
#[template(path = "admin/talks/get_delete_talk.html")]
pub struct AdminGetDeleteTalkTemplate {
    pub id: i64,
}

#[derive(Template, Debug)]
#[template(path = "admin/talks/get_talk.html")]
pub struct AdminGetTalkTemplate {
    pub talk: AdminTalkTemplate,
}

#[derive(Template, Debug)]
#[template(path = "admin/talks/list_talks.html")]
pub struct AdminListTalksTemplate {
    pub talks: Vec<AdminTalkTemplate>,
}

#[derive(Debug)]
pub struct AdminTalkTemplate {
    pub id: i64,
    pub name: String,
    pub date: String,
    pub media_link: String,
    pub org_name: String,
    pub org_link: String,
}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/blogs.html")]
pub struct AdminBlogsTemplate {
    pub blogs: Vec<BlogMetadataTemplate>,
    pub active_tags: Vec<String>,
}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/get_blogs.html")]
pub struct AdminGetBlogsTemplate {
    pub blogs: Vec<BlogMetadataTemplate>,
    pub active_tags: Vec<String>,
}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/get_add_blog.html")]
pub struct AdminGetAddBlogTemplate {
    pub id: i64,
    pub avail_tags: Vec<String>,
}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/get_edit_blog.html")]
pub struct AdminGetEditBlogTemplate {
    pub id: i64,
    pub name: String,
    pub body: String,
    pub blog_tags: Vec<String>,
    pub avail_tags: Vec<String>,
}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/get_blog.html")]
pub struct AdminGetBlogTemplate {
    pub blog: BlogMetadataTemplate,
}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/get_delete_blog.html")]
pub struct AdminGetDeleteBlogTemplate {
    pub id: i64,
}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/tags/tags.html")]
pub struct AdminBlogTagsTemplate {}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/tags/list_tags.html")]
pub struct AdminBlogTagsListTemplate {
    pub tags: Vec<Tag>,
}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/tags/get_add_tag.html")]
pub struct AdminGetAddTagTemplate {
    pub id: i64,
}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/tags/get_tag.html")]
pub struct AdminGetTagTemplate {
    pub id: i64,
    pub name: String,
}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/tags/get_edit_tag.html")]
pub struct AdminGetEditTagTemplate {
    pub id: i64,
    pub name: String,
}

#[derive(Template, Debug)]
#[template(path = "admin/blogs/tags/get_delete_tag.html")]
pub struct AdminGetDeleteTagTemplate {
    pub id: i64,
}
