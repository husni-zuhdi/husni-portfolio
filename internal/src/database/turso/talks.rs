use crate::database::turso::TursoDatabase;
use crate::model::talks::*;
use crate::repo::talks::TalkRepo;
use async_trait::async_trait;
use tracing::{debug, info};

#[async_trait]
impl TalkRepo for TursoDatabase {
    async fn get_new_id(&self) -> Option<TalkId> {
        let prep_query = "SELECT COUNT(id) AS lenght FROM talks";
        debug!("Executing lenght query {}", &prep_query);

        let row = self
            .conn
            .query(prep_query, ())
            .await
            .expect("Failed to query Talks length.")
            .next()
            .await
            .expect("Failed to access Talks length.")
            .expect("Failed to access Talks length row.");

        debug!("Debug Row {:?}", &row);

        let lenght_id: Option<i64> = row.get(0).unwrap();
        let new_id = lenght_id.unwrap() + 1;

        Some(TalkId { id: new_id })
    }
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
            .expect("Failed to access row talk.");

        debug!("Debug Row {:?}", &row);

        let mut media_link: Option<String> = row.get(3).unwrap();
        if media_link.clone().unwrap().is_empty() {
            media_link = None;
        }
        let mut org_name: Option<String> = row.get(4).unwrap();
        if org_name.clone().unwrap().is_empty() {
            org_name = None;
        }
        let mut org_link: Option<String> = row.get(5).unwrap();
        if org_link.clone().unwrap().is_empty() {
            org_link = None;
        }

        Some(Talk {
            id: TalkId {
                id: row.get(0).unwrap(),
            },
            name: row.get(1).unwrap(),
            date: row.get(2).unwrap(),
            media_link,
            org_name,
            org_link,
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
            .expect("Failed to prepare find Talks query.");

        let mut rows = stmt
            .query([limit, start_seq])
            .await
            .expect("Failed to query talks.");

        let mut talks: Vec<Talk> = Vec::new();

        while let Some(row) = rows.next().await.unwrap() {
            debug!("Debug Row {:?}", &row);

            let mut media_link: Option<String> = row.get(3).unwrap();
            if media_link.clone().unwrap().is_empty() {
                media_link = None;
            }
            let mut org_name: Option<String> = row.get(4).unwrap();
            if org_name.clone().unwrap().is_empty() {
                org_name = None;
            }
            let mut org_link: Option<String> = row.get(5).unwrap();
            if org_link.clone().unwrap().is_empty() {
                org_link = None;
            }

            talks.push(Talk {
                id: TalkId {
                    id: row.get(0).unwrap(),
                },
                name: row.get(1).unwrap(),
                date: row.get(2).unwrap(),
                media_link,
                org_name,
                org_link,
            });
        }

        Some(talks)
    }
    async fn add(
        &mut self,
        id: TalkId,
        name: String,
        date: String,
        media_link: Option<String>,
        org_name: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus> {
        let talk_id = &id.id;
        let talk_name = &name;
        let talk_date = &date;
        let talk_media_link = if let Some(val) = media_link {
            debug!("Media Link is present for talk id {}", &talk_id);
            val
        } else {
            "".to_string()
        };
        let talk_org_name = if let Some(val) = org_name {
            debug!("Organization Name is present for talk id {}", &talk_id);
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

        let prep_add_command =
            "INSERT INTO talks (id, name, date, media_link, org_name, org_link) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
        debug!("Executing query {} for id {}", &prep_add_command, &talk_id);

        let mut stmt = self
            .conn
            .prepare(prep_add_command)
            .await
            .expect("Failed to prepare add Talk command.");

        // TRIVIA: With libsql 0.6.0 don't use execute other than execute(())
        // Somehow it broke the complier and mess up the variables type
        let exe = stmt
            .execute((
                *talk_id,
                talk_name.clone(),
                talk_date.clone(),
                talk_media_link.clone(),
                talk_org_name.clone(),
                talk_org_link.clone(),
            ))
            .await
            .expect("Failed to add a Talk.");
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
            .expect("Failed to prepare delete Talk command.");

        let exe = stmt
            .execute([talk_id])
            .await
            .expect("Failed to delete a Talk.");

        debug!("Delete Execution returned: {}", exe);
        Some(TalkCommandStatus::Deleted)
    }
    async fn update(
        &mut self,
        id: TalkId,
        name: Option<String>,
        date: Option<String>,
        media_link: Option<String>,
        org_name: Option<String>,
        org_link: Option<String>,
    ) -> Option<TalkCommandStatus> {
        let talk_id = &id.id;
        let mut affected_col = "".to_string();
        match &name {
            Some(val) => {
                affected_col = format!("{} name = '{}' ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update name field")
            }
        }
        match &date {
            Some(val) => {
                affected_col = format!("{} date = '{}' ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update date field")
            }
        }
        match &media_link {
            Some(val) => {
                affected_col = format!("{} media_link = '{}' ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update media_link field")
            }
        }
        match &org_name {
            Some(val) => {
                affected_col = format!("{} org_name = '{}' ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update org_name field")
            }
        }
        match &org_link {
            Some(val) => {
                affected_col = format!("{} org_link = '{}' ,", &affected_col, val);
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
            .expect("Failed to prepare update Talk command.");

        let exe = stmt
            .execute([*talk_id])
            .await
            .expect("Failed to update a Talk.");
        info!("Update Execution returned: {}", exe);

        Some(TalkCommandStatus::Updated)
    }
}
