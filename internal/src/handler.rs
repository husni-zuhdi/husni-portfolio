use crate::config::Config;
use crate::model::data::BlogsData;
use crate::router::*;
use actix_web::{middleware, web, App, HttpServer};
use log::info;

pub async fn handler(cfg: Config) -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(cfg.log_level.clone()));

    let endpoint = cfg.svc_endpoint.as_str();
    let port = cfg
        .svc_port
        .parse::<u16>()
        .expect("Failed to get port number");

    let config = cfg.clone();
    let mut blogs_data = BlogsData::default();
    if !config.gh_owner.is_empty() && !config.gh_repo.is_empty() && !config.gh_branch.is_empty() {
        blogs_data = BlogsData::with_gh(&config.gh_owner, &config.gh_repo, &config.gh_branch).await;
    }

    info!(
        "Starting HTTP Server at http://{}:{}",
        cfg.svc_endpoint, cfg.svc_port
    );

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(blogs_data.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(web::resource("/").route(web::get().to(profile)))
            .service(web::resource("/statics/{static_file}").route(web::get().to(statics)))
            .service(web::resource("/blogs").route(web::get().to(get_blogs)))
            .service(web::resource("/blogs/{blogid}").route(web::get().to(get_blog)))
            .service(web::resource("/version").route(web::get().to(get_version)))
            .service(web::resource("/not-found").route(web::get().to(get_404_not_found)))
    })
    .bind((endpoint, port))
    .expect("Failed to start Http Server")
    .run()
    .await
}
