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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub fn data_size(&self) -> u32 {
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
            None => Some(empty_value.clone()),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
                debug!("TalkParams: set default end to 10");
                10_i64
            }
        };

        TalksParams {
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TalkCommandStatus {
    Stored,
    Updated,
    Deleted,
    CacheInserted,
    CacheInvalidated,
}
