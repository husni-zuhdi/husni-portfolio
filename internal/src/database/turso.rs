use crate::model::blogs::*;
use crate::repo::blogs::BlogRepo;
use async_trait::async_trait;
use libsql::{de, params, Builder, Connection};
use tracing::{debug, error, info, warn};

#[derive(Clone)]
pub struct TursoBlogRepo {
    pub blogs: Connection,
}

#[async_trait]
impl BlogRepo for TursoBlogRepo {
    async fn find(&self, id: BlogId) -> Option<Blog> {
        let blog_id = id.0;
        let prep_query = "SELECT * FROM blogs WHERE id = ?1 ORDER BY id";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let mut stmt = self
            .blogs
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find query.");

        let row = stmt
            .query([blog_id])
            .await
            .expect("Failed to query blog.")
            .next()
            .await
            .expect("Failed to access query blog.")
            .expect("Failed to access row blog");

        let blog =
            de::from_row::<Blog>(&row).expect("Failed to deserialize blog row to Blog struct.");

        Some(blog)
    }
    async fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage) -> Option<Vec<Blog>> {
        let start_seq = start.0;
        let end_seq = end.0;
        let limit = end_seq - start_seq;
        let prep_query = "SELECT * FROM blogs ORDER BY id LIMIT ?1 OFFSET ?2";
        debug!(
            "Executing query {} for start {}, end {}, limit {}",
            &prep_query, &start_seq, &end_seq, &limit
        );

        let mut stmt = self
            .blogs
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find query.");

        let row = stmt
            .query([start_seq, limit])
            .await
            .expect("Failed to query blogs.")
            .next()
            .await
            .expect("Failed to access query blogs.")
            .expect("Failed to access row blogs");

        let blogs = de::from_row::<Vec<Blog>>(&row)
            .expect("Failed to deserialize blog rows to Vector of Blog structs.");

        Some(blogs)
    }
    async fn check_id(&self, id: BlogId) -> Option<BlogStored> {
        let blog_id = id.0;
        let prep_query = "SELECT id FROM blogs WHERE id = ?1 ORDER BY id";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let mut stmt = self
            .blogs
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find query.");

        let row = stmt
            .query([blog_id.clone()])
            .await
            .expect("Failed to query blog id.")
            .next()
            .await
            .expect("Failed to access query blog id.")
            .expect("Failed to access row blog id.");

        match de::from_row::<String>(&row) {
            Ok(id) => {
                info!("Blog {} is in SQLite.", &id);
                Some(BlogStored(true))
            }
            Err(err) => {
                info!("Blog {} is not in SQLite. Error: {}", &blog_id, err);
                Some(BlogStored(false))
            }
        }
    }
    async fn add(
        &mut self,
        id: BlogId,
        name: BlogName,
        filename: BlogFilename,
        source: BlogSource,
        body: BlogBody,
    ) -> Option<Blog> {
        let blog_id = &id.0;
        let blog_name = name.0;
        let blog_filename = filename.0;
        let blog_source = format!("{}", source);
        let blog_body = body.0;
        let prep_add_query =
            "INSERT INTO blogs (id, name, filename, source, body) VALUES (?1, ?2, ?3, ?4, ?5)";
        debug!("Executing query {} for id {}", &prep_add_query, &blog_id);

        let mut stmt = self
            .blogs
            .prepare(prep_add_query)
            .await
            .expect("Failed to prepare add query.");

        let row = stmt
            .query([
                blog_id.clone(),
                blog_name.clone(),
                blog_filename.clone(),
                blog_source.clone(),
                blog_body.clone(),
            ])
            .await
            .expect("Failed to add blog.")
            .next()
            .await
            .expect("Failed to access add blog.")
            .expect("Failed to access row blog.");

        let blog =
            de::from_row::<Blog>(&row).expect("Failed to deserialize blog row to Blog struct.");

        Some(blog)
    }
    async fn delete(&mut self, id: BlogId) -> Option<BlogDeleted> {
        let blog_id = id.0;
        let prep_query = "DELETE FROM blogs WHERE id = ?1";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let mut stmt = self
            .blogs
            .prepare(prep_query)
            .await
            .expect("Failed to prepare delete command.");

        match stmt.execute([blog_id.clone()]).await {
            Ok(val) => {
                debug!("Blog {} was deleted. Unknown usize: {}", &blog_id, val);
                Some(BlogDeleted(true))
            }
            Err(err) => {
                debug!("Blog {} is not deleted in Turso. Error {}", &blog_id, err);
                Some(BlogDeleted(false))
            }
        }
    }
    async fn update(
        &mut self,
        id: BlogId,
        name: Option<BlogName>,
        filename: Option<BlogFilename>,
        source: Option<BlogSource>,
        body: Option<BlogBody>,
    ) -> Option<Blog> {
        let blog_id = &id.0;
        let mut affected_col = "".to_string();
        match name {
            Some(val) => {
                affected_col = format!("{} name = {} ", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update name field")
            }
        }
        match filename {
            Some(val) => {
                affected_col = format!("{} filename = {} ", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update name field")
            }
        }
        match source {
            Some(val) => {
                affected_col = format!("{} source = {} ", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update name field")
            }
        }
        match body {
            Some(val) => {
                affected_col = format!("{} body = {} ", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update name field")
            }
        }
        let prep_update_query = format!("UPDATE blogs SET{}WHERE id = ?1", &affected_col);
        debug!("Executing query {} for id {}", &prep_update_query, &blog_id);

        let mut stmt = self
            .blogs
            .prepare(&prep_update_query)
            .await
            .expect("Failed to prepare update query.");

        let row = stmt
            .query([blog_id.clone()])
            .await
            .expect("Failed to update blog.")
            .next()
            .await
            .expect("Failed to access update blog.")
            .expect("Failed to access row blog.");

        let blog =
            de::from_row::<Blog>(&row).expect("Failed to deserialize blog row to Blog struct.");

        Some(blog)
    }
}

impl TursoBlogRepo {
    pub async fn new(database_url: String, database_token: String) -> TursoBlogRepo {
        let db = Builder::new_remote(database_url, database_token)
            .build()
            .await
            .expect("Failed to build turso database.");

        let conn = db
            .connect()
            .expect("Failed to setup connection to turso database.");

        // Check blogs table is created or not
        let migration_command = r#"
        CREATE TABLE IF NOT EXISTS blogs (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            source TEXT NOT NULL,
            filename TEXT NOT NULL,
            body TEXT NOT NULL
        )"#;
        let _migration = conn
            .execute(migration_command, ())
            .await
            .expect("Failed to migrate blogs table.");

        TursoBlogRepo { blogs: conn }
    }
}
