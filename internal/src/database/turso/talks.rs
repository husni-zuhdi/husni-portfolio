use crate::database::turso::TursoDatabase;
use crate::model::talks::*;
use crate::repo::talks::TalkRepo;
use async_trait::async_trait;
use tracing::{debug, info};

#[async_trait]
impl TalkRepo for TursoDatabase {
    async fn find(&self, id: TalkId) -> Option<Talk> {
        let talk_id = id.id;
        let prep_query = "SELECT * FROM talks WHERE id = ?1 ORDER BY id";
        debug!("Executing query {} for id {}", &prep_query, &talk_id);

        let mut stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find query.");

        let row = stmt
            .query([talk_id])
            .await
            .expect("Failed to query talk.")
            .next()
            .await
            .expect("Failed to access query talk.")
            .expect("Failed to access row talk");

        debug!("Debug Row {:?}", &row);

        // We ditch Turso deserialize since it cannot submit id and source
        // id and source are Tuple Struct
        // I think libsql deserialize is not robust enough yet
        Some(Talk {
            id: TalkId {
                id: row.get(0).unwrap(),
            },
            name: row.get(1).unwrap(),
            // TODO: it's a dummy values. Need to be updated later
            media_link: None,
            org_link: None,
        })
    }
    async fn find_talks(&self, start: TalkStartPage, end: TalkEndPage) -> Option<Vec<Talk>> {
        let start_seq = start.0;
        let end_seq = end.0;
        let limit = end_seq - start_seq;
        let prep_query = "SELECT * FROM talks ORDER BY id LIMIT ?1 OFFSET ?2";
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
            .expect("Failed to query talks.");

        let mut talks: Vec<Talk> = Vec::new();

        while let Some(row) = rows.next().await.unwrap() {
            info!("Debug Row {:?}", &row);

            // We ditch Turso deserialize since it cannot submit id and source
            // id and source are Tuple Struct
            // I think libsql deserialize is not robust enough yet
            talks.push(Talk {
                id: TalkId {
                    id: row.get(0).unwrap(),
                },
                name: row.get(1).unwrap(),
                // TODO: it's a dummy values. Need to be updated later
                media_link: None,
                org_link: None,
            });
        }

        Some(talks)
    }
    async fn add(
        &mut self,
        id: TalkId,
        name: String,
        media_link: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus> {
        let talk_id = &id.id;
        let talk_name = &name;
        let talk_media_link = if let Some(val) = media_link {
            debug!("Media Link is present for talk id {}", &talk_id);
            val
        } else {
            "".to_string()
        };
        let talk_org_link = if let Some(val) = org_link {
            debug!("Organization Link is present for talk id {}", &talk_id);
            val
        } else {
            "".to_string()
        };

        let prep_add_query =
            "INSERT INTO talks (id, name, media_link, org_link) VALUES (?1, ?2, ?3, ?4)";
        debug!("Executing query {} for id {}", &prep_add_query, &talk_id);

        let mut stmt = self
            .conn
            .prepare(prep_add_query)
            .await
            .expect("Failed to prepare add query.");

        // TRIVIA: With libsql 0.6.0 don't use execute other than execute(())
        // Somehow it broke the complier and mess up the variables type
        let exe = stmt
            .execute((
                talk_id.clone(),
                talk_name.clone(),
                talk_media_link.clone(),
                talk_org_link.clone(),
            ))
            .await
            .expect("Failed to add talk.");
        info!("Add Execution returned: {}", exe);

        Some(TalkCommandStatus::Stored)
    }
    async fn delete(&mut self, id: TalkId) -> Option<TalkCommandStatus> {
        let talk_id = id.id;
        let prep_query = "DELETE FROM talks WHERE id = ?1";
        debug!("Executing query {} for id {}", &prep_query, &talk_id);

        let mut stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare delete command.");

        match stmt.execute([talk_id.clone()]).await {
            Ok(val) => {
                debug!(
                    "Talk {} was deleted. Execution returned : {}",
                    &talk_id, val
                );
                Some(TalkCommandStatus::Deleted)
            }
            Err(err) => {
                debug!("Talk {} is not deleted in Turso. Error {}", &talk_id, err);
                None
            }
        }
    }
    async fn update(
        &mut self,
        id: TalkId,
        name: Option<String>,
        media_link: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus> {
        let talk_id = &id.id;
        let mut affected_col = "".to_string();
        match &name {
            Some(val) => {
                affected_col = format!("{} name = {} ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update name field")
            }
        }
        match &media_link {
            Some(val) => {
                affected_col = format!("{} media_link = {} ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update media_link field")
            }
        }
        match &org_link {
            Some(val) => {
                affected_col = format!("{} org_link = {} ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update org_link field")
            }
        }
        // Trimming the last ','
        affected_col = affected_col.as_str()[0..affected_col.len() - 1].to_string();

        let prep_update_query = format!("UPDATE talks SET{}WHERE id = ?1", &affected_col);
        debug!("Executing query {} for id {}", &prep_update_query, &talk_id);

        let mut stmt = self
            .conn
            .prepare(&prep_update_query)
            .await
            .expect("Failed to prepare update query.");

        let exe = stmt
            .execute([talk_id.clone()])
            .await
            .expect("Failed to update talk.");
        info!("Update Execution returned: {}", exe);

        Some(TalkCommandStatus::Updated)
    }
}
