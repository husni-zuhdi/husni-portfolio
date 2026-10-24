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
use crate::repo::profile::ProfileDisplayRepo;
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

    get_admin_profile_list_with_error(app_state, String::new()).await
}

/// get_admin_profile_list_with_error
/// Render the profile section list with an optional error message.
/// Used by mutation endpoints to surface failures (e.g. protected deletion)
pub async fn get_admin_profile_list_with_error(
    app_state: AppState,
    error_message: String,
) -> Html<String> {
    let profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        profile_cache_uc_opt.clone().unwrap().find_sections().await
    } else {
        None
    };
    // If cache hit, return early
    if let Some(res) = cache_result {
        let list_res = AdminListProfileSectionsTemplate {
            sections: res.to_admin_template(),
            error_message,
        }
        .render();
        match list_res {
            Ok(html) => {
                info!("AdminListProfileSections askama template rendered.");
                return Html(html);
            }
            Err(err) => {
                error!("Failed to render admin/profile/list_sections.html. {}", err);
                return get_500_internal_server_error();
            }
        }
    }

    // If not, get data from database
    let db_result = profile_db_uc.find_sections().await;
    if db_result.is_none() {
        error!("Failed to find profile sections.");
        return get_500_internal_server_error();
    }

    // Insert cache
    if is_cache_enabled {
        let meta = profile_db_uc.find_meta().await;
        debug!("Caching profile page");
        let _ = profile_cache_uc_opt
            .clone()
            .unwrap()
            .profile_operation_repo
            .insert_page(crate::model::profile::ProfilePage {
                sections: db_result.clone().unwrap(),
                meta: meta.unwrap_or(crate::model::profile::ProfileMeta {
                    id: 0,
                    description: String::new(),
                }),
            })
            .await;
    }

    // Render Admin List Profile Sections
    let list_res = AdminListProfileSectionsTemplate {
        sections: db_result.unwrap().to_admin_template(),
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

    let profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    // Sanitize `path`
    let id = path.parse::<i64>();
    match &id {
        Ok(val) => {
            debug!("Successfully parse path {} into {} i64", &path, &val);
        }
        Err(err) => {
            warn!("Failed to parse path {} to i64. Err: {}", &path, err);
            return get_404_not_found().await;
        }
    };

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        profile_cache_uc_opt
            .clone()
            .unwrap()
            .find_section(id.clone().unwrap())
            .await
    } else {
        None
    };
    // If cache hit, return early
    if let Some(res) = cache_result {
        // Flag the page title section so the row renders the [Title] badge
        let is_title = is_protected_section(&app_state, &res).await;
        let section_res = AdminGetProfileSectionTemplate {
            section: res.to_admin_template(is_title),
        }
        .render();
        match section_res {
            Ok(html) => {
                info!("AdminGetProfileSection askama template rendered.");
                return Html(html);
            }
            Err(err) => {
                error!("Failed to render admin/profile/get_section.html. {}", err);
                return get_500_internal_server_error();
            }
        }
    }

    // If not, get data from database
    let db_result = profile_db_uc.find_section(id.clone().unwrap()).await;
    if db_result.is_none() {
        info!(
            "Failed to find Profile Section with Id {}.",
            &id.clone().unwrap()
        );
        return get_404_not_found().await;
    }

    // Insert cache
    if is_cache_enabled {
        let sections = profile_db_uc.find_sections().await;
        let meta = profile_db_uc.find_meta().await;
        if let (Some(sections), Some(meta)) = (sections, meta) {
            debug!("Caching profile page");
            let _ = profile_cache_uc_opt
                .clone()
                .unwrap()
                .profile_operation_repo
                .insert_page(crate::model::profile::ProfilePage { sections, meta })
                .await;
        }
    }

    // Render Profile Section
    let section = db_result.unwrap();
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

    let profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    // Calculate new Profile Section Id
    let result = profile_db_uc.profile_operation_repo.get_new_id().await;
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

    let profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    // Sanitize `path`
    let id = path.parse::<i64>();
    match &id {
        Ok(val) => {
            debug!("Successfully parse path {} into {} i64", &path, &val);
        }
        Err(err) => {
            warn!("Failed to parse path {} to i64. Err: {}", &path, err);
            return get_404_not_found().await;
        }
    };

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        profile_cache_uc_opt
            .clone()
            .unwrap()
            .find_section(id.clone().unwrap())
            .await
    } else {
        None
    };
    // If cache hit, return early
    if let Some(res) = cache_result {
        let edit_section_res = AdminGetEditProfileSectionTemplate {
            section: res.to_admin_template(false),
        }
        .render();
        match edit_section_res {
            Ok(html) => {
                info!("AdminGetEditProfileSection askama template rendered.");
                return Html(html);
            }
            Err(err) => {
                error!(
                    "Failed to render admin/profile/get_edit_section.html. {}",
                    err
                );
                return get_500_internal_server_error();
            }
        }
    }

    // If not, get data from database
    let db_result = profile_db_uc.find_section(id.clone().unwrap()).await;
    if db_result.is_none() {
        info!(
            "Failed to find Profile Section with Id {}.",
            &id.clone().unwrap()
        );
        return get_404_not_found().await;
    }

    // Insert cache
    if is_cache_enabled {
        let sections = profile_db_uc.find_sections().await;
        let meta = profile_db_uc.find_meta().await;
        if let (Some(sections), Some(meta)) = (sections, meta) {
            debug!("Caching profile page");
            let _ = profile_cache_uc_opt
                .clone()
                .unwrap()
                .profile_operation_repo
                .insert_page(crate::model::profile::ProfilePage { sections, meta })
                .await;
        }
    }

    // Render Edit Profile Section
    let edit_section_res = AdminGetEditProfileSectionTemplate {
        section: db_result.unwrap().to_admin_template(false),
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

    let profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    // Sanitize `path`
    let id = path.parse::<i64>();
    match &id {
        Ok(val) => {
            debug!("Successfully parse path {} into {} i64", &path, &val);
        }
        Err(err) => {
            warn!("Failed to parse path {} to i64. Err: {}", &path, err);
            return get_404_not_found().await;
        }
    };

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        profile_cache_uc_opt
            .clone()
            .unwrap()
            .find_section(id.clone().unwrap())
            .await
    } else {
        None
    };
    // If cache hit, check protection and render
    if let Some(res) = cache_result {
        let protected = is_protected_section(&app_state, &res).await;
        let delete_section_res = AdminGetDeleteProfileSectionTemplate {
            id: res.id,
            protected,
        }
        .render();
        match delete_section_res {
            Ok(html) => {
                info!("AdminGetDeleteProfileSection askama template rendered.");
                return Html(html);
            }
            Err(err) => {
                error!(
                    "Failed to render admin/profile/get_delete_section.html. {}",
                    err
                );
                return get_500_internal_server_error();
            }
        }
    }

    // If not, get data from database
    let db_result = profile_db_uc.find_section(id.clone().unwrap()).await;
    if db_result.is_none() {
        info!(
            "Failed to find Profile Section with Id {}.",
            &id.clone().unwrap()
        );
        return get_404_not_found().await;
    }

    // Insert cache
    if is_cache_enabled {
        let sections = profile_db_uc.find_sections().await;
        let meta = profile_db_uc.find_meta().await;
        if let (Some(sections), Some(meta)) = (sections, meta) {
            debug!("Caching profile page");
            let _ = profile_cache_uc_opt
                .clone()
                .unwrap()
                .profile_operation_repo
                .insert_page(crate::model::profile::ProfilePage { sections, meta })
                .await;
        }
    }

    // Check protection and render
    let section = db_result.unwrap();
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

    let profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    // Get Data from Cache
    let cache_result = if is_cache_enabled {
        profile_cache_uc_opt.clone().unwrap().find_meta().await
    } else {
        None
    };
    if let Some(meta) = cache_result {
        return render_meta_edit(meta.description).await;
    }

    // If not, get data from database
    let db_result = profile_db_uc.find_meta().await;
    match db_result {
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
    let profile_db_uc = app_state.profile_db_usecase.lock().await.clone();
    let profile_cache_uc_opt = app_state.profile_cache_usecase.lock().await.clone();
    let is_cache_enabled = profile_cache_uc_opt.is_some();

    let sections = if is_cache_enabled {
        let cache_result = profile_cache_uc_opt.clone().unwrap().find_sections().await;
        match cache_result {
            Some(res) => Some(res),
            None => profile_db_uc.find_sections().await,
        }
    } else {
        profile_db_uc.find_sections().await
    };

    match sections.and_then(|sections| sections.first().cloned()) {
        Some(first) => first.id == section.id,
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

        // Both must carry the same row content and controls
        for html in [&row, &list] {
            assert!(html.contains("[Title]"));
            assert!(html.contains("Position 1"));
            assert!(html.contains("(Page Title Section)"));
            assert!(html.contains("| Layout: stacked"));
            assert!(html.contains("hx-get=\"/admin/profile/1/edit\""));
            assert!(html.contains("hx-get=\"/admin/profile/1/delete\""));
            assert!(html.contains("hx-post=\"/admin/profile/1/move?dir=up\""));
            assert!(html.contains("hx-post=\"/admin/profile/1/move?dir=down\""));
        }
    }
}
