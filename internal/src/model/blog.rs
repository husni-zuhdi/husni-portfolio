use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow};
use std::fmt::Display;

/// BlogId
/// Identifier of Blog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogId(pub String);

impl BlogId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for BlogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// BlogName
/// Name of the Blog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogName(pub String);

impl BlogName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for BlogName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// BlogFilename
/// Filename of the Blog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogFilename(pub String);

impl BlogFilename {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for BlogFilename {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// BlogBody
/// HTML body of the Blog
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogBody(pub String);

impl BlogBody {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for BlogBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// BlogDeleted
/// Blog Deleted or not
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogDeleted(pub bool);

/// BlogType
/// Type of Blog source
/// Can be:
/// - FileSystem: Blog markdown come from filesystem
/// - Github: Blog markdown come from github repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlogSource {
    FileSystem,
    Github,
}

impl Display for BlogSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::FileSystem => {
                write!(f, "FileSystem")
            }
            Self::Github => {
                write!(f, "Github")
            }
        }
    }
}

/// Blog
/// Blog data with fields:
/// - id: Blog Identifier
/// - name: Blog name
/// - source: Blog source
/// - filename: Blog Filename or Source
/// - body: Blog HTML body
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Blog {
    pub id: BlogId,
    pub name: BlogName,
    pub source: BlogSource,
    pub filename: BlogFilename,
    pub body: BlogBody,
}

/// I don't knwo how it work
/// Ref: https://stackoverflow.com/questions/78615649/how-do-i-load-sqlx-rows-to-a-struct-with-a-vector-of-structs
impl<'r> FromRow<'r, SqliteRow> for Blog {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        let id = row.try_get("id")?;
        let name = row.try_get("name")?;
        let source = match row.try_get("source")? {
            "github" => BlogSource::Github,
            "filesystem" => BlogSource::FileSystem,
            &_ => {
                // Default to FileSystem
                BlogSource::FileSystem
            }
        };
        let filename = row.try_get("filename")?;
        let body = row.try_get("body")?;
        Ok(Blog {
            id: BlogId(id),
            name: BlogName(name),
            source,
            filename: BlogFilename(filename),
            body: BlogBody(body),
        })
    }
}

/// BlogStartPage
/// Start page of Blog Pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogStartPage(pub i32);

/// BlogEndPage
/// End page of Blog Pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogEndPage(pub i32);

/// BlogPagination
/// Axum Query struct for Blog Pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogPagination {
    pub start: Option<BlogStartPage>,
    pub end: Option<BlogEndPage>,
}
