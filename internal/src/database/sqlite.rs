use crate::model::blogs::*;
use crate::repo::blogs::BlogRepo;
use async_trait::async_trait;
use sqlx::sqlite::SqlitePool;
use sqlx::{query, query_as, Error, Sqlite};
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct SqliteBlogRepo {
    pub pool: SqlitePool,
}

#[async_trait]
impl BlogRepo for SqliteBlogRepo {
    async fn find(&self, id: BlogId) -> Option<Blog> {
        let blog_id = id.0;
        let prep_query = "SELECT * FROM blogs WHERE id = $1 ORDER BY id";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let row: Result<Blog, Error> = query_as(&prep_query)
            .bind(&blog_id)
            .fetch_one(&self.pool)
            .await;

        match row {
            Ok(blog) => {
                info!("Blog {} processed.", &blog.id);
                debug!("Blog HTML {}.", &blog.body);
                Some(blog)
            }
            Err(err) => {
                error!(
                    "Failed to get Blog with Id {}. Blog Not Found. Err: {}",
                    &blog_id, err
                );
                None
            }
        }
    }
    async fn find_blogs(&self, start: BlogStartPage, end: BlogEndPage) -> Option<Vec<Blog>> {
        let start_seq = start.0;
        let end_seq = end.0;
        let limit = end_seq - start_seq;
        let prep_query = "SELECT * FROM blogs ORDER BY id LIMIT $1 OFFSET $2";
        debug!(
            "Executing query {} for start {}, end {}, limit {}",
            &prep_query, &start_seq, &end_seq, &limit
        );

        let rows: Result<Vec<Blog>, Error> = query_as(&prep_query)
            .bind(&limit)
            .bind(&start_seq)
            .fetch_all(&self.pool)
            .await;

        match rows {
            Ok(blogs) => {
                info!("Blogs from {} to {} processed.", &start_seq, &end_seq);
                for row in &blogs {
                    info!("Blog {} processed.", &row.id);
                    debug!("Blog HTML {}.", &row.body);
                }
                Some(blogs)
            }
            Err(err) => {
                error!(
                    "Failed to get Blogs with Id started at {} and ended at {}. Err: {}",
                    &start_seq, &end_seq, err
                );
                None
            }
        }
    }
    async fn check_id(&self, id: BlogId) -> Option<BlogStored> {
        let blog_id = id.0;
        let prep_query = "SELECT id FROM blogs WHERE id = $1 ORDER BY id";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        match query_as::<Sqlite, BlogId>(&prep_query)
            .bind(&blog_id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(id) => {
                info!("Blog {} is in SQLite.", &id.0);
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
            "INSERT INTO blogs (id, name, filename, source, body) VALUES ($1, $2, $3, $4, $5)";
        debug!("Executing query {} for id {}", &prep_add_query, &blog_id);

        let query_res = query(&prep_add_query)
            .bind(&blog_id)
            .bind(&blog_name)
            .bind(&blog_filename)
            .bind(&blog_source)
            .bind(&blog_body)
            .execute(&self.pool)
            .await;

        match query_res {
            Ok(row) => {
                info!(
                    "Blog {} in row {} added to SQLite.",
                    &blog_id,
                    &row.rows_affected()
                );

                Self::find(&self.clone(), id.clone()).await
            }
            Err(err) => {
                error!("Failed to add Blog with Id {}. Err: {}", &blog_id, err);
                None
            }
        }
    }
    async fn delete(&mut self, id: BlogId) -> Option<BlogDeleted> {
        let blog_id = id.0;
        let prep_query = "DELETE FROM blogs WHERE id = $1";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let query_res = query(&prep_query).bind(&blog_id).execute(&self.pool).await;

        match query_res {
            Ok(row) => {
                info!(
                    "Blog {} in row {} was deleted.",
                    &blog_id,
                    &row.rows_affected()
                );
                Some(BlogDeleted(true))
            }
            Err(err) => {
                error!("Failed to delete Blog with Id {}. Err: {}", &blog_id, err);
                None
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
        let prep_update_query = format!("UPDATE blogs SET{}WHERE id = $1", &affected_col);
        debug!("Executing query {} for id {}", &prep_update_query, &blog_id);

        let query_res = query(&prep_update_query)
            .bind(&blog_id)
            .execute(&self.pool)
            .await;

        match query_res {
            Ok(row) => {
                info!(
                    "Blog {} in row {} was updated on SQLite.",
                    &blog_id,
                    &row.rows_affected()
                );

                Self::find(&self.clone(), id.clone()).await
            }
            Err(err) => {
                error!("Failed to update Blog with Id {}. Err: {}", &blog_id, err);
                None
            }
        }
    }
}

impl SqliteBlogRepo {
    pub async fn new(database_url: String) -> SqliteBlogRepo {
        let pool = SqlitePool::connect(database_url.as_str())
            .await
            .expect("Failed to start sqlite pool");
        SqliteBlogRepo { pool }
    }
}
