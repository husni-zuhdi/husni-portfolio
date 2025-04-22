use crate::database::turso::TursoDatabase;
use crate::model::blogs::*;
use crate::repo::blogs::BlogRepo;
use async_trait::async_trait;
use libsql::de;
use tracing::{debug, error, info};

#[async_trait]
impl BlogRepo for TursoDatabase {
    async fn find(&self, id: BlogId) -> Option<Blog> {
        let blog_id = id.id;
        let prep_query = "SELECT * FROM blogs WHERE id = ?1 ORDER BY id";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let mut stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find query.");

        let res = stmt
            .query([blog_id])
            .await
            .expect("Failed to query blog.")
            .next()
            .await
            .expect("Failed to access query blog.");

        match res {
            Some(row) => {
                debug!("Debug Row {:?}", &row);
                let source_string = row.get::<String>(2).unwrap();
                let source = match source_string.as_str() {
                    "Filesystem" => BlogSource::Filesystem,
                    "Github" => BlogSource::Github,
                    _ => {
                        error!("Failed to parse blog source. Default to Filesystem");
                        BlogSource::Filesystem
                    }
                };

                let tags: Vec<String> = row
                    .get::<String>(5)
                    .unwrap_or("".to_string())
                    .split(",")
                    .map(|tag| tag.to_string())
                    .collect();

                // We ditch Turso deserialize since it cannot submit id and source
                // id and source are Tuple Struct
                // I think libsql deserialize is not robust enough yet
                Some(Blog {
                    id: BlogId {
                        id: row.get(0).unwrap(),
                    },
                    name: Some(row.get(1).unwrap()),
                    source: Some(source),
                    filename: Some(row.get(3).unwrap()),
                    body: Some(row.get(4).unwrap()),
                    tags: Some(tags),
                })
            }
            None => {
                debug!("No Blog with Id {} is available", &blog_id);
                None
            }
        }
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
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find query.");

        let mut rows = stmt
            .query([limit, start_seq])
            .await
            .expect("Failed to query blogs.");

        let mut blogs: Vec<BlogMetadata> = Vec::new();

        while let Some(row) = rows.next().await.unwrap() {
            debug!("Debug Row {:?}", &row);

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
    async fn check_id(&self, id: BlogId) -> Option<BlogCommandStatus> {
        let blog_id = id.id;
        let prep_query = "SELECT id FROM blogs WHERE id = ?1 ORDER BY id";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let mut stmt = self
            .conn
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
                Some(BlogCommandStatus::Stored)
            }
            None => {
                info!("Blog {} is not in Turso/SQLite.", &blog_id);
                None
            }
        }
    }
    async fn add(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        let blog_id = &blog.id.id;
        let blog_name = &blog.name.unwrap();
        let blog_filename = &blog.filename.unwrap();
        let blog_source = format!("{}", blog.source.unwrap());
        let blog_body = &blog.body.unwrap();
        let blog_tags: &String = &blog.tags.unwrap().join(",");
        let prep_add_query =
            "INSERT INTO blogs (id, name, filename, source, body, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
        debug!("Executing query {} for id {}", &prep_add_query, &blog_id);

        let mut stmt = self
            .conn
            .prepare(prep_add_query)
            .await
            .expect("Failed to prepare add query.");

        let exe = stmt
            .execute((
                blog_id.clone(),
                blog_name.clone(),
                blog_filename.clone(),
                blog_source.clone(),
                blog_body.clone(),
                blog_tags.clone(),
            ))
            .await
            .expect("Failed to add blog.");
        debug!("Add Execution returned: {}", exe);

        Some(BlogCommandStatus::Stored)
    }
    async fn delete(&mut self, id: BlogId) -> Option<BlogCommandStatus> {
        let blog_id = id.id;
        let prep_query = "DELETE FROM blogs WHERE id = ?1";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let mut stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare delete command.");

        match stmt.execute([blog_id.clone()]).await {
            Ok(val) => {
                debug!(
                    "Blog {} was deleted. Execution returned : {}",
                    &blog_id, val
                );
                Some(BlogCommandStatus::Deleted)
            }
            Err(err) => {
                debug!("Blog {} is not deleted in Turso. Error {}", &blog_id, err);
                None
            }
        }
    }
    async fn update(&mut self, blog: Blog) -> Option<BlogCommandStatus> {
        let blog_id = &blog.id.id;
        let mut affected_col = "".to_string();
        match &blog.name {
            Some(val) => {
                affected_col = format!("{} name = {} ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update name field")
            }
        }
        match &blog.filename {
            Some(val) => {
                affected_col = format!("{} filename = {} ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update filename field")
            }
        }
        match &blog.source {
            Some(val) => {
                affected_col = format!("{} source = {} ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update source field")
            }
        }
        match &blog.body {
            Some(val) => {
                affected_col = format!("{} body = {} ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update body field")
            }
        }
        match &blog.tags {
            Some(val) => {
                let updated_tags = val.join(",");
                affected_col = format!("{} tags = {} ,", &affected_col, updated_tags);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update tags field")
            }
        }

        // Trimming the last ','
        affected_col = affected_col.as_str()[0..affected_col.len() - 1].to_string();
        let prep_update_query = format!("UPDATE blogs SET{}WHERE id = ?1", &affected_col);
        debug!("Executing query {} for id {}", &prep_update_query, &blog_id);

        let mut stmt = self
            .conn
            .prepare(&prep_update_query)
            .await
            .expect("Failed to prepare update query.");

        let exe = stmt
            .execute([blog_id.clone()])
            .await
            .expect("Failed to update blog.");
        debug!("Update Execution returned: {}", exe);

        Some(BlogCommandStatus::Updated)
    }
}
