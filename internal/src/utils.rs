use crate::model::data::*;
use log::info;
use markdown::to_html;
use std::fs;

pub async fn create_blogs() -> Result<BlogsData, String> {
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

    let blogs: Vec<BlogData> = blogs_paths
        .iter()
        .map(|blog_path| {
            let (id, name) = blog_path
                .split_once("-")
                .expect("Failed to split filename into id and name");
            BlogData {
                id: id.to_string(),
                name: name.to_string(),
                filename: blog_path.to_owned(),
                body: "".to_string(),
            }
        })
        .collect();

    info!("Blogs: {:?}", blogs);

    Ok(BlogsData { blogs })
}

pub async fn md_to_html(filename: String) -> Result<String, String> {
    let full_path = format!("./statics/blogs/{}", &filename);
    let body_md = fs::read_to_string(full_path).expect("Failed to read markdown blog file");
    Ok(to_html(&body_md))
}
