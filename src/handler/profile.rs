use crate::handler::status::get_500_internal_server_error;
use crate::model::templates::ProfileTemplate;
use askama::Template;
use axum::response::Html;
use tracing::{error, info};

/// get_profile
/// Serve Profile/Biography HTML file
pub async fn get_profile() -> Html<String> {
    let profile = ProfileTemplate.render();
    match profile {
        Ok(res) => {
            info!("Profile askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render profile.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_get_profile_renders_template() {
        let html = get_profile().await;
        assert!(html
            .0
            .contains("Husni Naufal Zuhdi - Site Reliability Engineer"));
    }
}
