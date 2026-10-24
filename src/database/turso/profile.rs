use crate::database::turso::TursoDatabase;
use crate::model::profile::*;
use crate::repo::profile::{ProfileDisplayRepo, ProfileOperationRepo};
use async_trait::async_trait;
use std::str::FromStr;
use tracing::{debug, info};

#[async_trait]
impl ProfileDisplayRepo for TursoDatabase {
    async fn find_sections(&self) -> Option<ProfileSections> {
        let prep_query =
            "SELECT id, position, title, body, layout FROM profile_sections ORDER BY position ASC";
        debug!("Executing query {}", &prep_query);

        let stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find profile sections query.");

        let mut rows = stmt
            .query(())
            .await
            .expect("Failed to query profile sections.");

        let mut sections: Vec<ProfileSection> = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .expect("Failed to access profile sections row.")
        {
            debug!("Find Profile Section {:?}", &row);
            let layout_str: String = row.get(4).unwrap();
            sections.push(ProfileSection {
                id: row.get(0).unwrap(),
                position: row.get(1).unwrap(),
                title: row.get(2).unwrap(),
                body: row.get(3).unwrap(),
                layout: Layout::from_str(&layout_str).unwrap_or(Layout::Stacked),
            });
        }

        Some(ProfileSections { sections })
    }
    async fn find_section(&self, id: i64) -> Option<ProfileSection> {
        let prep_query =
            "SELECT id, position, title, body, layout FROM profile_sections WHERE id = ?1";
        debug!("Executing query {} for id {}", &prep_query, &id);

        let stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find profile section query.");

        let mut rows = stmt
            .query([id])
            .await
            .expect("Failed to query profile section.");

        let row = rows
            .next()
            .await
            .expect("Failed to access profile section row.");
        match row {
            Some(row) => {
                debug!("Find Profile Section {:?}", &row);
                let layout_str: String = row.get(4).unwrap();
                Some(ProfileSection {
                    id: row.get(0).unwrap(),
                    position: row.get(1).unwrap(),
                    title: row.get(2).unwrap(),
                    body: row.get(3).unwrap(),
                    layout: Layout::from_str(&layout_str).unwrap_or(Layout::Stacked),
                })
            }
            None => {
                debug!("Profile Section with id {} is not found.", id);
                None
            }
        }
    }
    async fn find_meta(&self) -> Option<ProfileMeta> {
        let prep_query = "SELECT id, description FROM profile_meta ORDER BY id ASC LIMIT 1";
        debug!("Executing query {}", &prep_query);

        let stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare find profile meta query.");

        let mut rows = stmt.query(()).await.expect("Failed to query profile meta.");

        let row = rows
            .next()
            .await
            .expect("Failed to access profile meta row.");
        match row {
            Some(row) => {
                debug!("Find Profile Meta {:?}", &row);
                Some(ProfileMeta {
                    id: row.get(0).unwrap(),
                    description: row.get(1).unwrap(),
                })
            }
            None => {
                debug!("Profile Meta is not found.");
                None
            }
        }
    }
}

#[async_trait]
impl ProfileOperationRepo for TursoDatabase {
    async fn get_new_id(&self) -> Option<i64> {
        let prep_query = "SELECT COALESCE(MAX(id), 0) + 1 FROM profile_sections";
        debug!("Executing query {}", &prep_query);

        let stmt = self
            .conn
            .prepare(prep_query)
            .await
            .expect("Failed to prepare new profile section id query.");

        let mut rows = stmt
            .query(())
            .await
            .expect("Failed to query new profile section id.");
        let row = rows
            .next()
            .await
            .expect("Failed to access new profile section id row.")
            .expect("Failed to access new profile section id.");

        let new_id: i64 = row.get(0).unwrap();
        Some(new_id)
    }
    async fn add(&mut self, section: ProfileSection) -> Option<ProfileCommandStatus> {
        // A new section always appends at the end of the page
        let prep_add_command = "INSERT INTO profile_sections (id, position, title, body, layout)
            VALUES (?1, COALESCE((SELECT MAX(position) FROM profile_sections), 0) + 1, ?2, ?3, ?4)";
        debug!(
            "Executing query {} for id {}",
            &prep_add_command, &section.id
        );

        let stmt = self
            .conn
            .prepare(prep_add_command)
            .await
            .expect("Failed to prepare add profile section command.");

        let exe = stmt
            .execute((
                section.id,
                section.title.clone(),
                section.body.clone(),
                section.layout.to_string(),
            ))
            .await
            .expect("Failed to add a profile section.");
        info!("Add Execution returned: {}", exe);

        Some(ProfileCommandStatus::Stored)
    }
    async fn update(&mut self, section: ProfileSection) -> Option<ProfileCommandStatus> {
        let prep_update_command = "UPDATE profile_sections
            SET title = ?1, body = ?2, layout = ?3
            WHERE id = ?4";
        debug!(
            "Executing query {} for id {}",
            &prep_update_command, &section.id
        );

        let stmt = self
            .conn
            .prepare(prep_update_command)
            .await
            .expect("Failed to prepare update profile section command.");

        let exe = stmt
            .execute((
                section.title.clone(),
                section.body.clone(),
                section.layout.to_string(),
                section.id,
            ))
            .await
            .expect("Failed to update a profile section.");
        info!("Update Execution returned: {}", exe);

        Some(ProfileCommandStatus::Updated)
    }
    async fn delete(&mut self, id: i64) -> Option<ProfileCommandStatus> {
        // The page title section (lowest position) is protected from deletion
        debug!(
            "Checking deletion protection for profile section id {}",
            &id
        );

        // Multiple statements are not supported by one libsql prepare; use two queries
        let stmt = self
            .conn
            .prepare("SELECT position FROM profile_sections WHERE id = ?1")
            .await
            .expect("Failed to prepare section position query.");
        let mut rows = stmt
            .query([id])
            .await
            .expect("Failed to query section position.");
        let section_position: Option<i64> = rows
            .next()
            .await
            .expect("Failed to access section position row.")
            .map(|row| row.get(0).unwrap());

        let section_position = match section_position {
            Some(val) => val,
            None => {
                debug!("Profile Section with id {} is not found.", id);
                return None;
            }
        };

        let stmt = self
            .conn
            .prepare("SELECT MIN(position) FROM profile_sections")
            .await
            .expect("Failed to prepare min position query.");
        let mut rows = stmt.query(()).await.expect("Failed to query min position.");
        let row = rows
            .next()
            .await
            .expect("Failed to access min position row.")
            .expect("Failed to access min position.");
        let min_position: i64 = row.get(0).unwrap();

        if section_position == min_position {
            info!("Refused to delete the page title section with id {}", id);
            return None;
        }

        let prep_delete_command = "DELETE FROM profile_sections WHERE id = ?1";
        debug!("Executing query {} for id {}", &prep_delete_command, &id);

        let stmt = self
            .conn
            .prepare(prep_delete_command)
            .await
            .expect("Failed to prepare delete profile section command.");

        let exe = stmt
            .execute([id])
            .await
            .expect("Failed to delete a profile section.");
        debug!("Delete Execution returned: {}", exe);

        Some(ProfileCommandStatus::Deleted)
    }
    async fn move_section(
        &mut self,
        id: i64,
        direction: MoveDirection,
    ) -> Option<ProfileCommandStatus> {
        // Get the current position of the section
        let stmt = self
            .conn
            .prepare("SELECT position FROM profile_sections WHERE id = ?1")
            .await
            .expect("Failed to prepare section position query.");
        let mut rows = stmt
            .query([id])
            .await
            .expect("Failed to query section position.");
        let row = rows
            .next()
            .await
            .expect("Failed to access section position row.")
            .expect("Failed to access section position.");
        let current_position: i64 = row.get(0).unwrap();

        // Find the adjacent neighbor to swap position with
        let (prep_neighbor_query, order_dir) = match direction {
            MoveDirection::Up => (
                "SELECT id, position FROM profile_sections WHERE position < ?1
                 ORDER BY position DESC LIMIT 1",
                "Up",
            ),
            MoveDirection::Down => (
                "SELECT id, position FROM profile_sections WHERE position > ?1
                 ORDER BY position ASC LIMIT 1",
                "Down",
            ),
        };
        debug!(
            "Executing query {} for id {} direction {}",
            &prep_neighbor_query, &id, order_dir
        );

        let stmt = self
            .conn
            .prepare(prep_neighbor_query)
            .await
            .expect("Failed to prepare neighbor query.");
        let mut rows = stmt
            .query([current_position])
            .await
            .expect("Failed to query neighbor.");
        let neighbor = rows.next().await.expect("Failed to access neighbor row.");

        let (neighbor_id, neighbor_position) = match neighbor {
            Some(row) => (row.get::<i64>(0).unwrap(), row.get::<i64>(1).unwrap()),
            None => {
                // Section is already at the edge; nothing to swap
                debug!("No neighbor for section id {} direction {}", id, order_dir);
                return Some(ProfileCommandStatus::Moved);
            }
        };

        // Swap positions
        let stmt = self
            .conn
            .prepare("UPDATE profile_sections SET position = ?1 WHERE id = ?2")
            .await
            .expect("Failed to prepare swap position command.");
        let exe = stmt
            .execute((neighbor_position, id))
            .await
            .expect("Failed to swap section position.");
        debug!("Swap Execution returned: {}", exe);

        let stmt = self
            .conn
            .prepare("UPDATE profile_sections SET position = ?1 WHERE id = ?2")
            .await
            .expect("Failed to prepare swap neighbor position command.");
        let exe = stmt
            .execute((current_position, neighbor_id))
            .await
            .expect("Failed to swap neighbor position.");
        debug!("Swap Execution returned: {}", exe);

        Some(ProfileCommandStatus::Moved)
    }
    async fn update_meta(&mut self, meta: ProfileMeta) -> Option<ProfileCommandStatus> {
        // Update the first meta row; when absent, seed it
        let prep_update_command = "UPDATE profile_meta SET description = ?1 WHERE id = ?2";
        debug!(
            "Executing query {} for id {}",
            &prep_update_command, &meta.id
        );

        let stmt = self
            .conn
            .prepare(prep_update_command)
            .await
            .expect("Failed to prepare update profile meta command.");
        let exe = stmt
            .execute((meta.description.clone(), meta.id))
            .await
            .expect("Failed to update profile meta.");
        debug!("Update Execution returned: {}", exe);

        if exe == 0 {
            info!("Profile Meta row is absent. Seeding it.");
            let prep_insert_command = "INSERT INTO profile_meta (id, description) VALUES (1, ?1)";
            let stmt = self
                .conn
                .prepare(prep_insert_command)
                .await
                .expect("Failed to prepare seed profile meta command.");
            stmt.execute([meta.description.clone()])
                .await
                .expect("Failed to seed profile meta.");
        }

        Some(ProfileCommandStatus::Updated)
    }
}

/// Seed content of the Profile page sections.
/// (title, layout, markdown body). The first section is the page title
/// section: no heading, and its body starts with the markdown h1.
/// External links use raw HTML anchors to preserve `target="_blank"`.
pub const PROFILE_SECTIONS_SEED: [(&str, &str, &str); 7] = [
    (
        "",
        "stacked",
        r"# Husni Naufal Zuhdi - Site Reliability Engineer",
    ),
    (
        "Overview",
        "stacked",
        r"Hi! I'm a Site Reliability Engineer by job and Software Engineer by passion. I'm comfortable writing code with python, golang, and rust.

I love exploring several topics such as Cloud Computing, DevOps, Platform Engineering, Nuclear Science, and Computational Physics.",
    ),
    (
        "Work Experiences",
        "stacked",
        r#"### 2022 - Now  | Site Reliability Engineer at <a href="https://accelbyte.io/about-us" target="_blank">Accelbyte</a>

- Manage and operate Accelbyte game-platform cloud infrastructure
- Handle incident management related to client and internal cloud infrastructure
- Develop Backend Service for internal alerting subscription

### 2022 - 2022 | System Engineer at <a href="https://www.tokopedia.com/about/" target="_blank">Tokopedia</a>

- Handle Tokopedia Multi-Cloud Environment
- Onboard several existing Jenkins Pipelines into our new GitHub Action Workflows
- Rolling out our existing Consul Service Discovery into the Enterprise version
- Develop Cloud Platform Diary for our Internal Cloud-related Data Analytics Platform"#,
    ),
    (
        "Volunteers",
        "stacked",
        r#"### 2024 - Now   | Organizer at <a href="https://devops-jogja.github.io" target="_blank">DevOps Jogja</a>

- Coordinate with several co-working spaces to hold the first Community Meetup
- Organized community meetups and collaborate with other communities in Indonesia
- Sometimes I present DevOps/SRE/Cloud materials
- Having fun with community members

### 2023 - Now   | Instructor and Advisor at <a href="https://grow.google/intl/id_id/bangkit" target="_blank">Bangkit Academy</a>

- Teaching on 20+ Cloud ILT sessions in Bangkit 2023
- Colaborate with Mentors to setup ILT sessions
- Advise 3 capstone teams to deliver their final projects

### 2022 - 2022  | Facilitator at <a href="https://grow.google/intl/id_id/bangkit" target="_blank">Bangkit Academy</a>

- Colaborate with Instructors to setup ILT sessions
- Conduct weekly meeting to monitor students progress and play games (to refresh their mind)
- During the final project period, conduct 1-on-1 session to give advices to students"#,
    ),
    (
        "Skills",
        "cards",
        r#"### Cloud Computing

Mostly manage and operating GCP and AWS in day to day basis

- <a href="https://www.credly.com/badges/a76fc573-ac7d-479a-a391-9b597b667850/public_url" target="_blank">AWS Certified Developer – Associate</a>
- <a href="https://www.credential.net/c2cbc5cd-b8ca-41fc-8417-f618631078e1" target="_blank">GCP Certified Associate Cloud Engineer</a>

### Kubernetes

Manage hundreds of Kubernetes clusters and create my own cluster

- <a href="https://www.credly.com/badges/a7ef8fab-b735-4c73-9fe2-39e14e2a458d/public_url" target="_blank">K8S Certified Kubernetes Administrator</a>

### Programming (Rust, Go, Python)

Build, Deploy, Maintain, and Operate internal infra software for my organizations

Love writting simple codes that works and helpful

### Terraform

Develop and maintain internal terraform modules

### Linux

Use Linux and in daily basis. Experienced delete whole database cluster"#,
    ),
    (
        "Projects",
        "cards",
        r#"### <a href="https://github.com/husni-zuhdi/husni-portfolio" target="_blank">Portfolio Website</a>

Build in **Rust** programming language, **Tokio** ecosystem, and tokio **Jinja** templating

Deployed in **Cloud Run (GCP)** with **$0/month** operating cost

### <a href="https://github.com/husni-zuhdi/ilt-cloud-bangkit-demo" target="_blank">Bangkit ILT Demo List</a>

Catalogue of my demo session when I teached Cloud Computing in Bangkit Academy

Most of the demo have 2 ways to interact with GCP: 1) **ClickOps/Scripting** 2) **Terraform**

### <a href="https://github.com/husni-zuhdi/Sapa.AI" target="_blank">Sapa.AI</a>

Design **microservice architecture** to serve simple CURD API and inference ML API

Develop CRUD and ML APIs with **Flask** and **Tensorflow** frameworks

**Lead** team of 6 people to deliver a violence reporting android app

### <a href="https://github.com/husni-zuhdi/husni-undergrad-thesis" target="_blank">Bachelor Thesis</a>

Design a neutronic fusion reactor simulation with **Paramak and OpenMC** libraries

Optimize simulation workflow to run in Cloud-Native environment

Running **parallel simulations** on top of GCP and AWS cloud providers to speed-up simulation time"#,
    ),
    (
        "Personal Interests",
        "stacked",
        r"*Running, Hard Sci-fi and Fantasy, Books, Climbing, Staring*",
    ),
];

/// Seed description of the Profile page meta
pub const PROFILE_META_SEED: &str = "Husni Zuhdi Profile";
