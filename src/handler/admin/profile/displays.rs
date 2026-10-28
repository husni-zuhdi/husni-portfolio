use crate::handler::auth::is_auth_verified;
use crate::handler::status::{
    get_401_unauthorized, get_404_not_found, get_500_internal_server_error,
};
use crate::model::axum::AppState;
use crate::model::templates_admin::{
    AdminGetAddProfileSectionTemplate, AdminGetDeleteProfileSectionTemplate,
    AdminGetEditProfileSectionTemplate, AdminGetProfileSectionTemplate,
    AdminListProfileSectionsTemplate, AdminProfileMetaEditTemplate, AdminProfileTemplate,
};
use askama::Template;
use axum::debug_handler;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Html;
use tracing::{debug, error, info, warn};

/// get_base_admin_profile
/// Serve GET (base) admin profile HTML file
/// Under endpoint /admin/profile
/// It's the base of Admin Profile feature
#[debug_handler]
pub async fn get_base_admin_profile(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    let profile_res = AdminProfileTemplate {}.render();
    match profile_res {
        Ok(res) => {
            info!("Admin Profile askama template rendered.");
            Html(res)
        }
        Err(err) => {
            error!("Failed to render admin/profile/profile.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_admin_profile_list
/// Serve to list profile sections HTML file and return point for
/// /admin/profile/add cancel button
#[debug_handler]
pub async fn get_admin_profile_list(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    render_profile_list(app_state, String::new()).await
}

/// render_profile_list
/// Render the profile section list with an optional error message.
/// Used by mutation endpoints to surface failures (e.g. protected deletion)
pub async fn render_profile_list(app_state: AppState, error_message: String) -> Html<String> {
    let mut profile_uc = app_state.profile_usecase.lock().await.clone();

    // The sections read rebuilds the page cache when the cache is cold.
    let sections = profile_uc.find_sections().await;
    let Some(sections_data) = sections else {
        error!("Failed to find profile sections.");
        return get_500_internal_server_error();
    };

    let list_res = AdminListProfileSectionsTemplate {
        sections: sections_data.to_admin_template(),
        error_message,
    }
    .render();
    match list_res {
        Ok(html) => {
            info!("AdminListProfileSections askama template rendered.");
            Html(html)
        }
        Err(err) => {
            error!("Failed to render admin/profile/list_sections.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_admin_profile_section
/// Serve GET a single profile section HTML file and return point for
/// several cancelation endpoints
#[debug_handler]
pub async fn get_admin_profile_section(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    let mut profile_uc = app_state.profile_usecase.lock().await.clone();
    let section_result = profile_uc.find_section(id).await;

    // Early check result. If empty, return 404 error
    let Some(section) = section_result else {
        info!("Failed to find Profile Section with Id {}.", &id);
        return get_404_not_found().await;
    };

    // Flag the page title section so the row renders the [Title] badge
    let is_title = is_protected_section(&app_state, &section).await;
    let section_res = AdminGetProfileSectionTemplate {
        section: section.to_admin_template(is_title),
    }
    .render();
    match section_res {
        Ok(html) => {
            info!("AdminGetProfileSection askama template rendered.");
            Html(html)
        }
        Err(err) => {
            error!("Failed to render admin/profile/get_section.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// get_add_admin_profile_section
/// Serve GET add profile section HTML file in a form format
#[debug_handler]
pub async fn get_add_admin_profile_section(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    // Calculate new Profile Section Id
    let result = {
        let mut profile_uc = app_state.profile_usecase.lock().await.clone();
        profile_uc.get_new_id().await
    };
    if result.is_none() {
        info!("Failed to get new Profile Section Id.");
        return get_404_not_found().await;
    }

    let section_id = result.unwrap();
    debug!(
        "Construct AdminGetAddProfileSectionTemplate for Section Id {}",
        &section_id
    );
    let add_section_res = AdminGetAddProfileSectionTemplate { id: section_id }.render();
    match add_section_res {
        Ok(html) => {
            info!("AdminGetAddProfileSection askama template rendered.");
            Html(html)
        }
        Err(err) => {
            error!(
                "Failed to render admin/profile/get_add_section.html. {}",
                err
            );
            get_500_internal_server_error()
        }
    }
}

/// get_edit_admin_profile_section
/// Serve GET edit profile section HTML file to edit a section
#[debug_handler]
pub async fn get_edit_admin_profile_section(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    let mut profile_uc = app_state.profile_usecase.lock().await.clone();
    let section_result = profile_uc.find_section(id).await;

    // Early check result. If empty, return 404 error
    let Some(section) = section_result else {
        info!("Failed to find Profile Section with Id {}.", &id);
        return get_404_not_found().await;
    };

    let edit_section_res = AdminGetEditProfileSectionTemplate {
        section: section.to_admin_template(false),
    }
    .render();
    match edit_section_res {
        Ok(html) => {
            info!("AdminGetEditProfileSection askama template rendered.");
            Html(html)
        }
        Err(err) => {
            error!(
                "Failed to render admin/profile/get_edit_section.html. {}",
                err
            );
            get_500_internal_server_error()
        }
    }
}

/// get_delete_admin_profile_section
/// Serve GET delete profile section HTML file to delete a section.
/// The page title section (first by position) is protected from deletion.
#[debug_handler]
pub async fn get_delete_admin_profile_section(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    let mut profile_uc = app_state.profile_usecase.lock().await.clone();
    let section_result = profile_uc.find_section(id).await;

    // Early check result. If empty, return 404 error
    let Some(section) = section_result else {
        info!("Failed to find Profile Section with Id {}.", &id);
        return get_404_not_found().await;
    };

    let protected = is_protected_section(&app_state, &section).await;
    let delete_section_res = AdminGetDeleteProfileSectionTemplate {
        id: section.id,
        protected,
    }
    .render();
    match delete_section_res {
        Ok(html) => {
            info!("AdminGetDeleteProfileSection askama template rendered.");
            Html(html)
        }
        Err(err) => {
            error!(
                "Failed to render admin/profile/get_delete_section.html. {}",
                err
            );
            get_500_internal_server_error()
        }
    }
}

/// get_admin_profile_meta_edit
/// Serve GET edit profile meta HTML file in a form format
#[debug_handler]
pub async fn get_admin_profile_meta_edit(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers, &app_state.config.secrets.jwt_secret) {
        return get_401_unauthorized().await;
    }

    let mut profile_uc = app_state.profile_usecase.lock().await.clone();
    match profile_uc.find_meta().await {
        Some(meta) => render_meta_edit(meta.description).await,
        None => {
            info!("Profile Meta is not found. Render an empty description.");
            render_meta_edit(String::new()).await
        }
    }
}

/// render_meta_edit
/// Render the profile meta edit form with the given description
async fn render_meta_edit(description: String) -> Html<String> {
    let meta_edit_res = AdminProfileMetaEditTemplate { description }.render();
    match meta_edit_res {
        Ok(html) => {
            info!("AdminProfileMetaEdit askama template rendered.");
            Html(html)
        }
        Err(err) => {
            error!("Failed to render admin/profile/meta_edit.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// is_protected_section
/// Check whether the section is the page title section (lowest position)
async fn is_protected_section(
    app_state: &AppState,
    section: &crate::model::profile::ProfileSection,
) -> bool {
    let mut profile_uc = app_state.profile_usecase.lock().await.clone();

    match profile_uc.find_sections().await {
        Some(sections) => sections
            .first()
            .map(|first| first.id == section.id)
            .unwrap_or(false),
        None => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::profile::{Layout, ProfileSection, ProfileSections};

    fn sample_title_section() -> ProfileSection {
        ProfileSection {
            id: 1,
            position: 1,
            title: String::new(),
            body: "# Husni Naufal Zuhdi - Site Reliability Engineer".to_string(),
            layout: Layout::Stacked,
        }
    }

    /// Regression test: the standalone section view (returned after an edit
    /// is accepted or canceled) must render the same row markup as the
    /// section list so the swapped-in fragment looks identical to the
    /// original list row.
    #[test]
    fn test_get_section_row_matches_list_row_markup() {
        let section = sample_title_section();

        // Standalone row fragment (get_section.html)
        let row = AdminGetProfileSectionTemplate {
            section: section.to_admin_template(true),
        }
        .render()
        .unwrap();

        // The list renders the same section via the shared partial
        let list = AdminListProfileSectionsTemplate {
            sections: ProfileSections {
                sections: vec![section],
            }
            .to_admin_template(),
            error_message: String::new(),
        }
        .render()
        .unwrap();

        // Both must carry the same table row content and controls
        for html in [&row, &list] {
            assert!(html.contains("<tr id=\"profile_section_1_target\">"));
            assert!(html.contains("[Title] 1"));
            assert!(html.contains("(Page Title Section)"));
            assert!(html.contains("<td>stacked</td>"));
            assert!(html.contains("hx-get=\"/admin/profile/1/edit\""));
            assert!(html.contains("hx-swap=\"outerHTML\""));
            assert!(html.contains("hx-get=\"/admin/profile/1/delete\""));
            assert!(html.contains("hx-post=\"/admin/profile/1/move?dir=up\""));
            assert!(html.contains("hx-post=\"/admin/profile/1/move?dir=down\""));
        }

        // The list renders a table with headers; the row view must not
        // duplicate the table element itself
        assert!(list.contains("<table"));
        assert!(list.contains("<th>Position</th>"));
        assert!(!row.contains("<table"));
    }
}
