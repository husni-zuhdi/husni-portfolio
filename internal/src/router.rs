use crate::model::templates::*;
use actix_files::NamedFile;
use actix_web::{web, Responder, Result};
use actix_web_lab::respond::Html;
use askama::Template;
use log::info;
use std::fs;

pub async fn styles() -> Result<NamedFile> {
    Ok(NamedFile::open("./statics/styles.css")?)
}

pub async fn profile() -> Result<impl Responder> {
    let profile = Profile.render().expect("Failed to render profile.html");
    Ok(Html(profile))
}

pub async fn blogs() -> Result<impl Responder> {
    let static_path = fs::read_dir("./statics/blogs/").unwrap();

    let blogs_paths: Vec<String> = static_path
        .filter_map(|blog_path| {
            let path = blog_path.ok().expect("Failed to get blog path").path();
            if path.is_file() {
                path.file_name()
                    .expect("Failed to get filename")
                    .to_str()
                    .map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();

    let blogs: Vec<Blog> = blogs_paths
        .iter()
        .map(|blog_path| {
            let (id, name) = blog_path
                .split_once("-")
                .expect("Failed to split filename into id and name");
            Blog { id, name }
        })
        .collect();

    info!("Blogs: {:?}", blogs);

    let blogs = Blogs { blogs: &blogs }
        .render()
        .expect("Failed to render blogs.html");
    Ok(Html(blogs))
}

pub async fn get_blog(path: web::Path<String>) -> Result<impl Responder> {
    let blogid = path.into_inner();
    let blog = Blog {
        id: &blogid,
        name: "test",
    }
    .render()
    .expect("Failed to render blog.html");
    Ok(Html(blog))
}
