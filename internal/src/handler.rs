use crate::config::Config;
use crate::router::*;
use actix_web::{middleware, web, App, HttpServer};

pub async fn handler(cfg: &Config) -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!(
        "Starting HTTP Server at http://{}:{}",
        cfg.svc_endpoint,
        cfg.svc_port
    );
    let endpoint = cfg.svc_endpoint.as_str();
    let port = cfg
        .svc_port
        .parse::<u16>()
        .expect("Failed to get port number");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
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
