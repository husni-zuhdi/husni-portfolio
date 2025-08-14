use crate::model::blogs::{Blog, BlogId, BlogMetadata, BlogSource};
use crate::model::github::{GithubBranch, GithubOwner, GithubRepository, GithubTree, GithubTrees};
use crate::repo::api::ApiRepo;
use crate::utils::capitalize;
use async_trait::async_trait;
use http_body_util::BodyExt;
use markdown::{to_html_with_options, Options};
use octocrab;
use octocrab::models::repos::Content;
use regex::Regex;
use serde_json;
use std::num::IntErrorKind;
use tracing::{debug, error, info, warn};

#[derive(Clone)]
pub struct GithubApiUseCase {
    pub github_owner: GithubOwner,
    pub github_repo: GithubRepository,
    pub github_branch: GithubBranch,
}

#[async_trait]
impl ApiRepo for GithubApiUseCase {
    async fn list_metadata(&self) -> Option<Vec<BlogMetadata>> {
        let trees_result = Self::fetch_github_trees(self).await;

        let mut blogs_metadata: Vec<BlogMetadata> = Vec::new();
        match trees_result {
            Some(github_trees) => {
                for tree in github_trees.trees {
                    let blog_metadata = Self::process_github_metadata(self, tree.clone()).await;
                    match blog_metadata {
                        Some(metadata) => blogs_metadata.push(metadata),
                        None => {
                            debug!("Skipped tree with path {}", &tree.path)
                        }
                    }
                }
            }
            None => {
                error!("Failed to filter Github Trees result")
            }
        };
        Some(blogs_metadata)
    }
    async fn fetch(&self, metadata: BlogMetadata) -> Option<Blog> {
        let result = Self::fetch_github_content(self, metadata.filename.clone()).await;

        match result {
            Some(content) => Self::process_github_content(self, content, metadata),
            None => {
                error!(
                    "Failed to get Blog content with Blog ID {} and Name {}: File Not Found",
                    &metadata.id, &metadata.name
                );
                None
            }
        }
    }
}

impl GithubApiUseCase {
    pub async fn new(
        github_owner: String,
        github_repo: String,
        github_branch: String,
    ) -> GithubApiUseCase {
        GithubApiUseCase {
            github_owner: GithubOwner(github_owner),
            github_repo: GithubRepository(github_repo),
            github_branch: GithubBranch(github_branch),
        }
    }
    /// Fetch Github trees
    /// Based on repository data from the GithubApiUseCase fields
    /// Returned Optional GithubTrees
    async fn fetch_github_trees(&self) -> Option<GithubTrees> {
        let trees_endpoint = format!(
            "https://api.github.com/repos/{}/{}/git/trees/{}",
            self.github_owner, self.github_repo, self.github_branch
        );
        let github_trees = octocrab::instance()._get(trees_endpoint).await;
        
        match github_trees {
            Ok(github_trees) => {
                let body_bytes = github_trees.into_body().collect().await.unwrap().to_bytes();
                let body_json = String::from_utf8(body_bytes.to_vec()).unwrap();
                let result: GithubTrees = serde_json::from_str(&body_json).unwrap();
                Some(result)
            }
            Err(err) => {
                error!("Failed to parse Github Trees result: {}", err);
                None
            }
        }
    }
    /// Get blog_id with specification of 3 digit integer and blog_name
    /// Return an optional 2 string for blog_id and blog_name
    fn create_tree_id_and_name(tree_path: String) -> Option<(String, String)> {
        let blog_id = tree_path.get(0..3).unwrap().to_string();
        let blog_name = tree_path.get(4..).unwrap().to_string();
        Some((blog_id, blog_name))
    }
    /// Process Github Metadata from a GithubTree
    /// Returned Optional BlogMetadata
    async fn process_github_metadata(&self, tree: GithubTree) -> Option<BlogMetadata> {
        let filename = format!(
            "https://api.github.com/repos/{}/{}/contents/{}/README.md",
            self.github_owner, self.github_repo, &tree.path
        )
        .to_string();

        let (blog_id, blog_name) = Self::create_tree_id_and_name(tree.path.0.clone())
            .expect("Failed to spearate Blog id and name");
        let tree_is_dir = !tree.path.0.contains(".");
        // Main Infrastructure is the base-level step to replicate
        // all infrastructure from `husni-blog-resource`
        // Ref: https://github.com/husni-zuhdi/husni-blog-resources/tree/main/000-main-infrastructure
        let blog_id_is_not_main_infra = blog_id != *"000";

        if tree_is_dir {
            match blog_id.parse::<i64>() {
                Ok(id) => {
                    if blog_id_is_not_main_infra {
                        // let id = format!("{}-g", blog_id);
                        // let id = format!("{}", blog_id);
                        info!(
                            "Blog Metadata for Id {} and Name {} is processed",
                            &id, &blog_name
                        );

                        Some(BlogMetadata {
                            id: BlogId { id },
                            name: blog_name,
                            filename,
                            // TODO: remove the empty tags
                            tags: vec!["".to_string()],
                        })
                    } else {
                        debug!("Folder prefix is 000-main-infrastructure. Skip this folder");
                        None
                    }
                }
                Err(err) => {
                    if err.kind() == &IntErrorKind::InvalidDigit {
                        debug!("Error Kind {:?}. Skipped.", err.kind());
                    }
                    warn!(
                        "Failed to parse Tree Path {}. Error {:?}. Skipped",
                        &tree.path,
                        err.kind()
                    );
                    None
                }
            }
        } else {
            debug!("Tree {} is not a directory. Skipped.", &tree.path);
            None
        }
    }
    /// Fetch Github Content
    /// Take a filename with type BlogFilename (should be url instead?)
    /// Returned Optional octocrab::models::Content
    async fn fetch_github_content(&self, url: String) -> Option<Content> {
        let github_content = octocrab::instance()._get(url.clone()).await;
        
        match github_content {
            Ok(content) => {
                let body_bytes = content.into_body().collect().await.unwrap().to_bytes();
                let body_json = String::from_utf8(body_bytes.to_vec()).unwrap();
                let result: Content = serde_json::from_str(&body_json).unwrap();
                Some(result)
            }
            Err(err) => {
                error!(
                    "Failed to parse Github Content for filename {}: {}",
                    &url, err
                );
                None
            }
        }
    }
    /// Process Content Markdown
    /// Included replace Github Blog relative links with full github content links
    /// Take String of markdown body
    /// and String of github blog endpoint and github raw blog link
    /// then return an optional string of processed markdown
    fn process_content_markdown(
        markdown: String,
        gh_blog_link: String,
        gh_raw_blog_link: String,
    ) -> Option<String> {
        let raw_body = to_html_with_options(&markdown, &Options::gfm())
            .expect("Failed to convert html with options");
        // Regex href=.\.\/ mean
        // find string with character 'href='
        // then followed by any character (I tried to use '"' but didn't work)
        // then followed by '.' (must use escape character)
        // then followed by '/' (must use escape character)
        let re_href = Regex::new(r"href=.\.\/").expect("Failed to build regex href");

        let replaced_str_href = format!("href=\"{}/", gh_blog_link);
        debug!("Replaced str: {}", &replaced_str_href);

        let res_href = re_href
            .replace_all(raw_body.as_str(), replaced_str_href.as_str())
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

        let body = re_src
            .replace_all(res_href.as_str(), replaced_str_src.as_str())
            .to_string();
        debug!("Replaced Body: {}", &body);
        Some(body)
    }
    /// Process Github Content and Metadata
    /// Returned an optional Blog
    fn process_github_content(&self, content: Content, metadata: BlogMetadata) -> Option<Blog> {
        let gh_blog_link = format!(
            "https://github.com/{}/{}/tree/{}/{}-{}",
            self.github_owner, self.github_repo, self.github_branch, &metadata.id, &metadata.name
        );
        let gh_raw_blog_link = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}-{}",
            self.github_owner, self.github_repo, self.github_branch, &metadata.id, &metadata.name
        );

        let name_formated = metadata.name.replace("-", " ");
        let name = capitalize(&name_formated);

        info!(
            "Markdown of Blog id {} with name {} loaded",
            &metadata.id, &name
        );

        let markdown = content.decoded_content().unwrap();
        let body = Self::process_content_markdown(markdown, gh_blog_link, gh_raw_blog_link)
            .expect("Failed to process content body");

        debug!("HTML Body of {}: {}", &metadata.name, &body);

        Some(Blog {
            id: metadata.id,
            name: Some(name),
            source: Some(BlogSource::Github),
            filename: Some(metadata.filename),
            body: Some(body),
            // Set empty tags for non-database
            tags: Some(vec!["".to_string()]),
        })
    }
}
