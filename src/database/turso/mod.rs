pub mod auth;
pub mod blog_tag_mappings;
pub mod blogs;
pub mod profile;
pub mod tags;
pub mod talks;

use std::collections::HashMap;

use libsql::{Builder, Connection};
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct TursoDatabase {
    pub conn: Connection,
}

impl TursoDatabase {
    pub async fn new(mode: String, database_url: String, database_token: Option<String>) -> Self {
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
                r"CREATE TABLE IF NOT EXISTS blogs (
                    id INTEGER PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    source TEXT NOT NULL,
                    filename TEXT NOT NULL,
                    body TEXT NOT NULL
                    );",
            ),
            (
                "Talks Migration",
                r"CREATE TABLE IF NOT EXISTS talks (
                    id INTEGER PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    date TEXT NOT NULL,
                    media_link TEXT,
                    org_name TEXT,
                    org_link TEXT
                    );",
            ),
            (
                "2025-04-28 Tags Migration",
                r"CREATE TABLE IF NOT EXISTS tags (
                    id INTEGER PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL
                    );",
            ),
            (
                "2025-04-28 Blog Tag Mapping Migration",
                r"CREATE TABLE IF NOT EXISTS blog_tag_mapping(
                    blog_ref INTEGER NOT NULL,
                    tag_ref INTEGER NOT NULL
                    );",
            ),
            (
                "2025-09-23 User Migration",
                r"CREATE TABLE IF NOT EXISTS users (
                    id TEXT NOT NULL,
                    email TEXT NOT NULL,
                    hashed_password TEXT NOT NULL
                )",
            ),
            (
                "2025-09-23 Session Migration",
                r"CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT NOT NULL,
                    user_id TEXT NOT NULL,
                    token TEXT NOT NULL,
                    expire TEXT NOT NULL
                )",
            ),
            (
                "2025-10-24 Profile Sections Migration",
                r"CREATE TABLE IF NOT EXISTS profile_sections (
                    id INTEGER PRIMARY KEY NOT NULL,
                    position INTEGER NOT NULL,
                    title TEXT NOT NULL,
                    body TEXT NOT NULL,
                    layout TEXT NOT NULL
                    );",
            ),
            (
                "2025-10-24 Profile Meta Migration",
                r"CREATE TABLE IF NOT EXISTS profile_meta (
                    id INTEGER PRIMARY KEY NOT NULL,
                    description TEXT NOT NULL
                    );",
            ),
        ]);

        for (mig_name, mig_command) in &migration_commands {
            info!("Starting {mig_name}...");
            let _migration = conn
                .execute(mig_command, ())
                .await
                .expect("Failed to migrate tables.");
        }

        seed_profile_tables(&conn).await;
        info!("Database Setup is finished");

        Self { conn }
    }
}

/// Seed the Profile tables when they are empty.
/// Idempotent: runs on every startup and only inserts rows when the
/// corresponding table has no rows yet.
async fn seed_profile_tables(conn: &Connection) {
    let count = table_count(conn, "profile_sections").await;
    if count == 0 {
        info!("Seeding profile sections...");
        for (idx, (title, layout, body)) in profile::PROFILE_SECTIONS_SEED.iter().enumerate() {
            let position = (idx + 1) as i64;
            let command = "INSERT INTO profile_sections (id, position, title, body, layout)
                VALUES (?1, ?2, ?3, ?4, ?5)";
            let _ = conn
                .execute(command, (position, position, *title, *body, *layout))
                .await
                .expect("Failed to seed profile section.");
        }
    }

    let count = table_count(conn, "profile_meta").await;
    if count == 0 {
        info!("Seeding profile meta...");
        let command = "INSERT INTO profile_meta (id, description) VALUES (1, ?1)";
        let _ = conn
            .execute(command, [profile::PROFILE_META_SEED])
            .await
            .expect("Failed to seed profile meta.");
    }
}

/// Count rows of a table
async fn table_count(conn: &Connection, table_name: &str) -> i64 {
    let query = format!("SELECT COUNT(*) FROM {}", table_name);
    let stmt = conn
        .prepare(&query)
        .await
        .unwrap_or_else(|_| panic!("Failed to prepare row count query of {table_name}."));
    let mut rows = stmt
        .query(())
        .await
        .unwrap_or_else(|_| panic!("Failed to query row count of {table_name}."));
    let row = rows
        .next()
        .await
        .unwrap_or_else(|_| panic!("Failed to access row count row of {table_name}."))
        .expect("Failed to access row count row of {table_name}.");
    row.get(0).unwrap()
}

/// Trimming the last ',' for UPDATE command
fn trim_update_fields(affected_columns: &str) -> String {
    affected_columns[0..affected_columns.len() - 1].to_string()
}
