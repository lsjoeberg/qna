use qna::{config, run, setup_store};

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    // Load vars from .env and then parse configuration.
    dotenv::dotenv().ok();

    let cfg = config::Config::new().expect("Failed to read configuration");
    let store = setup_store(&cfg).await?;

    tracing::info!("Q&A service build ID {}", env!("RUST_WEB_DEV_VERSION"));

    run(cfg, store).await;

    Ok(())
}
