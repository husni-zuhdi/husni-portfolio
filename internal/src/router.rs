use crate::config::Config;
use crate::model::{data::*, templates::*};
use crate::utils::read_version_manifest;
use actix_files::NamedFile;
use actix_web::{web, Responder, Result};
use actix_web_lab::respond::Html;
use askama::Template;
use log::info;

pub async fn styles() -> Result<NamedFile> {
    let styles = NamedFile::open("./statics/styles.css").expect("Failed to render styles.css");
    Ok(styles)
}

pub async fn profile() -> Result<impl Responder> {
    let profile = Profile.render().expect("Failed to render profile.html");
    Ok(Html(profile))
}

pub async fn get_blogs(blogs_data: web::Data<BlogsData>) -> Result<impl Responder> {
    // Copy data to Template struct
    let blogs_template: Vec<Blog> = blogs_data
        .blogs
        .iter()
        .map(|blog| Blog {
            id: &blog.id,
            name: &blog.name,
            filename: &blog.filename,
            body: &blog.body,
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

    let blog = Blog {
        id: &blog_id,
        name: &blog_data.name,
        filename: &blog_data.filename,
        body: &blog_data.body,
    }
    .render()
    .expect("Failed to render blog.html");
    Ok(Html(blog))
}

pub async fn get_version(config: web::Data<Config>) -> Result<impl Responder> {
    let version_json = read_version_manifest().expect("Failed to get version manifest");
    let version = Version {
        version: version_json.version.as_str(),
        environment: config.environment.as_str(),
        build_hash: version_json.build_hash.as_str(),
        build_date: version_json.build_date.as_str(),
    }
    .render()
    .expect("Failed to render version.html");
    info!("Version Template created");

    Ok(Html(version))
}
