pub mod displays;
pub mod operations;

use crate::model::talks::Talk;

/// sanitize media and org part of Talk fields
pub fn sanitize_talk_media_org(talk_data: &Talk) -> (String, String, String) {
    let empty_value = "".to_string();

    let media_link = match &talk_data.media_link {
        Some(val) => val.clone(),
        None => empty_value.clone(),
    };
    let org_name = match &talk_data.org_name {
        Some(val) => val.clone(),
        None => empty_value.clone(),
    };
    let org_link = match &talk_data.org_link {
        Some(val) => val.clone(),
        None => empty_value.clone(),
    };

    (media_link, org_name, org_link)
}
