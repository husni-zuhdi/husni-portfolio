use crate::model::templates::*;
use actix_files::NamedFile;
use actix_web::{web, Responder, Result};
use actix_web_lab::respond::Html;
use askama::Template;

pub async fn styles() -> Result<NamedFile> {
    Ok(NamedFile::open("./statics/styles.css")?)
}

pub async fn profile() -> Result<impl Responder> {
    let profile = Profile.render().expect("Failed to render profile.html");
    Ok(Html(profile))
}
