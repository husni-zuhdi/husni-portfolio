use crate::handler::admin::blogs::displays::get_admin_blogs_list;
use crate::handler::admin::blogs::process_blog_body;
use crate::handler::auth::{process_login_header, verify_jwt};
use crate::handler::status::get_401_unauthorized;
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
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
    // TODO: Implement check and add on tags and blog_tag_mappings
    let mut blog_uc = app_state.blog_db_usecase.lock().await.clone();

    let blog = process_blog_body(body);
    let add_result = blog_uc.blog_repo.add(blog.clone()).await;

    match add_result {
        Some(blog_command_status) => {
            if blog_command_status != BlogCommandStatus::Stored {
                error!("Failed to add blog with Id {}", &blog.id);
                return get_500_internal_server_error();
            }
        }
        None => {
            info!("Failed to add blog with Id {}.", &blog.id);
            return get_404_not_found().await;
        }
    }

    // Check if tags is available. if not add the new tag
    // No need to check tags. We only provide available tags for now
    let tag_uc = app_state.tag_db_usecase.lock().await.clone();
    if tag_uc.is_none() {
        error!("Failed to lock tag usecase mutex");
        return get_500_internal_server_error();
    }

    let available_tags = tag_uc
        .unwrap()
        .tag_display_repo
        .find_tags(TagsListParams {
            start: Some(0),
            end: Some(1000),
        })
        .await;

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

    let mut blog_uc = app_state.blog_db_usecase.lock().await.clone();
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

    let blog = process_blog_body(body);

    let blog_result = blog_uc.blog_repo.update(blog.clone()).await;

    match blog_result {
        Some(blog_command_status) => {
            if blog_command_status != BlogCommandStatus::Updated {
                error!("Failed to edit Blog with Id {}", &path);
                return get_500_internal_server_error();
            }
        }
        None => {
            info!("Failed to edit Blog with Id {}.", &path);
            return get_404_not_found().await;
        }
    }

    // Get selected tags id
    let tag_uc = app_state.tag_db_usecase.lock().await.clone();
    if tag_uc.is_none() {
        error!("Failed to lock tag usecase mutex");
        return get_500_internal_server_error();
    }

    let tags_result = tag_uc
        .clone()
        .unwrap()
        .tag_display_repo
        .find_tags(TagsListParams {
            start: Some(0),
            end: Some(1000),
        })
        .await;
    let Some(tags) = tags_result else {
        error!("Failed to get all Tags");
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
    if blog_tag_mapping_uc.is_none() {
        error!("Failed to lock blog tag mapping usecase mutex");
        return get_500_internal_server_error();
    }

    let btm_result = blog_tag_mapping_uc
        .clone()
        .unwrap()
        .blog_tag_mapping_repo
        .find_by_blog_id(id.clone().unwrap())
        .await;
    let Some(btm) = btm_result else {
        error!(
            "Failed to get Blog Tag Mapping for Blog ID {}",
            id.clone().unwrap()
        );
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

        match delete_map_result {
            Some(btm_command_status) => {
                if btm_command_status != BlogTagMappingCommandStatus::Deleted {
                    error!(
                        "Failed to delete Blog Tag Mapping for Blog ID {} and Tag ID {}",
                        &delete_tag.blog_id, &delete_tag.tag_id
                    );
                    return get_500_internal_server_error();
                }
            }
            None => {
                info!("Failed to delete Blog Tag Mapping");
                return get_404_not_found().await;
            }
        }
    }
    // Find tags not present in mapping but present in request
    // Add those mapping
    let add_plan_mapping: Vec<i64> = selected_tag_ids
        .iter()
        .filter(|tag_id| {
            !btm.maps.contains(&BlogTagMapping {
                blog_id: id.clone().unwrap(),
                tag_id: **tag_id,
            })
        })
        .cloned()
        .collect();
    for add_tag_id in add_plan_mapping {
        let blog_id = id.clone().unwrap();
        info!(
            "Adding Blog Tag Mapping for Blog ID {} and Tag ID {}",
            &blog_id, &add_tag_id
        );
        let add_map_result = blog_tag_mapping_uc
            .clone()
            .unwrap()
            .blog_tag_mapping_repo
            .add(blog_id, add_tag_id)
            .await;

        match add_map_result {
            Some(btm_command_status) => {
                if btm_command_status != BlogTagMappingCommandStatus::Stored {
                    error!(
                        "Failed to add Blog Tag Mapping for Blog ID {} and Tag ID {}",
                        &blog_id, &add_tag_id
                    );
                    return get_500_internal_server_error();
                }
            }
            None => {
                info!("Failed to add Blog Tag Mapping");
                return get_404_not_found().await;
            }
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

    let mut blog_uc = app_state.blog_db_usecase.lock().await.clone();
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

    let delete_result = blog_uc.blog_repo.delete(id.clone().unwrap()).await;

    match delete_result {
        Some(blog_command_status) => {
            if blog_command_status != BlogCommandStatus::Deleted {
                error!("Failed to delete Blog with Id {}", &path);
                return get_500_internal_server_error();
            }
        }
        None => {
            info!("Failed to delete Blog with Id {}.", &path);
            return get_404_not_found().await;
        }
    }

    let blog_tag_mapping_uc = app_state.blog_tag_mapping_db_usecase.lock().await.clone();
    if blog_tag_mapping_uc.is_none() {
        error!("Failed to lock blog tag mapping usecase mutex");
        return get_500_internal_server_error();
    }

    let deleted_mappings = blog_tag_mapping_uc
        .clone()
        .unwrap()
        .blog_tag_mapping_repo
        .delete_by_blog_id(id.clone().unwrap())
        .await;
    if deleted_mappings.is_none() {
        error!(
            "Failed to delete blog tag mappings for blog id {}",
            id.unwrap().clone(),
        );
        return get_500_internal_server_error();
    }
    if deleted_mappings.unwrap() != BlogTagMappingCommandStatus::Deleted {
        error!(
            "Failed to delete blog tag mapping for blog id {}. Command Status is not Deleted",
            id.unwrap().clone(),
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
