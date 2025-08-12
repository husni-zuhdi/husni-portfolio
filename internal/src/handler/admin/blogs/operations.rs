use crate::handler::admin::blogs::displays::get_admin_blogs;
use crate::handler::status::{get_404_not_found, get_500_internal_server_error};
use crate::model::axum::AppState;
use crate::model::blog_tag_mappings::BlogTagMappingCommandStatus;
use crate::model::blogs::{Blog, BlogCommandStatus, BlogId, BlogsParams};
use crate::model::tags::Tag;
use crate::utils::remove_whitespace;
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::response::Html;
use tracing::{debug, error, info, warn};
use urlencoding::decode;

// Take request body String from PUT and POST operations to create a new blog
fn process_blog_body(body: String) -> Blog {
    // Initialize fields
    let mut blog_id = 0_i64;
    let mut blog_name = String::new();
    let mut blog_body = String::new();
    let mut blog_tags = vec![String::new()];

    let req_fields: Vec<&str> = body.split("&").collect();
    for req_field in req_fields {
        let (key, value) = req_field.split_once("=").unwrap();
        let value_decoded = decode(value).unwrap();
        debug!("Request field key/value {:?}/{:?}", key, value_decoded);
        match key {
            "blog_id" => {
                blog_id = value_decoded
                    .parse::<i64>()
                    .expect("Failed to parse path from request body")
            }
            "blog_name" => blog_name = value_decoded.to_string(),
            "blog_body" => blog_body = value_decoded.to_string(),
            "blog_tag" => {
                let clean_tag = remove_whitespace(&value_decoded);
                blog_tags.push(clean_tag);
            }
            _ => {
                warn!("Unrecognized key/value: {:?}/{:?}", key, value_decoded);
                continue;
            }
        }
    }

    Blog {
        id: BlogId { id: blog_id },
        name: Some(blog_name),
        body: Some(blog_body),
        tags: Some(blog_tags),
        source: None,
        filename: None,
    }
}

/// post_add_admin_blog
/// Serve POST add blog endpoint
#[debug_handler]
pub async fn post_add_admin_blog(State(app_state): State<AppState>, body: String) -> Html<String> {
    // Locking Mutex
    // TODO: Implement check and add on tags and blog_tag_mappings
    let mut blog_uc = app_state.blog_usecase.lock().await.clone();

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
    let tag_uc = app_state.tag_usecase.lock().await.clone();
    if tag_uc.is_none() {
        error!("Failed to lock tag usecase mutex");
        return get_500_internal_server_error();
    }

    let available_tags = tag_uc.unwrap().tag_repo.find_all().await;

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
    let blog_tag_mapping_uc = app_state.blog_tag_mapping_usecase.lock().await.clone();
    if blog_tag_mapping_uc.is_none() {
        error!("Failed to lock blog tag mapping usecase mutex");
        return get_500_internal_server_error();
    }

    for tag in selected_tags {
        let added_mapping = blog_tag_mapping_uc
            .clone()
            .unwrap()
            .blog_tag_mapping_repo
            .add(blog.id.id, tag.id)
            .await;
        if added_mapping.is_none() {
            error!(
                "Failed to add blog tag mapping for blog id {} and tag id {}",
                blog.id.id.clone(),
                tag.id.clone()
            );
            return get_500_internal_server_error();
        }
        if added_mapping.unwrap() != BlogTagMappingCommandStatus::Stored {
            error!("Failed to add blog tag mapping for blog id {} and tag id {}. Command Status is not Stored", blog.id.id.clone(), tag.id.clone());
            return get_500_internal_server_error();
        }
    }

    let query_params = BlogsParams {
        start: None,
        end: None,
        tags: None,
    };

    get_admin_blogs(State(app_state), Query(query_params)).await
}

/// delete_delete_admin_blog
/// Serve DELETE delete blog HTML file
#[debug_handler]
pub async fn delete_delete_admin_blog(
    Path(path): Path<String>,
    State(app_state): State<AppState>,
) -> Html<String> {
    let mut data = app_state.blog_usecase.lock().await.clone();
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

    let delete_result = data
        .blog_repo
        .delete(BlogId {
            id: id.clone().unwrap(),
        })
        .await;

    match delete_result {
        Some(talk_command_status) => {
            if talk_command_status != BlogCommandStatus::Deleted {
                error!("Failed to delete Talk with Id {}", &path);
                return get_500_internal_server_error();
            }
        }
        None => {
            info!("Failed to edit Talk with Id {}.", &path);
            return get_404_not_found().await;
        }
    }

    let blog_tag_mapping_uc = app_state.blog_tag_mapping_usecase.lock().await.clone();
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

    get_admin_blogs(State(app_state), Query(query_params)).await
}
