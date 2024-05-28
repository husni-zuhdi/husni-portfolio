use crate::config::Config;
use crate::router::*;
use crate::utils::create_blogs;
use actix_web::{middleware, web, App, HttpServer};

pub async fn handler(cfg: &Config) -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(cfg.log_level.clone()));

    let endpoint = cfg.svc_endpoint.as_str();
    let port = cfg
        .svc_port
        .parse::<u16>()
        .expect("Failed to get port number");
    let blogs_data = create_blogs().await.expect("Failed to get blogs data");

    log::info!(
        "Starting HTTP Server at http://{}:{}",
        cfg.svc_endpoint,
        cfg.svc_port
    );

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(blogs_data.clone()))
            .service(web::resource("/").route(web::get().to(profile)))
            .service(web::resource("/styles.css").route(web::get().to(styles)))
            .service(web::resource("/blogs").route(web::get().to(blogs)))
            .service(web::resource("/blogs/{blogid}").route(web::get().to(get_blog)))
    })
    .bind((endpoint, port))
    .expect("Failed to start Http Server")
    .run()
    .await
}
