use crate::model::blogs::*;
use crate::repo::blogs::BlogRepo;
use async_trait::async_trait;
use libsql::{de, Builder, Connection};
use tracing::{debug, error, info, warn};

#[derive(Clone)]
pub struct TursoBlogRepo {
    pub blogs: Connection,
}

#[async_trait]
impl BlogRepo for TursoBlogRepo {
    async fn find(&self, id: BlogId) -> Option<Blog> {
        let blog_id = id.id;
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

        info!("Debug Row {:?}", &row);
        let source_string = row.get::<String>(2).unwrap();
        let source = match source_string.as_str() {
            "Filesystem" => BlogSource::Filesystem,
            "Github" => BlogSource::Github,
            _ => {
                error!("Failed to parse blog source. Default to Filesystem");
                BlogSource::Filesystem
            }
        };

        // We ditch Turso deserialize since it cannot submit id and source
        // id and source are Tuple Struct
        // I think libsql deserialize is not robust enough yet
        Some(Blog {
            id: BlogId {
                id: row.get(0).unwrap(),
            },
            name: row.get(1).unwrap(),
            source,
            filename: row.get(3).unwrap(),
            body: row.get(4).unwrap(),
        })
    }
    async fn find_blogs(
        &self,
        start: BlogStartPage,
        end: BlogEndPage,
    ) -> Option<Vec<BlogMetadata>> {
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

        let mut rows = stmt
            .query([limit, start_seq])
            .await
            .expect("Failed to query blogs.");

        let mut blogs: Vec<BlogMetadata> = Vec::new();

        while let Some(row) = rows.next().await.unwrap() {
            info!("Debug Row {:?}", &row);

            // We ditch Turso deserialize since it cannot submit id and source
            // id and source are Tuple Struct
            // I think libsql deserialize is not robust enough yet
            blogs.push(BlogMetadata {
                id: BlogId {
                    id: row.get(0).unwrap(),
                },
                name: row.get(1).unwrap(),
                filename: row.get(3).unwrap(),
            });
        }

        Some(blogs)
    }
    async fn check_id(&self, id: BlogId) -> Option<BlogStored> {
        let blog_id = id.id;
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
            .expect("Failed to access query blog id.");

        match row {
            Some(val) => {
                let id: BlogId = de::from_row(&val).unwrap();
                info!("Blog {:?} is in Turso/SQLite.", &id);
                Some(BlogStored(true))
            }
            None => {
                info!("Blog {} is not in Turso/SQLite.", &blog_id);
                Some(BlogStored(false))
            }
        }
    }
    async fn add(
        &mut self,
        id: BlogId,
        name: String,
        filename: String,
        source: BlogSource,
        body: String,
    ) -> Option<Blog> {
        let blog_id = &id.id;
        let blog_name = &name;
        let blog_filename = &filename;
        let blog_source = format!("{}", source);
        let blog_body = &body;
        let prep_add_query =
            "INSERT INTO blogs (id, name, filename, source, body) VALUES (?1, ?2, ?3, ?4, ?5)";
        debug!("Executing query {} for id {}", &prep_add_query, &blog_id);

        let mut stmt = self
            .blogs
            .prepare(prep_add_query)
            .await
            .expect("Failed to prepare add query.");

        let exe = stmt
            .execute([
                blog_id.clone(),
                blog_name.clone(),
                blog_filename.clone(),
                blog_source.clone(),
                blog_body.clone(),
            ])
            .await
            .expect("Failed to add blog.");
        info!("Add Execution returned: {}", exe);

        Some(Blog {
            id,
            name,
            filename,
            source,
            body,
        })
    }
    async fn delete(&mut self, id: BlogId) -> Option<BlogDeleted> {
        let blog_id = id.id;
        let prep_query = "DELETE FROM blogs WHERE id = ?1";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let mut stmt = self
            .blogs
            .prepare(prep_query)
            .await
            .expect("Failed to prepare delete command.");

        match stmt.execute([blog_id.clone()]).await {
            Ok(val) => {
                debug!(
                    "Blog {} was deleted. Execution returned : {}",
                    &blog_id, val
                );
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
        name: Option<String>,
        filename: Option<String>,
        source: Option<BlogSource>,
        body: Option<String>,
    ) -> Option<Blog> {
        let blog_id = &id.id;
        let mut affected_col = "".to_string();
        match &name {
            Some(val) => {
                affected_col = format!("{} name = {} ", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update name field")
            }
        }
        match &filename {
            Some(val) => {
                affected_col = format!("{} filename = {} ", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update name field")
            }
        }
        match &source {
            Some(val) => {
                affected_col = format!("{} source = {} ", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update name field")
            }
        }
        match &body {
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

        let exe = stmt
            .execute([blog_id.clone()])
            .await
            .expect("Failed to update blog.");
        info!("Update Execution returned: {}", exe);

        // TODO make sure the data is presented in here is correct and consistent
        Some(Blog {
            id,
            name: name.unwrap(),
            filename: filename.unwrap(),
            source: source.unwrap(),
            body: body.unwrap(),
        })
    }
}

impl TursoBlogRepo {
    pub async fn new(
        mode: String,
        database_url: String,
        database_token: Option<String>,
    ) -> TursoBlogRepo {
        let db: libsql::Database = match mode.as_str() {
            "sqlite" => Builder::new_local(database_url)
                .build()
                .await
                .expect("Failed to build SQLITE database."),
            "turso" => Builder::new_remote(database_url, database_token.unwrap())
                .build()
                .await
                .expect("Failed to build turso database."),
            &_ => {
                warn!("Turso Database mode not set. Default to 'sqlite'");
                Builder::new_remote(database_url, database_token.unwrap())
                    .build()
                    .await
                    .expect("Failed to build turso database.")
            }
        };

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
