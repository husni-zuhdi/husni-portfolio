use crate::database::turso::TursoDatabase;
use crate::model::blog_tag_mappings::*;
use crate::repo::blog_tag_mappings::BlogTagMappingRepo;
use async_trait::async_trait;
use tracing::debug;

#[async_trait]
impl BlogTagMappingRepo for TursoDatabase {
    async fn find_by_blog_id(&self, blog_id: i64) -> Option<BlogTagMappings> {
        let prep_query = r#"
            SELECT
                blog_ref,
                tag_ref
            FROM blog_tag_mapping
            WHERE blog_ref = ?1
        "#;
        debug!("Executing query {} for id {}", &prep_query, &blog_id);

        let mut stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find blog tag mappings by blog_id query.");

        let mut rows = stmt
            .query([blog_id])
            .await
            .expect("Failed to query blog tag mappings by blog_id.");

        let mut maps: Vec<BlogTagMapping> = Vec::new();

        while let Some(row) = rows.next().await.unwrap() {
            debug!("Debug Row {:?}", &row);
            maps.push(BlogTagMapping {
                blog_id: row.get(0).unwrap(),
                tag_id: row.get(1).unwrap(),
            });
        }

        Some(BlogTagMappings { maps })
    }
    async fn find_by_tag_id(&self, tag_id: i64) -> Option<BlogTagMappings> {
        let prep_query = r#"
            SELECT
                blog_ref,
                tag_ref
            FROM blog_tag_mapping
            WHERE tag_ref = ?1
        "#;
        debug!("Executing query {} for id {}", &prep_query, &tag_id);

        let mut stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find blog tag mappings by blog_id query.");

        let mut rows = stmt
            .query([tag_id])
            .await
            .expect("Failed to query blog tag mappings by blog_id.");

        let mut maps: Vec<BlogTagMapping> = Vec::new();

        while let Some(row) = rows.next().await.unwrap() {
            debug!("Debug Row {:?}", &row);
            maps.push(BlogTagMapping {
                blog_id: row.get(0).unwrap(),
                tag_id: row.get(1).unwrap(),
            });
        }

        Some(BlogTagMappings { maps })
    }
    async fn add(&mut self, blog_id: i64, tag_id: i64) -> Option<BlogTagMappingCommandStatus> {
        let prep_add_command = "INSERT INTO blog_tag_mapping (blog_ref, tag_ref) VALUES (?1, ?2)";
        debug!(
            "Executing query {} for blog_id {} and tag_id {}",
            &prep_add_command, &blog_id, &tag_id
        );

        let mut stmt = self
            .conn
            .prepare(prep_add_command)
            .await
            .expect("Failed to prepare add commmand.");

        let exe = stmt
            .execute((blog_id, tag_id))
            .await
            .expect("Failed to add blog tag mapping.");
        debug!("Add Execution returned: {}", exe);

        Some(BlogTagMappingCommandStatus::Stored)
    }
    async fn delete_by_blog_id(&mut self, blog_id: i64) -> Option<BlogTagMappingCommandStatus> {
        let prep_delete_command = "DELETE FROM blog_tag_mapping WHERE blog_ref = ?1";
        debug!(
            "Executing query {} for id {}",
            &prep_delete_command, &blog_id
        );

        let mut stmt = self
            .conn
            .prepare(prep_delete_command)
            .await
            .expect("Failed to prepare delete command.");

        let exe = stmt
            .execute([blog_id])
            .await
            .expect("Failed to delete a Blog Tag Mapping.");

        debug!("Delete Execution returned: {}", exe);
        Some(BlogTagMappingCommandStatus::Deleted)
    }
    async fn delete_by_blog_id_and_tag_id(
        &mut self,
        blog_id: i64,
        tag_id: i64,
    ) -> Option<BlogTagMappingCommandStatus> {
        let prep_delete_command =
            "DELETE FROM blog_tag_mapping WHERE blog_ref = ?1 AND tag_ref = ?2";
        debug!(
            "Executing query {} for blog id {} and tag id {}",
            &prep_delete_command, &blog_id, &tag_id
        );

        let mut stmt = self
            .conn
            .prepare(prep_delete_command)
            .await
            .expect("Failed to prepare delete command.");

        let exe = stmt
            .execute([blog_id, tag_id])
            .await
            .expect("Failed to delete a Blog Tag Mapping.");

        debug!("Delete Execution returned: {}", exe);
        Some(BlogTagMappingCommandStatus::Deleted)
    }
}
