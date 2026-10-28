use crate::handler::admin::blogs::displays::get_admin_blogs_list;
use crate::handler::admin::blogs::process_blog_body;
use crate::handler::auth::csrf::verify_csrf_token;
use crate::handler::auth::is_auth_verified;
use crate::handler::status::{
    get_401_unauthorized, get_404_not_found, get_500_internal_server_error,
};
use crate::model::axum::AppState;
use crate::model::blog_tag_mappings::{BlogTagMapping, BlogTagMappingCommandStatus};
use crate::model::blogs::{BlogCommandStatus, BlogsParams};
use crate::model::tags::{Tag, TagCommandStatus, TagsListParams};
use crate::model::templates_admin::AdminBlogPreviewTemplate;
use crate::usecase::tags::{TagCacheUseCase, TagDBUseCase};
use crate::utils::convert_markdown_to_html;
use askama::Template;
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use tracing::{debug, error, info, warn};
use urlencoding::decode;

/// Ensure all tag names in `selected_names` exist, creating any missing tags.
/// Returns the corresponding `Tag` records with ids.
async fn ensure_selected_tags(
    selected_names: Vec<String>,
    tag_db_uc: &mut TagDBUseCase,
    tags_cache_uc_opt: &Option<TagCacheUseCase>,
    is_tags_cache_enabled: bool,
) -> Option<Vec<Tag>> {
    let available = tag_db_uc
        .tag_display_repo
        .find_tags(TagsListParams {
            start: Some(0),
            end: Some(1000),
        })
        .await?;

    let mut result = Vec::with_capacity(selected_names.len());
    let mut missing = Vec::new();

    for name in selected_names {
        match available.tags.iter().find(|t| t.name == name) {
            Some(tag) => result.push(tag.clone()),
            None => missing.push(name),
        }
    }

    for name in missing {
        let new_id = tag_db_uc.tag_operation_repo.get_new_id().await?;
        let add_status = tag_db_uc.tag_operation_repo.add(new_id, name.clone()).await;
        if add_status != Some(TagCommandStatus::Stored) {
            error!("Failed to create new tag {}", name);
            return None;
        }
        let tag = Tag {
            id: new_id,
            name: name.clone(),
        };
        if is_tags_cache_enabled {
            let _ = tags_cache_uc_opt
                .clone()
                .unwrap()
                .tag_operation_repo
                .insert(tag.clone())
                .await;
        }
        result.push(tag);
    }

    Some(result)
}

/// post_admin_blog_preview
/// Serve POST preview blog content endpoint.
/// Renders the markdown body without touching the database.
#[debug_handler]
pub async fn post_admin_blog_preview(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let mut blog_body = String::new();
    for req_field in body.split('&') {
        let Some((key, value)) = req_field.split_once('=') else {
            continue;
        };
        if key == "blog_body" {
            blog_body = decode(value).unwrap_or_default().to_string();
            break;
        }
    }

    let preview = AdminBlogPreviewTemplate {
        html: convert_markdown_to_html(&blog_body),
    }
    .render();

    match preview {
        Ok(html) => {
            info!("AdminBlogPreview askama template rendered.");
            Html(html)
        }
        Err(err) => {
            error!("Failed to render admin/blogs/preview.html. {}", err);
            get_500_internal_server_error()
        }
    }
}

/// post_add_admin_blog
/// Serve POST add blog endpoint
#[debug_handler]
pub async fn post_add_admin_blog(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    // Locking Mutex
    let mut blogs_db_uc = app_state.blog_db_usecase.lock().await.clone();
    let blogs_cache_uc_opt = app_state.blog_cache_usecase.lock().await.clone();
    let is_blogs_cache_enabled = blogs_cache_uc_opt.is_some();

    let blog = process_blog_body(body);
    let add_result = blogs_db_uc.blog_operation_repo.add(blog.clone()).await;

    if add_result.is_none() {
        info!("Failed to add blog with Id {}.", &blog.id);
        return get_404_not_found().await;
    }

    if add_result.unwrap() != BlogCommandStatus::Stored {
        error!("Failed to add blog with Id {}", &blog.id);
        return get_500_internal_server_error();
    }

    // Insert cache
    if is_blogs_cache_enabled {
        debug!("Caching blog {}", &blog.id);
        let _ = blogs_cache_uc_opt
            .clone()
            .unwrap()
            .blog_operation_repo
            .insert(blog.clone())
            .await;
    }

    // Ensure selected tags exist, creating any new tags inline.
    let mut tag_db_uc = app_state.tag_db_usecase.lock().await.clone();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_tags_cache_enabled = tags_cache_uc_opt.is_some();

    if tag_db_uc.is_none() {
        error!("Failed to lock tag usecase mutex");
        return get_500_internal_server_error();
    }

    let Some(selected_tags) = ensure_selected_tags(
        blog.tags.clone().unwrap_or_default(),
        tag_db_uc.as_mut().unwrap(),
        &tags_cache_uc_opt,
        is_tags_cache_enabled,
    )
    .await
    else {
        error!("Failed to ensure selected tags for blog {}", blog.id);
        return get_500_internal_server_error();
    };
    debug!("Selected Tags: {:?}", selected_tags);

    // Add blog_tag_mappings
    let btms_uc = app_state.blog_tag_mapping_db_usecase.lock().await.clone();
    let btms_cache_uc_opt = app_state
        .blog_tag_mapping_cache_usecase
        .lock()
        .await
        .clone();
    let is_btms_cache_enabled = btms_cache_uc_opt.is_some();

    if btms_uc.is_none() {
        error!("Failed to lock blog tag mapping usecase mutex");
        return get_500_internal_server_error();
    }

    for tag in selected_tags {
        let added_mapping = btms_uc
            .clone()
            .unwrap()
            .operation
            .add(blog.id, tag.id)
            .await;
        if added_mapping.is_none() {
            error!(
                "Failed to add blog tag mapping for blog id {} and tag id {}",
                blog.id.clone(),
                tag.id.clone()
            );
            return get_500_internal_server_error();
        }
        if added_mapping.unwrap() != BlogTagMappingCommandStatus::Stored {
            error!("Failed to add blog tag mapping for blog id {} and tag id {}. Command Status is not Stored", blog.id.clone(), tag.id.clone());
            return get_500_internal_server_error();
        }

        if is_btms_cache_enabled {
            debug!(
                "Caching Blog Tag Mapping  blog_id {}, tag_id {}",
                blog.id, tag.id
            );
            let _ = btms_cache_uc_opt
                .clone()
                .unwrap()
                .operation
                .insert(blog.id, tag.id)
                .await;
        }
    }

    let query_params = BlogsParams {
        start: None,
        end: None,
        tags: None,
    };

    get_admin_blogs_list(State(app_state), headers, Query(query_params)).await
}

/// put_edit_admin_blog
/// Serve PUT edit blog HTML file
#[debug_handler]
pub async fn put_edit_admin_blog(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let mut blogs_db_uc = app_state.blog_db_usecase.lock().await.clone();
    let blogs_cache_uc_opt = app_state.blog_cache_usecase.lock().await.clone();
    let is_blogs_cache_enabled = blogs_cache_uc_opt.is_some();

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    let blog = process_blog_body(body);
    let edit_result = blogs_db_uc.blog_operation_repo.update(blog.clone()).await;

    if edit_result.is_none() {
        info!("Failed to edit blog with Id {}.", &blog.id);
        return get_404_not_found().await;
    }

    if edit_result.unwrap() != BlogCommandStatus::Updated {
        error!("Failed to edit blog with Id {}", &blog.id);
        return get_500_internal_server_error();
    }

    // Re-insert cache
    if is_blogs_cache_enabled {
        debug!("Invalidating blog {}", &blog.id);
        let _ = blogs_cache_uc_opt
            .clone()
            .unwrap()
            .blog_operation_repo
            .invalidate(blog.id)
            .await;
        debug!("Re-caching tag {}", &blog.id);
        let _ = blogs_cache_uc_opt
            .clone()
            .unwrap()
            .blog_operation_repo
            .insert(blog.clone())
            .await;
    }

    // Ensure selected tags exist, creating any new tags inline.
    let mut tag_db_uc = app_state.tag_db_usecase.lock().await.clone();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_tags_cache_enabled = tags_cache_uc_opt.is_some();

    if tag_db_uc.is_none() {
        error!("Failed to lock tag usecase mutex");
        return get_500_internal_server_error();
    }

    let Some(selected_tags) = ensure_selected_tags(
        blog.tags.clone().unwrap_or_default(),
        tag_db_uc.as_mut().unwrap(),
        &tags_cache_uc_opt,
        is_tags_cache_enabled,
    )
    .await
    else {
        error!("Failed to ensure selected tags for blog {}", blog.id);
        return get_500_internal_server_error();
    };

    let selected_tag_ids: Vec<i64> = selected_tags.iter().map(|t| t.id).collect();
    debug!("Selected Tag IDs {:?}", &selected_tag_ids);

    // Get blog tag mapping by blog id and tag id
    let btms_uc = app_state.blog_tag_mapping_db_usecase.lock().await.clone();
    let btms_cache_uc_opt = app_state
        .blog_tag_mapping_cache_usecase
        .lock()
        .await
        .clone();
    let is_btms_cache_enabled = btms_cache_uc_opt.is_some();

    if btms_uc.is_none() {
        error!("Failed to lock blog tag mapping usecase mutex");
        return get_500_internal_server_error();
    }

    let btms_result = btms_uc.clone().unwrap().display.find_by_blog_id(id).await;
    let Some(btm) = btms_result else {
        error!("Failed to get Blog Tag Mapping for Blog ID {}", &id);
        return get_500_internal_server_error();
    };
    // Find tags not present in request but present in mapping
    // Delete those mapping
    let delete_plan_mapping: Vec<BlogTagMapping> = btm
        .maps
        .iter()
        .filter(|map| !selected_tag_ids.contains(&map.tag_id))
        .cloned()
        .collect();
    for delete_btm in delete_plan_mapping {
        info!(
            "Deleting Blog Tag Mapping for Blog ID {} and Tag ID {}",
            &delete_btm.blog_id, &delete_btm.tag_id
        );
        let delete_map_result = btms_uc
            .clone()
            .unwrap()
            .operation
            .delete_by_blog_id_and_tag_id(delete_btm.blog_id, delete_btm.tag_id)
            .await;

        if delete_map_result.is_none()
            || delete_map_result.unwrap() != BlogTagMappingCommandStatus::Deleted
        {
            info!(
                "Failed to delete Blog Tag Mapping with Blog Id {} and Tag Id {}.",
                &delete_btm.blog_id, &delete_btm.tag_id
            );
            return get_500_internal_server_error();
        }

        if is_btms_cache_enabled {
            debug!(
                "Invalidating Blog Tag Mapping cache blog_id {}, tag_id {}",
                &delete_btm.blog_id, &delete_btm.tag_id
            );
            let _ = btms_cache_uc_opt
                .clone()
                .unwrap()
                .operation
                .invalidate(delete_btm.blog_id, delete_btm.tag_id)
                .await;
        }
    }
    // Find tags not present in mapping but present in request
    // Add those mapping
    let add_plan_mapping: Vec<i64> = selected_tag_ids
        .iter()
        .filter(|tag_id| {
            !btm.maps.contains(&BlogTagMapping {
                blog_id: id,
                tag_id: **tag_id,
            })
        })
        .cloned()
        .collect();
    for add_tag_id in add_plan_mapping {
        info!(
            "Adding Blog Tag Mapping for Blog ID {} and Tag ID {}",
            &id, &add_tag_id
        );
        let add_map_result = btms_uc.clone().unwrap().operation.add(id, add_tag_id).await;

        if add_map_result.is_none()
            || add_map_result.unwrap() != BlogTagMappingCommandStatus::Stored
        {
            error!(
                "Failed to add Blog Tag Mapping for Blog ID {} and Tag ID {}",
                &id, &add_tag_id
            );
            return get_500_internal_server_error();
        }

        if is_btms_cache_enabled {
            debug!(
                "Re-Inserting Blog Tag Mapping cache blog_id {}, tag_id {}",
                &id, &add_tag_id
            );
            let _ = btms_cache_uc_opt
                .clone()
                .unwrap()
                .operation
                .invalidate(id, add_tag_id)
                .await;
            let _ = btms_cache_uc_opt
                .clone()
                .unwrap()
                .operation
                .insert(id, add_tag_id)
                .await;
        }
    }

    let query_params = BlogsParams {
        start: None,
        end: None,
        tags: None,
    };

    get_admin_blogs_list(State(app_state), headers, Query(query_params)).await
}

/// delete_delete_admin_blog
/// Serve DELETE delete blog HTML file
#[debug_handler]
pub async fn delete_delete_admin_blog(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    if !is_auth_verified(headers.clone(), &app_state.config.secrets.jwt_secret)
        || !verify_csrf_token(&headers)
    {
        return get_401_unauthorized().await;
    }

    let mut blogs_db_uc = app_state.blog_db_usecase.lock().await.clone();
    let blogs_cache_uc_opt = app_state.blog_cache_usecase.lock().await.clone();
    let is_blogs_cache_enabled = blogs_cache_uc_opt.is_some();

    // Sanitize `path`
    let Ok(id) = path.parse::<i64>() else {
        warn!("Failed to parse path {} to i64", &path);
        return get_404_not_found().await;
    };

    let delete_result = blogs_db_uc.blog_operation_repo.delete(id).await;

    if delete_result.is_none() || delete_result.unwrap() != BlogCommandStatus::Deleted {
        error!("Failed to delete blog with Id {}", id);
        return get_500_internal_server_error();
    }

    // Invalidate cache
    if is_blogs_cache_enabled {
        debug!("Invalidating blog {} cache", id);
        let _ = blogs_cache_uc_opt
            .clone()
            .unwrap()
            .blog_operation_repo
            .invalidate(id)
            .await;
    }

    let btms_uc = app_state.blog_tag_mapping_db_usecase.lock().await.clone();
    let btms_cache_uc_opt = app_state
        .blog_tag_mapping_cache_usecase
        .lock()
        .await
        .clone();
    let is_btms_cache_enabled = btms_cache_uc_opt.is_some();

    if btms_uc.is_none() {
        error!("Failed to lock blog tag mapping usecase mutex");
        return get_500_internal_server_error();
    }

    let deleted_mappings = btms_uc
        .clone()
        .unwrap()
        .operation
        .delete_by_blog_id(id)
        .await;
    if deleted_mappings.is_none() {
        error!("Failed to delete blog tag mappings for blog id {}", &id,);
        return get_500_internal_server_error();
    }
    if deleted_mappings.unwrap() != BlogTagMappingCommandStatus::Deleted {
        error!(
            "Failed to delete blog tag mapping for blog id {}. Command Status is not Deleted",
            &id,
        );
        return get_500_internal_server_error();
    }

    if is_btms_cache_enabled {
        debug!("Invalidating Blog Tag Mapping cache blog_id {}", &id);
        let invalidate_opt = btms_cache_uc_opt
            .clone()
            .unwrap()
            .operation
            .invalidate_by_blog_id(id)
            .await;
        if invalidate_opt.is_none() {
            error!("Failed to delete blog tag mappings for blog id {}", &id,);
        }
    }

    let query_params = BlogsParams {
        start: None,
        end: None,
        tags: None,
    };

    get_admin_blogs_list(State(app_state), headers, Query(query_params)).await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Config;
    use crate::state::state_factory;

    fn unauthorized_html(html: &Html<String>) -> bool {
        html.0.contains("401 Unauthorized")
    }

    #[tokio::test]
    async fn test_post_admin_blog_preview_unauthorized() {
        let state = state_factory(Config::default()).await;
        let headers = HeaderMap::new();
        let body = "blog_body=%23%20Hello".to_string();
        let result = post_admin_blog_preview(State(state), headers, body).await;
        assert!(unauthorized_html(&result));
    }
}
