use crate::model::templates::ProfileSectionTemplate;
use crate::model::templates_admin::AdminProfileSectionTemplate;
use crate::utils::convert_markdown_to_html;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use tracing::{debug, warn};

/// Layout
/// How a Profile Section is rendered on the Profile page:
/// - Stacked: render body as a normal document flow (like Work Experiences)
/// - Cards: render body wrapped in a flex-container grid (like Skills/Projects)
///
/// Unknown layout values fall back to `Stacked`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Layout {
    Stacked,
    Cards,
}

impl FromStr for Layout {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cards" => Ok(Layout::Cards),
            "stacked" => Ok(Layout::Stacked),
            _ => Err(()),
        }
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stacked => write!(f, "stacked"),
            Self::Cards => write!(f, "cards"),
        }
    }
}

impl Layout {
    /// CSS class applied to the section body wrapper.
    /// `Cards` renders inside a `flex-container` grid; `Stacked` uses none.
    pub const fn as_css_class(&self) -> &'static str {
        match self {
            Self::Stacked => "",
            Self::Cards => "flex-container",
        }
    }
}

/// MoveDirection
/// Direction of a Profile Section move operation:
/// - Up: swap position with the section above (lower position)
/// - Down: swap position with the section below (higher position)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
}

/// ProfileCommandStatus
/// Status of Profile Command Operations:
/// - Stored
/// - Updated
/// - Deleted
/// - Moved
/// - CacheInserted
/// - CacheInvalidated
///
/// I think you should wrap this with Option so you can check if it `None`
/// then check the value of the status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProfileCommandStatus {
    Stored,
    Updated,
    Deleted,
    Moved,
    CacheInserted,
    CacheInvalidated,
}

/// ProfileSection
/// A section of the Profile page with fields:
/// - id: Section Identifier
/// - position: Rendering order (ascending); the lowest position is the page title section
/// - title: Section heading; empty string means no heading is rendered
/// - body: Section content in markdown
/// - layout: How the section body is rendered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSection {
    pub id: i64,
    pub position: i64,
    pub title: String,
    pub body: String,
    pub layout: Layout,
}

impl ProfileSection {
    /// Convert ProfileSection to (Askama) ProfileSectionTemplate
    /// The markdown body is converted to sanitized HTML
    pub fn to_template(&self) -> ProfileSectionTemplate {
        debug!(
            "ProfileSection: Constructing ProfileSectionTemplate for Section Id {}",
            &self.id
        );
        ProfileSectionTemplate {
            id: self.id,
            position: self.position,
            title: self.title.clone(),
            html_body: convert_markdown_to_html(&self.body),
            css_class: self.layout.as_css_class().to_string(),
        }
    }
    /// Convert ProfileSection to (Askama) AdminProfileSectionTemplate
    /// The body stays as raw markdown for editing
    pub fn to_admin_template(&self, is_title: bool) -> AdminProfileSectionTemplate {
        debug!(
            "ProfileSection: Constructing AdminProfileSectionTemplate for Section Id {}",
            &self.id
        );
        AdminProfileSectionTemplate {
            id: self.id,
            position: self.position,
            title: self.title.clone(),
            body: self.body.clone(),
            layout: self.layout.to_string(),
            is_title,
        }
    }
    /// Calculate size of ProfileSection in u32
    /// Useful for weighing data size
    pub const fn data_size(&self) -> u32 {
        (size_of_val(&self.id)
            + size_of_val(&self.position)
            + size_of_val(&self.title)
            + size_of_val(&self.body)
            + size_of_val(&self.layout)) as u32
    }
}

/// ProfileSections
/// Collection of Profile Sections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSections {
    pub sections: Vec<ProfileSection>,
}

impl ProfileSections {
    /// Sort sections by position ascending
    pub fn sort_by_position(&mut self) {
        self.sections.sort_by_key(|section| section.position);
    }
    /// Borrow the section with the lowest position (the page title section)
    pub fn first(&self) -> Option<&ProfileSection> {
        self.sections.iter().min_by_key(|section| section.position)
    }
    /// Extract the page title from the first section body.
    /// The first section body is expected to start with a markdown h1
    /// (`# Title`); take its first line and strip leading `#`.
    /// Return empty string when there is no section.
    pub fn page_title(&self) -> String {
        let Some(first) = self.first() else {
            return String::new();
        };
        first
            .body
            .lines()
            .next()
            .unwrap_or_default()
            .trim_start_matches('#')
            .trim()
            .to_string()
    }
    /// Convert ProfileSections to (Askama) ProfileSectionTemplates
    /// ordered by position
    pub fn to_template(&self) -> Vec<ProfileSectionTemplate> {
        let mut sections = self.sections.clone();
        sections.sort_by_key(|section| section.position);
        sections
            .iter()
            .map(|section| section.to_template())
            .collect()
    }
    /// Convert ProfileSections to (Askama) AdminProfileSectionTemplates
    /// ordered by position. The first section is flagged as the title section
    pub fn to_admin_template(&self) -> Vec<AdminProfileSectionTemplate> {
        let mut sections = self.sections.clone();
        sections.sort_by_key(|section| section.position);
        let title_id = sections.first().map(|section| section.id);
        sections
            .iter()
            .map(|section| {
                let is_title = title_id == Some(section.id);
                section.to_admin_template(is_title)
            })
            .collect()
    }
}

/// ProfileMeta
/// Profile page metadata with fields:
/// - id: ProfileMeta Identifier
/// - description: HTML meta description of the Profile page
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileMeta {
    pub id: i64,
    pub description: String,
}

impl ProfileMeta {
    /// Calculate size of ProfileMeta in u32
    /// Useful for weighing data size
    pub const fn data_size(&self) -> u32 {
        (size_of_val(&self.id) + size_of_val(&self.description)) as u32
    }
}

/// ProfilePage
/// The whole editable content of the Profile page:
/// - sections: Ordered Profile Sections
/// - meta: Profile page metadata
///
/// Used as the unit of cache storage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfilePage {
    pub sections: ProfileSections,
    pub meta: ProfileMeta,
}

impl ProfilePage {
    /// Calculate size of ProfilePage in u32
    /// Useful for weighing data size
    pub fn data_size(&self) -> u32 {
        let mut size = self.meta.data_size();
        for section in &self.sections.sections {
            size = size.saturating_add(section.data_size());
        }
        size
    }
}

/// MoveParams
/// Axum Query parameters for the section move endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveParams {
    pub dir: Option<String>,
}

impl MoveParams {
    /// Sanitize MoveParams into a MoveDirection.
    /// Unknown or missing value defaults to `Up` with a warning.
    pub fn sanitize(&self) -> MoveDirection {
        match self.dir.as_deref() {
            Some("down") => MoveDirection::Down,
            Some(dir) => {
                warn!("Unknown move direction {}. Default to Up.", dir);
                MoveDirection::Up
            }
            None => {
                warn!("Missing move direction. Default to Up.");
                MoveDirection::Up
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn sample_section(id: i64, position: i64) -> ProfileSection {
        ProfileSection {
            id,
            position,
            title: "Overview".to_string(),
            body: "Hello World".to_string(),
            layout: Layout::Stacked,
        }
    }

    #[test]
    fn test_layout_from_str() {
        assert_eq!(Layout::from_str("stacked"), Ok(Layout::Stacked));
        assert_eq!(Layout::from_str("cards"), Ok(Layout::Cards));
        assert_eq!(Layout::from_str("unknown"), Err(()));
        assert_eq!(Layout::from_str(""), Err(()));
    }

    #[test]
    fn test_layout_display() {
        assert_eq!(Layout::Stacked.to_string(), "stacked");
        assert_eq!(Layout::Cards.to_string(), "cards");
    }

    #[test]
    fn test_layout_as_css_class() {
        assert_eq!(Layout::Stacked.as_css_class(), "");
        assert_eq!(Layout::Cards.as_css_class(), "flex-container");
    }

    #[test]
    fn test_profile_section_to_template() {
        let section = ProfileSection {
            id: 1,
            position: 2,
            title: "Overview".to_string(),
            body: "# Hello\n\nWorld".to_string(),
            layout: Layout::Stacked,
        };
        let template = section.to_template();
        assert_eq!(template.id, 1);
        assert_eq!(template.position, 2);
        assert_eq!(template.title, "Overview");
        assert!(template.html_body.contains("<h1>Hello</h1>"));
        assert!(template.html_body.contains("<p>World</p>"));
        assert_eq!(template.css_class, "");
    }

    #[test]
    fn test_profile_section_to_template_cards_layout() {
        let section = ProfileSection {
            layout: Layout::Cards,
            ..sample_section(1, 1)
        };
        let template = section.to_template();
        assert_eq!(template.css_class, "flex-container");
    }

    #[test]
    fn test_profile_section_to_admin_template() {
        let section = sample_section(1, 3);
        let template = section.to_admin_template(true);
        assert_eq!(template.id, 1);
        assert_eq!(template.position, 3);
        assert_eq!(template.title, "Overview");
        assert_eq!(template.body, "Hello World");
        assert_eq!(template.layout, "stacked");
        assert!(template.is_title);
    }

    #[test]
    fn test_profile_section_data_size() {
        assert!(sample_section(1, 1).data_size() > 0);
    }

    #[test]
    fn test_profile_sections_sort_and_to_template() {
        let mut sections = ProfileSections {
            sections: vec![
                ProfileSection {
                    id: 2,
                    position: 5,
                    ..sample_section(2, 5)
                },
                ProfileSection {
                    id: 1,
                    position: 1,
                    ..sample_section(1, 1)
                },
            ],
        };
        sections.sort_by_position();
        assert_eq!(sections.sections[0].id, 1);
        assert_eq!(sections.first().unwrap().id, 1);

        let templates = sections.to_template();
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0].id, 1);
        assert_eq!(templates[1].id, 2);
    }

    #[test]
    fn test_profile_sections_page_title() {
        let sections = ProfileSections {
            sections: vec![
                ProfileSection {
                    id: 2,
                    position: 2,
                    title: "Overview".to_string(),
                    body: "Overview body".to_string(),
                    layout: Layout::Stacked,
                },
                ProfileSection {
                    id: 1,
                    position: 1,
                    title: String::new(),
                    body: "# Husni Naufal Zuhdi - Site Reliability Engineer".to_string(),
                    layout: Layout::Stacked,
                },
            ],
        };
        assert_eq!(
            sections.page_title(),
            "Husni Naufal Zuhdi - Site Reliability Engineer"
        );
    }

    #[test]
    fn test_profile_sections_page_title_empty() {
        let sections = ProfileSections { sections: vec![] };
        assert_eq!(sections.page_title(), "");
    }

    #[test]
    fn test_profile_sections_to_admin_template_flags_first_as_title() {
        let sections = ProfileSections {
            sections: vec![
                ProfileSection {
                    id: 2,
                    position: 5,
                    ..sample_section(2, 5)
                },
                ProfileSection {
                    id: 1,
                    position: 1,
                    ..sample_section(1, 1)
                },
            ],
        };
        let templates = sections.to_admin_template();
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0].id, 1);
        assert!(templates[0].is_title);
        assert_eq!(templates[1].id, 2);
        assert!(!templates[1].is_title);
    }

    #[test]
    fn test_profile_meta_data_size() {
        let meta = ProfileMeta {
            id: 1,
            description: "Husni Zuhdi Profile".to_string(),
        };
        assert!(meta.data_size() > 0);
    }

    #[test]
    fn test_profile_page_data_size() {
        let page = ProfilePage {
            sections: ProfileSections {
                sections: vec![sample_section(1, 1)],
            },
            meta: ProfileMeta {
                id: 1,
                description: "Husni Zuhdi Profile".to_string(),
            },
        };
        assert!(page.data_size() > 0);
    }

    #[test]
    fn test_move_params_sanitize() {
        assert_eq!(
            MoveParams {
                dir: Some("up".to_string())
            }
            .sanitize(),
            MoveDirection::Up
        );
        assert_eq!(
            MoveParams {
                dir: Some("down".to_string())
            }
            .sanitize(),
            MoveDirection::Down
        );
        assert_eq!(
            MoveParams {
                dir: Some("sideways".to_string())
            }
            .sanitize(),
            MoveDirection::Up
        );
        assert_eq!(MoveParams { dir: None }.sanitize(), MoveDirection::Up);
    }
}
