use std::fs;

use crate::model::{data::*, templates::*};
use crate::utils::md_to_html;
use actix_files::NamedFile;
use actix_web::{web, Responder, Result};
use actix_web_lab::respond::Html;
use askama::Template;
use log::info;
use markdown::*;

pub async fn styles() -> Result<NamedFile> {
    Ok(NamedFile::open("./statics/styles.css")?)
}

pub async fn profile() -> Result<impl Responder> {
    let profile = Profile.render().expect("Failed to render profile.html");
    Ok(Html(profile))
}

pub async fn blogs(blogs_data: web::Data<BlogsData>) -> Result<impl Responder> {
    // Copy data to Template struct
    let blogs_template: Vec<Blog> = blogs_data
        .blogs
        .iter()
        .map(|blog| Blog {
            id: &blog.id,
            name: &blog.name,
            filename: &blog.filename,
            body: "",
        })
        .collect();

    let blogs = Blogs {
        blogs: &blogs_template,
    }
    .render()
    .expect("Failed to render blogs.html");
    info!("Blogs Template created");
    Ok(Html(blogs))
}

pub async fn get_blog(
    path: web::Path<String>,
    blogs_data: web::Data<BlogsData>,
) -> Result<impl Responder> {
    let blog_id = path.into_inner();
    let blog_data = blogs_data
        .blogs
        .iter()
        .filter(|blog| blog.id == blog_id)
        .next()
        .expect("Failed to get blog name with id {blog_id}");

    let body = md_to_html(blog_data.filename.clone())
        .await
        .expect("Failed to render markdown to html");

    let blog = Blog {
        id: &blog_id,
        name: &blog_data.name,
        filename: &blog_data.filename,
        body: &body,
    }
    .render()
    .expect("Failed to render blog.html");
    Ok(Html(blog))
}
