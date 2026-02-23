use crate::handler::admin::blogs::displays::get_admin_blogs_list;
use crate::handler::admin::blogs::process_blog_body;
use crate::handler::auth::{process_login_header, verify_jwt};
use crate::handler::status::{
    get_401_unauthorized, get_404_not_found, get_500_internal_server_error,
};
use crate::model::axum::AppState;
use crate::model::blog_tag_mappings::{BlogTagMapping, BlogTagMappingCommandStatus};
use crate::model::blogs::{BlogCommandStatus, BlogsParams};
use crate::model::tags::{Tag, TagsListParams};
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use tracing::{debug, error, info, warn};

/// post_add_admin_blog
/// Serve POST add blog endpoint
#[debug_handler]
pub async fn post_add_admin_blog(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Html<String> {
    let (user_agent, token) = process_login_header(headers.clone()).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
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

    // Check if tags is available. if not add the new tag
    // No need to check tags. We only provide available tags for now
    let tag_db_uc = app_state.tag_db_usecase.lock().await.clone();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_tags_cache_enabled = tags_cache_uc_opt.is_some();

    if tag_db_uc.is_none() {
        error!("Failed to lock tag usecase mutex");
        return get_500_internal_server_error();
    }

    // Get Data from Cache or Database
    let mut available_tags = if is_tags_cache_enabled {
        tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_display_repo
            .find_tags(TagsListParams {
                start: Some(0),
                end: Some(1000),
            })
            .await
    } else {
        None
    };

    if available_tags.is_none() {
        debug!("Tags Cache is empty. Getting data from database");
        available_tags = tag_db_uc
            .unwrap()
            .tag_display_repo
            .find_tags(TagsListParams {
                start: Some(0),
                end: Some(1000),
            })
            .await;
    }

    // Filter tag id from the response body in the available_tags
    if available_tags.is_none() {
        error!("Failed to get all tags");
        return get_500_internal_server_error();
    }

    let selected_tags: Vec<Tag> = available_tags
        .unwrap()
        .tags
        .iter()
        .filter(|t| blog.tags.clone().unwrap().contains(&t.name))
        .cloned()
        .collect();
    debug!("Selected Tags: {:?}", selected_tags);

    // Add blog_tag_mappings
    let blog_tag_mapping_uc = app_state.blog_tag_mapping_db_usecase.lock().await.clone();
    // TODO: implement caching for blog tag mapping

    if blog_tag_mapping_uc.is_none() {
        error!("Failed to lock blog tag mapping usecase mutex");
        return get_500_internal_server_error();
    }

    for tag in selected_tags {
        let added_mapping = blog_tag_mapping_uc
            .clone()
            .unwrap()
            .blog_tag_mapping_repo
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
    let (user_agent, token) = process_login_header(headers.clone()).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
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

    // Get selected tags id
    let tag_db_uc = app_state.tag_db_usecase.lock().await.clone();
    let tags_cache_uc_opt = app_state.tag_cache_usecase.lock().await.clone();
    let is_tags_cache_enabled = tags_cache_uc_opt.is_some();

    if tag_db_uc.is_none() {
        error!("Failed to lock tag usecase mutex");
        return get_500_internal_server_error();
    }

    // Get Data from Cache or Database
    let mut tags_result = if is_tags_cache_enabled {
        tags_cache_uc_opt
            .clone()
            .unwrap()
            .tag_display_repo
            .find_tags(TagsListParams {
                start: Some(0),
                end: Some(1000),
            })
            .await
    } else {
        None
    };

    if tags_result.is_none() {
        debug!("Tags Cache is empty. Getting data from database");
        tags_result = tag_db_uc
            .unwrap()
            .tag_display_repo
            .find_tags(TagsListParams {
                start: Some(0),
                end: Some(1000),
            })
            .await;
    }

    let Some(tags) = tags_result else {
        error!("Failed to get Tags with id from 0 - 1000");
        return get_500_internal_server_error();
    };

    let selected_tag_ids: Vec<i64> = tags
        .tags
        .iter()
        .filter(|t| blog.tags.clone().unwrap().contains(&t.name))
        .map(|t| t.id)
        .collect();
    debug!("Selected Tag IDs {:?}", &selected_tag_ids);

    // Get blog tag mapping by blog id and tag id
    let blog_tag_mapping_uc = app_state.blog_tag_mapping_db_usecase.lock().await.clone();
    // TODO: implement caching for blog tag mapping

    if blog_tag_mapping_uc.is_none() {
        error!("Failed to lock blog tag mapping usecase mutex");
        return get_500_internal_server_error();
    }

    let btm_result = blog_tag_mapping_uc
        .clone()
        .unwrap()
        .blog_tag_mapping_repo
        .find_by_blog_id(id)
        .await;
    let Some(btm) = btm_result else {
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
    for delete_tag in delete_plan_mapping {
        info!(
            "Deleting Blog Tag Mapping for Blog ID {} and Tag ID {}",
            &delete_tag.blog_id, &delete_tag.tag_id
        );
        let delete_map_result = blog_tag_mapping_uc
            .clone()
            .unwrap()
            .blog_tag_mapping_repo
            .delete_by_blog_id_and_tag_id(delete_tag.blog_id, delete_tag.tag_id)
            .await;

        if delete_map_result.is_none()
            || delete_map_result.unwrap() != BlogTagMappingCommandStatus::Deleted
        {
            info!(
                "Failed to delete Blog Tag Mapping with Blog Id {} and Tag Id {}.",
                &delete_tag.blog_id, &delete_tag.tag_id
            );
            return get_500_internal_server_error();
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
        let add_map_result = blog_tag_mapping_uc
            .clone()
            .unwrap()
            .blog_tag_mapping_repo
            .add(id, add_tag_id)
            .await;

        if add_map_result.is_none()
            || add_map_result.unwrap() != BlogTagMappingCommandStatus::Stored
        {
            error!(
                "Failed to add Blog Tag Mapping for Blog ID {} and Tag ID {}",
                &id, &add_tag_id
            );
            return get_500_internal_server_error();
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
    let (user_agent, token) = process_login_header(headers.clone()).unwrap();
    info!("User Agent: {} and JWT processed", user_agent);

    if !verify_jwt(&token, &app_state.config.secrets.jwt_secret) {
        info!("Unauthorized access.");
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

    let blog_tag_mapping_uc = app_state.blog_tag_mapping_db_usecase.lock().await.clone();
    // TODO: implement caching for blog tag mapping
    if blog_tag_mapping_uc.is_none() {
        error!("Failed to lock blog tag mapping usecase mutex");
        return get_500_internal_server_error();
    }

    let deleted_mappings = blog_tag_mapping_uc
        .clone()
        .unwrap()
        .blog_tag_mapping_repo
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

    let query_params = BlogsParams {
        start: None,
        end: None,
        tags: None,
    };

    get_admin_blogs_list(State(app_state), headers, Query(query_params)).await
}
