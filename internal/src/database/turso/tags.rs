use crate::database::turso::TursoDatabase;
use crate::model::tags::*;
use crate::repo::tags::TagRepo;
use async_trait::async_trait;
use tracing::debug;

#[async_trait]
impl TagRepo for TursoDatabase {
    async fn find(&self, id: i64) -> Option<Tag> {
        let prep_query = r#"
            SELECT
                id,
                name
            FROM tags
            WHERE id=?1
            LIMIT 1
        "#;
        debug!("Executing query {} for id {}", &prep_query, &id);

        let mut stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find query.");

        let res = stmt
            .query([id])
            .await
            .expect("Failed to query tag.")
            .next()
            .await
            .expect("Failed to access query tag.");

        let Some(row) = res else {
            debug!("No Tag with Id {} is available.", &id);
            return None;
        };

        debug!("Debug Row {:?}", &row);
        Some(Tag {
            id: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
        })
    }
    async fn find_all(&self) -> Option<Tags> {
        let prep_query = r#"
            SELECT
                id,
                name
            FROM tags
        "#;
        debug!("Executing query {}", &prep_query);

        let mut stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find query.");

        let mut rows = stmt.query(()).await.expect("Failed to query tags.");

        let mut tags: Vec<Tag> = Vec::new();

        while let Some(row) = rows.next().await.unwrap() {
            debug!("Debug Row {:?}", &row);
            tags.push(Tag {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
            });
        }

        Some(Tags { tags })
    }
    async fn get_new_id(&self) -> Option<i64> {
        let prep_query = "SELECT COUNT(id) AS length FROM tags";
        debug!("Executing lenght query {}", &prep_query);

        let row = self
            .conn
            .query(prep_query, ())
            .await
            .expect("Failed to query Tags length.")
            .next()
            .await
            .expect("Failed to access Tags length.")
            .expect("Failed to access Tags length row.");

        debug!("Debug Row {:?}", &row);

        let lenght_id: Option<i64> = row.get(0).unwrap();
        let new_id = lenght_id.unwrap() + 1;

        Some(new_id)
    }
    async fn add(&mut self, id: i64, name: String) -> Option<TagCommandStatus> {
        let prep_add_command = "INSERT INTO tags (id, name) VALUES (?1, ?2)";
        debug!("Executing query {} for id {}", &prep_add_command, &id);

        let mut stmt = self
            .conn
            .prepare(prep_add_command)
            .await
            .expect("Failed to prepare add commmand.");

        let exe = stmt
            .execute((id, name.clone()))
            .await
            .expect("Failed to add blog.");
        debug!("Add Execution returned: {}", exe);

        Some(TagCommandStatus::Stored)
    }
    async fn delete(&mut self, id: i64) -> Option<TagCommandStatus> {
        let prep_delete_command = "DELETE FROM tags WHERE id = ?1";
        debug!("Executing query {} for id {}", &prep_delete_command, &id);

        let mut stmt = self
            .conn
            .prepare(prep_delete_command)
            .await
            .expect("Failed to prepare delete command.");

        let exe = stmt.execute([id]).await.expect("Failed to delete a Tag.");

        debug!("Delete Execution returned: {}", exe);
        Some(TagCommandStatus::Deleted)
    }
    async fn update(&mut self, id: i64, name: Option<String>) -> Option<TagCommandStatus> {
        let mut affected_col = "".to_string();
        match &name {
            Some(val) => {
                affected_col = format!("{} name = '{}' ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update name field.")
            }
        }

        // Trimming the last ','
        affected_col = affected_col.as_str()[0..affected_col.len() - 1].to_string();
        let prep_update_command = format!("UPDATE tags SET{}WHERE id = ?1", &affected_col);
        debug!("Executing query {} for id {}", &prep_update_command, &id);

        let mut stmt = self
            .conn
            .prepare(&prep_update_command)
            .await
            .expect("Failed to prepare update tag command.");

        let exe = stmt.execute([id]).await.expect("Failed to update tag.");
        debug!("Update Execution returned: {}", exe);

        Some(TagCommandStatus::Updated)
    }
}
