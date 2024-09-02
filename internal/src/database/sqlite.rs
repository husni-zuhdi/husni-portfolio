use crate::api::github::get_gh_blogs;
use crate::model::blog::{
    Blog, BlogBody, BlogDeleted, BlogEndPage, BlogFilename, BlogId, BlogName, BlogSource,
    BlogStartPage,
};
use crate::repo::blog::BlogRepo;
use async_trait::async_trait;
use log::{debug, error, info};
use sqlx::sqlite::SqlitePool;
use sqlx::{query, query_as, Row};

#[derive(Clone)]
pub struct SqliteBlogRepo {
    pub pool: SqlitePool,
}

#[async_trait]
impl BlogRepo for SqliteBlogRepo {
    async fn find(&mut self, id: BlogId) -> Blog {
        let blog_id = id.0;
        let prep_query = "SELECT * FROM blogs WHERE id = $1 ORDER BY id";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let row: Blog = query_as(&prep_query)
            .bind(&blog_id)
            .fetch_one(&self.pool)
            .await
            .expect("Failed to execute get query");
        info!("Blog {} processed.", &row.id);
        debug!("Blog HTML {}.", &row.body);
        row
    }
    async fn find_blogs(&mut self, start: BlogStartPage, end: BlogEndPage) -> Vec<Blog> {
        let start_seq = start.0;
        let end_seq = end.0;
        let limit = end_seq - start_seq;
        let prep_query = "SELECT * FROM blogs ORDER BY id LIMIT $1 OFFSET $2";
        debug!(
            "Executing query {} for start {}, end {}, limit {}",
            &prep_query, &start_seq, &end_seq, &limit
        );

        let rows: Vec<Blog> = query_as(&prep_query)
            .bind(&limit)
            .bind(&start_seq)
            .fetch_all(&self.pool)
            .await
            .expect("Failed to execute get query");
        info!("Blogs from {} to {} processed.", &start_seq, &end_seq);
        for row in rows {
            info!("Blog {} processed.", &row.id);
            debug!("Blog HTML {}.", &row.body);
        }
        rows
    }
    async fn add(
        &mut self,
        id: BlogId,
        name: BlogName,
        filename: BlogFilename,
        source: BlogSource,
        body: BlogBody,
    ) -> Blog {
        let blog_id = id.0;
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
            .await
            .expect("Failed to execute add query");
        info!("Blog {} was added.", &blog_id);

        let prep_get_query = "SELECT * FROM blogs WHERE id = $1 ORDER BY id";
        debug!("Executing query {} for id {}", &prep_get_query, &blog_id);

        let row: Blog = query_as(&prep_get_query)
            .bind(&blog_id)
            .fetch_one(&self.pool)
            .await
            .expect("Failed to execute get query");
        info!("Blog {} processed.", &row.id);
        debug!("Blog HTML {}.", &row.body);
        row
    }
    async fn delete(&mut self, id: BlogId) -> BlogDeleted {
        let blog_id = id.0;
        let prep_query = "DELETE FROM blogs WHERE id = $1";
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let query_res = query(&prep_query)
            .bind(&blog_id)
            .execute(&self.pool)
            .await
            .expect("Failed to execute delete query");
        info!(
            "Blog {} in row {} was deleted.",
            &blog_id,
            &query_res.rows_affected()
        );
        BlogDeleted(true)
    }
    async fn update(
        &mut self,
        id: BlogId,
        name: Option<BlogName>,
        filename: Option<BlogFilename>,
        source: Option<BlogSource>,
        body: Option<BlogBody>,
    ) -> Blog {
        let blog_id = id.0;
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
        let prep_update_query = format!("UPDATE blogs SET{}WHERE id = $1", &affected_col).as_str();
        debug!("Executing query {} for id {}", &prep_update_query, &blog_id);

        let query_res = query(&prep_update_query)
            .bind(&blog_id)
            .execute(&self.pool)
            .await
            .expect("Failed to execute update query");
        info!("Blog {} was updated.", &blog_id);

        let prep_get_query = "SELECT * FROM blogs WHERE id = $1 ORDER BY id";
        debug!("Executing query {} for id {}", &prep_get_query, &blog_id);

        let row: Blog = query_as(&prep_get_query)
            .bind(&blog_id)
            .fetch_one(&self.pool)
            .await
            .expect("Failed to execute get query");
        info!("Blog {} processed.", &row.id);
        debug!("Blog HTML {}.", &row.body);
        row
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
