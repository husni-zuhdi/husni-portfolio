use crate::model::data::{BlogData, BlogDataType, Trees};
use crate::utils::{capitalize, md_to_html};
use http_body_util::BodyExt;
use log::{error, info};
use octocrab;
use serde_json;
use std::num::IntErrorKind;

/// Testing 1
/// Nge get readme dulu
pub async fn get_blog_readme() -> String {
    let readme = octocrab::instance()
        .repos("husni-zuhdi", "husni-blog-resources")
        .get_readme()
        .path("README.md")
        .r#ref("main")
        .send()
        .await
        .expect("Failed to get readme content");
    readme.name
}

/// get_gh_blog_data()
/// An async function that
/// take String of repository owner
/// and String of repository repo
/// and String of repository branch
/// Return an Option of Vector of BlogData
///
/// Example:
/// let owner = "husni-zuhdi".to_string();
/// let repo = "husni-blog-resources".to_string();
/// let branch = "main".to_string();
/// let gh_blog_data = get_gh_blog_list(owner, repo, branch).await?;
pub async fn get_gh_blog_data(
    owner: String,
    repo: String,
    branch: String,
) -> Option<Vec<BlogData>> {
    let tree_endpoint = format!(
        "https://api.github.com/repos/{}/{}/git/trees/{}",
        &owner, &repo, &branch
    );
    let gh_trees = octocrab::instance()._get(tree_endpoint).await;

    let trees_result = match gh_trees {
        Ok(val) => {
            let body_bytes = val.into_body().collect().await.unwrap().to_bytes();
            let body_json = String::from_utf8(body_bytes.to_vec()).unwrap();
            let result: Trees = serde_json::from_str(&body_json).unwrap();
            Some(result)
        }
        Err(err) => {
            error!("Failed to parse Github Trees result: {}", err);
            None
        }
    };

    let mut blog_trees: Vec<BlogData> = Vec::new();
    match trees_result {
        Some(val) => {
            for tree in val.tree {
                let blog_path = tree.path;

                // Check to make sure the path doesn't have a extention
                if !blog_path.contains(".") {
                    // Get blog id with specification of 3 digit integer
                    let blog_id = blog_path.get(0..3).unwrap();
                    let blog_name = blog_path.get(4..).unwrap();

                    match blog_id.parse::<i32>() {
                        Ok(_) => {
                            if &blog_id != &"000" {
                                info!("Blog Name: {}", &blog_name);
                                let blog_readme_path = format!("{}/README.md", &blog_path);
                                let blog_content = octocrab::instance()
                                    .repos(&owner, &repo)
                                    .get_content()
                                    .path(&blog_readme_path)
                                    .r#ref(&branch)
                                    .send()
                                    .await;
                                match blog_content {
                                    Ok(mut res) => {
                                        let content = res.take_items();
                                        let decoded_content =
                                            &content[0].decoded_content().unwrap().clone();

                                        let name_formated = blog_name.replace("-", " ");
                                        let name = capitalize(&name_formated);
                                        info!("Markdown of {} loaded", &blog_name);

                                        let body =
                                            md_to_html(None, Some(decoded_content.to_string()))
                                                .expect("Failed to convert markdown to html");
                                        info!("HTML Body of {}: {}", &blog_name, &body);

                                        blog_trees.push(BlogData {
                                            id: format!("{}-g", blog_id).to_string(),
                                            name,
                                            source: BlogDataType::Github,
                                            filename: format!(
                                                "https://api.github.com/repos/{}/{}/contents/{}",
                                                &owner, &repo, &blog_readme_path
                                            )
                                            .to_string(),
                                            body,
                                        })
                                    }
                                    Err(err) => {
                                        error!(
                                            "Failed to get Blog content with Blog ID {} and Name {}: {}",
                                            &blog_id, &blog_name, err
                                        )
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            if err.kind() == &IntErrorKind::InvalidDigit {
                                continue;
                            }
                            println!("Failed to parse Blog ID: {}", err);
                        }
                    };
                }
            }
        }
        None => {
            error!("failed to filter Github Trees result")
        }
    };
    Some(blog_trees)
}

// Test n
// Nge get semua markdown yang ada di repo
// pub async fn get_github_blogs() -> Vec<BlogData> {
//     let repo = octocrab::instance()
//         .repos("husni-zuhdi", "husni-blog-resources")
//         .get()
//         .await
//         .expect("Failed to fetch blog resources repo");
//     repo.contents_url.expect("Failed to get contents url")
// }
