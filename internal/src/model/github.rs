use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Github Owner Name
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GithubOwner(String);

impl Display for GithubOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Github Owner Repository
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GithubRepository(String);

impl Display for GithubRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Github Owner Branch
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GithubBranch(String);

impl Display for GithubBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Collection of Tree of github blog data
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GithubTrees {
    pub trees: Vec<GithubTree>,
}

/// The file mode one of
/// 100644 for file (blob)
/// 100755 for executable (blob)
/// 040000 for subdirectory (tree)
/// 160000 for submodule (commit)
/// 120000 for a blob that specifies the path of a symlink.
/// Reference: https://docs.github.com/en/rest/git/trees?apiVersion=2022-11-28
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum GithubTreeMode {
    #[serde(rename(deserialize = "100644"))]
    File,
    #[serde(rename(deserialize = "100755"))]
    Executable,
    #[serde(rename(deserialize = "040000"))]
    SubDir,
    #[serde(rename(deserialize = "160000"))]
    SubModeule,
    #[serde(rename(deserialize = "120000"))]
    Symlink,
}

/// Either blob, tree, or commit.
/// Reference: https://docs.github.com/en/rest/git/trees?apiVersion=2022-11-28
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum GithubTreeType {
    #[serde(rename(deserialize = "blob"))]
    Blob,
    #[serde(rename(deserialize = "tree"))]
    Tree,
    #[serde(rename(deserialize = "commit"))]
    Commit,
}

/// Tree structure of git
/// Reference: https://docs.github.com/en/rest/git/trees?apiVersion=2022-11-28
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GithubTree {
    pub path: String,
    #[serde(rename(deserialize = "mode"))]
    pub tree_mode: GithubTreeMode,
    #[serde(rename(deserialize = "type"))]
    pub tree_type: GithubTreeType,
    pub sha: String,
    pub url: String,
}
