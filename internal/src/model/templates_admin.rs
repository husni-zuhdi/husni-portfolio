use askama::Template;

#[derive(Template, Debug)]
#[template(path = "admin/talks.html")]
pub struct AdminTalksTemplate {
    pub talks: Vec<AdminTalkTemplate>,
}

#[derive(Template, Debug)]
#[template(path = "admin/get_edit_talk.html")]
pub struct AdminGetEditTalkTemplate {
    pub talk: AdminTalkTemplate,
}

#[derive(Template, Debug)]
#[template(path = "admin/get_talk.html")]
pub struct AdminGetTalkTemplate {
    pub talk: AdminTalkTemplate,
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
