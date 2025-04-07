pub mod blogs;
pub mod talks;

use std::collections::HashMap;

use libsql::{Builder, Connection};
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct TursoDatabase {
    pub conn: Connection,
}

impl TursoDatabase {
    pub async fn new(
        mode: String,
        database_url: String,
        database_token: Option<String>,
    ) -> TursoDatabase {
        info!("Setting up Database...");
        let db: libsql::Database = match mode.as_str() {
            "sqlite" => {
                debug!("Setup a SQLITE connection to {}", &database_url);
                Builder::new_local(database_url)
                    .build()
                    .await
                    .expect("Failed to build SQLITE database.")
            }
            "turso" => {
                debug!("Setup a remote Turso connection to {}", &database_url);
                Builder::new_remote(database_url, database_token.unwrap())
                    .build()
                    .await
                    .expect("Failed to build turso database.")
            }
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
            .expect("Failed to setup connection to the database.");

        // Checking schema migrations
        let migration_commands = HashMap::from([
            (
                "Blogs Migration",
                r#"CREATE TABLE IF NOT EXISTS blogs (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            source TEXT NOT NULL,
            filename TEXT NOT NULL,
            body TEXT NOT NULL);"#,
            ),
            (
                "Talks Migration",
                r#"CREATE TABLE IF NOT EXISTS talks (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            date TEXT NOT NULL,
            media_link TEXT,
            org_name TEXT,
            org_link TEXT);"#,
            ),
        ]);

        for (mig_name, mig_command) in &migration_commands {
            info!("Starting {mig_name}...");
            let _migration = conn
                .execute(mig_command, ())
                .await
                .expect("Failed to migrate tables.");
        }
        info!("Database Setup is finished");

        TursoDatabase { conn }
    }
}
