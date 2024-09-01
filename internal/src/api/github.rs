use crate::model::blog::{Blog, BlogBody, BlogFilename, BlogId, BlogName, BlogSource};
use crate::model::github::{GithubTree, GithubTrees};
use crate::utils::capitalize;
use http_body_util::BodyExt;
use log::{debug, error, info};
use markdown::{to_html_with_options, Options};
use octocrab;
use regex::Regex;
use serde_json;
use std::num::IntErrorKind;

// pub struct MemoryGithubRepo {}
//
// impl MemoryGithubRepo {
//     pub fn new() -> MemoryGithubRepo {
//         MemoryGithubRepo {}
//     }
// }
//
// impl Default for MemoryGithubRepo {
//     fn default() -> Self {
//         MemoryGithubRepo::new()
//     }
// }

// #[async_trait]
// impl GithubRepo for MemoryGithubRepo {
/// find all()
/// An async function that
/// take String of repository owner
/// and String of repository repo
/// and String of repository branch
/// Return an Option of GithubTrees
///
/// Example:
/// let owner = "husni-zuhdi".to_string();
/// let repo = "husni-blog-resources".to_string();
/// let branch = "main".to_string();
/// let gh_trees = MemoryGithubRepo::new().find_all(owner, repo, branch).await?;
//     async fn find_all(
//         &self,
//         owner: GithubOwner,
//         repo: GithubRepository,
//         branch: GithubBranch,
//     ) -> Option<GithubTrees> {
//         let tree_endpoint = format!(
//             "https://api.github.com/repos/{}/{}/git/trees/{}",
//             &owner, &repo, &branch
//         );
//         let gh_trees = octocrab::instance()._get(tree_endpoint).await;
//
//         let trees_result = match gh_trees {
//             Ok(val) => {
//                 let body_bytes = val.into_body().collect().await.unwrap().to_bytes();
//                 let body_json = String::from_utf8(body_bytes.to_vec()).unwrap();
//                 let result: GithubTrees = serde_json::from_str(&body_json).unwrap();
//                 Some(result)
//             }
//             Err(err) => {
//                 error!("Failed to parse Github Trees result: {}", err);
//                 None
//             }
//         };
//
//         trees_result
//     }
// }

/// get_gh_blogs()
/// An async function that
/// take String of repository owner
/// and String of repository repo
/// and String of repository branch
/// Return an Option of GithubTrees
///
/// Example:
/// let owner = "husni-zuhdi".to_string();
/// let repo = "husni-blog-resources".to_string();
/// let branch = "main".to_string();
/// let gh_trees = get_gh_blogs(owner, repo, branch).await?;
pub async fn get_gh_blogs(owner: String, repo: String, branch: String) -> Option<Vec<Blog>> {
    let tree_endpoint = format!(
        "https://api.github.com/repos/{}/{}/git/trees/{}",
        &owner, &repo, &branch
    );
    let gh_trees = octocrab::instance()._get(tree_endpoint).await;

    let trees_result = match gh_trees {
        Ok(val) => {
            let body_bytes = val.into_body().collect().await.unwrap().to_bytes();
            let body_json = String::from_utf8(body_bytes.to_vec()).unwrap();
            let result: GithubTrees = serde_json::from_str(&body_json).unwrap();
            Some(result)
        }
        Err(err) => {
            error!("Failed to parse Github Trees result: {}", err);
            None
        }
    };

    let mut blog_trees: Vec<Blog> = Vec::new();
    match trees_result {
        Some(val) => {
            for tree in val.trees {
                let blog_res =
                    get_gh_blog(tree.clone(), owner.clone(), repo.clone(), branch.clone()).await;
                match blog_res {
                    Some(val) => blog_trees.push(val),
                    None => {
                        debug!("Skipped tree {:?}", &tree)
                    }
                }
            }
        }
        None => {
            error!("failed to filter Github Trees result")
        }
    };
    Some(blog_trees)
}

async fn get_gh_blog(
    tree: GithubTree,
    owner: String,
    repo: String,
    branch: String,
) -> Option<Blog> {
    let blog_path = tree.path;
    let gh_blog_link = format!(
        "https://github.com/{}/{}/tree/{}/{}",
        &owner, &repo, &branch, &blog_path
    );
    let gh_raw_blog_link = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        &owner, &repo, &branch, &blog_path
    );

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
                            let decoded_content = &content[0].decoded_content().unwrap().clone();

                            let name_formated = blog_name.replace("-", " ");
                            let name = capitalize(&name_formated);
                            info!("Markdown of {} loaded", &blog_name);

                            let body = process_gh_markdown(
                                decoded_content.to_string(),
                                gh_blog_link,
                                gh_raw_blog_link,
                            );
                            debug!("HTML Body of {}: {}", &blog_name, &body);

                            let id = format!("{}-g", blog_id).to_string();
                            let filename = format!(
                                "https://api.github.com/repos/{}/{}/contents/{}",
                                &owner, &repo, &blog_readme_path
                            )
                            .to_string();

                            Some(Blog {
                                id: BlogId(id),
                                name: BlogName(name),
                                source: BlogSource::Github,
                                filename: BlogFilename(filename),
                                body: BlogBody(body),
                            })
                        }
                        Err(err) => {
                            error!(
                                "Failed to get Blog content with Blog ID {} and Name {}: {}",
                                &blog_id, &blog_name, err
                            );
                            None
                        }
                    }
                } else {
                    debug!("Folder prefix is 000. Skip this folder");
                    None
                }
            }
            Err(err) => {
                if err.kind() == &IntErrorKind::InvalidDigit {
                    debug!("Error Kind {:?}. Safe to ignore.", err.kind());
                }
                error!("Failed to parse Blog ID: {}", err);
                None
            }
        }
    } else {
        info!("This is not a folder. Skip this tree");
        None
    }
}

fn process_gh_markdown(markdown: String, gh_blog_link: String, gh_raw_blog_link: String) -> String {
    let raw_body = to_html_with_options(&markdown, &Options::gfm())
        .expect("Failed to convert html with options");
    let body = replace_gh_link(raw_body, gh_blog_link, gh_raw_blog_link);
    body
}

/// replace_gh_link
/// Replace Github Blog relative links
/// with full github content links
/// Take String of markdown body
/// and String of github blog endpoint
/// then return String of updated body
fn replace_gh_link(body: String, gh_blog_link: String, gh_raw_blog_link: String) -> String {
    // Regex href=.\.\/ mean
    // find string with character 'href='
    // then followed by any character (I tried to use '"' but didn't work)
    // then followed by '.' (must use escape character)
    // then followed by '/' (must use escape character)
    let re_href = Regex::new(r"href=.\.\/").expect("Failed to build regex href");

    let replaced_str_href = format!("href=\"{}/", gh_blog_link);
    debug!("Replaced str: {}", &replaced_str_href);

    let res_href = re_href
        .replace_all(body.as_str(), replaced_str_href.as_str())
        .to_string();
    debug!("Replaced Body: {}", &res_href);

    // Regex src=.\.\/ mean
    // find string with character 'src='
    // then followed by any character (I tried to use '"' but didn't work)
    // then followed by '.' (must use escape character)
    // then followed by '/' (must use escape character)
    let re_src = Regex::new(r"src=.\.\/").expect("Failed to build regex src");

    let replaced_str_src = format!("src=\"{}/", gh_raw_blog_link);
    debug!("Replaced str: {}", &replaced_str_src);

    let res = re_src
        .replace_all(res_href.as_str(), replaced_str_src.as_str())
        .to_string();
    debug!("Replaced Body: {}", &res);

    res
}
