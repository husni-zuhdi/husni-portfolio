use internal::{self, config::Config, handler::handler};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_envar();
    handler(config).await
}
