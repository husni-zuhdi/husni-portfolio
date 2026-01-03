use crate::database::turso::{trim_update_fields, TursoDatabase};
use crate::model::auth::*;
use crate::repo::auth::AuthRepo;
use async_trait::async_trait;
use tracing::{debug, info};

#[async_trait]
impl AuthRepo for TursoDatabase {
    async fn find_user_by_id(&self, id: String) -> Option<User> {
        let prep_query = "SELECT id, email, hashed_password FROM users WHERE id=?1 LIMIT 1";
        debug!("Executing query {} for id {}", &prep_query, &id);

        let stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find user query");

        let res = stmt
            .query([id.clone()])
            .await
            .expect("Failed to query user")
            .next()
            .await
            .expect("Failed to access query result");

        let Some(row) = res else {
            debug!("No User with Id {} is available.", &id);
            return None;
        };

        let user = User {
            id: row.get(0).unwrap(),
            email: row.get(1).unwrap(),
            hashed_password: row.get(2).unwrap(),
        };
        debug!("User id {:?} found", &user.id);
        Some(user)
    }
    async fn find_user_by_email(&self, email: String) -> Option<User> {
        let prep_query = "SELECT id, email, hashed_password FROM users WHERE email=?1 LIMIT 1";
        debug!("Executing query {} for email {}", &prep_query, &email);

        let stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find user query");

        let res = stmt
            .query([email.clone()])
            .await
            .expect("Failed to query user")
            .next()
            .await
            .expect("Failed to access query result");

        let Some(row) = res else {
            debug!("No User with email {} is available.", &email);
            return None;
        };

        let user = User {
            id: row.get(0).unwrap(),
            email: row.get(1).unwrap(),
            hashed_password: row.get(2).unwrap(),
        };
        debug!("User id {:?} found", &user.id);
        Some(user)
    }
    async fn add_user(
        &self,
        id: String,
        email: String,
        hpass: String,
    ) -> Option<UserCommandStatus> {
        let prep_add_command = "INSERT INTO users (id, email, hashed_password) VALUES (?1, ?2, ?3)";
        debug!("Executing query {} for id {}", &prep_add_command, &id);

        let stmt = self
            .conn
            .prepare(prep_add_command)
            .await
            .expect("Failed to prepare add user commmand");

        let exe = stmt
            .execute((id.clone(), email.clone(), hpass.clone()))
            .await
            .expect("Failed to add user.");
        debug!("Add Execution returned: {}", exe);

        Some(UserCommandStatus::Stored)
    }
    async fn update_user(
        &self,
        id: String,
        email: Option<String>,
        hpass: Option<String>,
    ) -> Option<UserCommandStatus> {
        let mut affected_col = "".to_string();
        match &email {
            Some(val) => {
                affected_col = format!("{} email = '{}' ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update email field")
            }
        }
        match &hpass {
            Some(val) => {
                affected_col = format!("{} hashed_password = '{}' ,", &affected_col, val);
                debug!("Affected Column: '{}'", &affected_col)
            }
            None => {
                debug!("Skipped update hashed_password field")
            }
        }

        affected_col = trim_update_fields(&affected_col);
        let prep_update_query = format!("UPDATE users SET{}WHERE id = ?1", &affected_col);
        debug!("Executing query {} for id {}", &prep_update_query, &id);

        let stmt = self
            .conn
            .prepare(&prep_update_query)
            .await
            .expect("Failed to prepare update user command");

        let exe = stmt.execute([id]).await.expect("Failed to update a User");
        info!("Update Execution returned: {}", exe);

        Some(UserCommandStatus::Updated)
    }
    async fn delete_user(&self, id: String) -> Option<UserCommandStatus> {
        let prep_delete_command = "DELETE FROM users WHERE id = ?1";
        debug!("Executing query {} for id {}", &prep_delete_command, &id);

        let stmt = self
            .conn
            .prepare(prep_delete_command)
            .await
            .expect("Failed to prepare delete user command.");

        let exe = stmt.execute([id]).await.expect("Failed to delete a User.");

        debug!("Delete Execution returned: {}", exe);
        Some(UserCommandStatus::Deleted)
    }
    async fn find_session(&self, id: String) -> Option<Session> {
        let prep_query = "SELECT id, user_id, token, expire FROM sessions WHERE id=?1 LIMIT 1";
        debug!("Executing query {} for id {}", &prep_query, &id);

        let stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find session query");

        let res = stmt
            .query([id.clone()])
            .await
            .expect("Failed to query session")
            .next()
            .await
            .expect("Failed to access query result");

        let Some(row) = res else {
            debug!("No Session with Id {} is available.", &id);
            return None;
        };

        debug!("Debug Row {:?}", &row);
        Some(Session {
            id: row.get(0).unwrap(),
            user_id: row.get(1).unwrap(),
            token: row.get(2).unwrap(),
            expire: row.get(3).unwrap(),
        })
    }
    async fn add_session(
        &self,
        id: String,
        user_id: String,
        token: String,
        expire: String,
    ) -> Option<SessionCommandStatus> {
        let prep_add_command =
            "INSERT INTO sessions (id, user_id, token, expire) VALUES (?1, ?2, ?3, ?4)";
        debug!("Executing query {} for id {}", &prep_add_command, &id);

        let stmt = self
            .conn
            .prepare(prep_add_command)
            .await
            .expect("Failed to prepare add session commmand");

        let exe = stmt
            .execute((id.clone(), user_id.clone(), token.clone(), expire.clone()))
            .await
            .expect("Failed to add session");
        debug!("Add Execution returned: {}", exe);

        Some(SessionCommandStatus::Stored)
    }
    async fn delete_session(&self, id: String) -> Option<SessionCommandStatus> {
        let prep_delete_command = "DELETE FROM sessions WHERE id = ?1";
        debug!("Executing query {} for id {}", &prep_delete_command, &id);

        let stmt = self
            .conn
            .prepare(prep_delete_command)
            .await
            .expect("Failed to prepare delete session command");

        let exe = stmt
            .execute([id])
            .await
            .expect("Failed to delete a Session");

        debug!("Delete Execution returned: {}", exe);
        Some(SessionCommandStatus::Deleted)
    }
}
