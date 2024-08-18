use internal::{self, config::Config, handler::handler};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_envar();
    handler(config).await;
    Ok(())
}
