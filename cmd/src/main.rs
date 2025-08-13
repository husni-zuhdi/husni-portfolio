use internal::app::app;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    app().await;
    Ok(())
}
