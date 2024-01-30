use askama::Template;

#[derive(Template)]
#[template(path = "profile.html")]
pub struct Profile;
