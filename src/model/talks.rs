use crate::model::{
    templates::{TalkTemplate, TalksTemplate},
    templates_admin::{AdminListTalksTemplate, AdminTalkTemplate},
};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Talk
/// Talk data with fields:
/// - id: Talk Identifier
/// - name: Talk Name
/// - media_link: (Optional) Talk media (video/record) link
/// - org_link: (Optional) Talk organisation link
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Talk {
    pub id: i64,
    pub name: String,
    pub date: String,
    pub media_link: Option<String>,
    pub org_name: Option<String>,
    pub org_link: Option<String>,
}

impl Talk {
    /// Convert Talk to (Askama) TalkTemplate
    pub fn to_template(&self) -> TalkTemplate {
        debug!("Talk: Constructing TalkTemplate for Talk Id {}", &self.id);
        TalkTemplate {
            id: self.id,
            name: self.name.clone(),
            date: self.date.clone(),
            media_link: self.media_link.clone().unwrap(),
            org_name: self.org_name.clone().unwrap(),
            org_link: self.org_link.clone().unwrap(),
        }
    }
    /// Convert Talk to (Askama) AdminTalkTemplate
    pub fn to_admin_template(&self) -> AdminTalkTemplate {
        debug!(
            "Talk: Constructing AdminTalkTemplate for Talk Id {}",
            self.id
        );
        AdminTalkTemplate {
            id: self.id,
            name: self.name.clone(),
            date: self.date.clone(),
            media_link: self.media_link.clone().unwrap(),
            org_name: self.org_name.clone().unwrap(),
            org_link: self.org_link.clone().unwrap(),
        }
    }
    /// Calculate size of Talks in u32
    /// Useful for weighing data size
    pub const fn data_size(&self) -> u32 {
        (size_of_val(&self.id)
            + size_of_val(&self.name)
            + size_of_val(&self.date)
            + size_of_val(&self.org_name)
            + size_of_val(&self.org_link)
            + size_of_val(&self.media_link)) as u32
    }
    /// Sanitize media and org part of Talk by set default empty value if None
    pub fn sanitize_talk_media_org(&self) -> Self {
        let empty_value = "".to_string();

        let media_link = match &self.media_link {
            None => Some(empty_value.clone()),
            val => val.clone(),
        };
        let org_name = match &self.org_name {
            None => Some(empty_value.clone()),
            val => val.clone(),
        };
        let org_link = match &self.org_link {
            None => Some(empty_value),
            val => val.clone(),
        };

        Self {
            id: self.id,
            name: self.name.clone(),
            date: self.date.clone(),
            media_link,
            org_name,
            org_link,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Talks {
    pub talks: Vec<Talk>,
}

impl Talks {
    /// Sanitize Talks
    pub fn sanitize(&self) -> Self {
        Self {
            talks: self
                .talks
                .iter()
                .map(|talk| talk.sanitize_talk_media_org())
                .collect(),
        }
    }
    /// Convert Talks to (Askama) TalksTemplate
    pub fn to_template(&self) -> TalksTemplate {
        TalksTemplate {
            talks: self.talks.iter().map(|talk| talk.to_template()).collect(),
        }
    }
    /// Convert Talks to (Askama) AdminListTalksTemplate
    pub fn to_admin_list_template(&self) -> AdminListTalksTemplate {
        AdminListTalksTemplate {
            talks: self
                .talks
                .iter()
                .map(|admin_talk| admin_talk.to_admin_template())
                .collect(),
        }
    }
}

/// TalksParams
/// Axum parameters query for pagination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TalksParams {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

impl TalksParams {
    /// Sanitize TalksParams by checking negative value and set unknown to
    /// the default values
    pub fn sanitize(&self) -> Self {
        let start = match self.start {
            Some(val) if val >= 0 => val,
            _ => {
                debug!("TalkParams: set default start to 0");
                0_i64
            }
        };
        let end = match self.end {
            Some(val) if val >= 0 => val,
            _ => {
                debug!("TalkParams: set default end to 100");
                100_i64
            }
        };

        Self {
            start: Some(start),
            end: Some(end),
        }
    }
}

/// TalkCommandStatus
/// Status of Talk Command Operations:
/// - Stored
/// - Updated
/// - Deleted
/// - CacheInserted
/// - CacheInvalidated
///
/// I think you should wrap this with Option so you can check if it `None`
/// then check the value of the status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TalkCommandStatus {
    Stored,
    Updated,
    Deleted,
    CacheInserted,
    CacheInvalidated,
}

#[cfg(test)]
mod test {
    use super::*;

    fn sample_talk() -> Talk {
        Talk {
            id: 1,
            name: "Talk 1".to_string(),
            date: "2024-01-01".to_string(),
            media_link: Some("https://example.com/media".to_string()),
            org_name: Some("Org".to_string()),
            org_link: Some("https://example.com/org".to_string()),
        }
    }

    fn talk_with_empty_media_org() -> Talk {
        Talk {
            id: 2,
            name: "Talk 2".to_string(),
            date: "2024-02-02".to_string(),
            media_link: None,
            org_name: None,
            org_link: None,
        }
    }

    #[test]
    fn test_talk_to_template() {
        let template = sample_talk().to_template();
        assert_eq!(template.id, 1);
        assert_eq!(template.name, "Talk 1");
        assert_eq!(template.date, "2024-01-01");
        assert_eq!(template.media_link, "https://example.com/media");
        assert_eq!(template.org_name, "Org");
        assert_eq!(template.org_link, "https://example.com/org");
    }

    #[test]
    fn test_talk_to_admin_template() {
        let talk = sample_talk();
        let template = talk.to_admin_template();
        assert_eq!(template.id, talk.id);
        assert_eq!(template.name, talk.name);
        assert_eq!(template.date, talk.date);
        assert_eq!(template.media_link, talk.media_link.unwrap());
        assert_eq!(template.org_name, talk.org_name.unwrap());
        assert_eq!(template.org_link, talk.org_link.unwrap());
    }

    #[test]
    fn test_talk_sanitize_talk_media_org_fills_empty_values() {
        let sanitized = talk_with_empty_media_org().sanitize_talk_media_org();
        assert_eq!(sanitized.media_link, Some(String::new()));
        assert_eq!(sanitized.org_name, Some(String::new()));
        assert_eq!(sanitized.org_link, Some(String::new()));
        assert_eq!(sanitized.id, 2);
        assert_eq!(sanitized.name, "Talk 2");
        assert_eq!(sanitized.date, "2024-02-02");
    }

    #[test]
    fn test_talk_sanitize_talk_media_org_keeps_present_values() {
        let talk = sample_talk();
        let sanitized = talk.sanitize_talk_media_org();
        assert_eq!(sanitized.media_link, talk.media_link);
        assert_eq!(sanitized.org_name, talk.org_name);
        assert_eq!(sanitized.org_link, talk.org_link);
    }

    #[test]
    fn test_talk_data_size() {
        assert!(sample_talk().data_size() > 0);
    }

    #[test]
    fn test_talks_sanitize() {
        let talks = Talks {
            talks: vec![sample_talk(), talk_with_empty_media_org()],
        };
        let sanitized = talks.sanitize();
        assert_eq!(sanitized.talks.len(), 2);
        assert_eq!(
            sanitized.talks[0].media_link,
            Some("https://example.com/media".to_string())
        );
        assert_eq!(sanitized.talks[1].media_link, Some(String::new()));
        assert_eq!(sanitized.talks[1].org_name, Some(String::new()));
        assert_eq!(sanitized.talks[1].org_link, Some(String::new()));
    }

    #[test]
    fn test_talks_to_template() {
        let template = Talks {
            talks: vec![sample_talk()],
        }
        .to_template();
        assert_eq!(template.talks.len(), 1);
        assert_eq!(template.talks[0].id, 1);
        assert_eq!(template.talks[0].name, "Talk 1");
    }

    #[test]
    fn test_talks_to_admin_list_template() {
        let template = Talks {
            talks: vec![sample_talk()],
        }
        .to_admin_list_template();
        assert_eq!(template.talks.len(), 1);
        assert_eq!(template.talks[0].name, "Talk 1");
    }

    #[test]
    fn test_talks_params_sanitize_defaults() {
        let params = TalksParams {
            start: None,
            end: None,
        };
        assert_eq!(
            params.sanitize(),
            TalksParams {
                start: Some(0),
                end: Some(100)
            }
        );
    }

    #[test]
    fn test_talks_params_sanitize_negative_values() {
        let params = TalksParams {
            start: Some(-5),
            end: Some(-1),
        };
        assert_eq!(
            params.sanitize(),
            TalksParams {
                start: Some(0),
                end: Some(100)
            }
        );
    }

    #[test]
    fn test_talks_params_sanitize_valid_values() {
        let params = TalksParams {
            start: Some(5),
            end: Some(20),
        };
        assert_eq!(
            params.sanitize(),
            TalksParams {
                start: Some(5),
                end: Some(20)
            }
        );
    }

    #[test]
    fn test_talks_params_sanitize_mixed() {
        let params = TalksParams {
            start: Some(5),
            end: None,
        };
        assert_eq!(
            params.sanitize(),
            TalksParams {
                start: Some(5),
                end: Some(100)
            }
        );
    }
}
