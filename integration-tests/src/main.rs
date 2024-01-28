use qna::{config, handle_errors, oneshot, setup_store};

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    dotenv::dotenv().ok();
    let config = config::Config::new().expect("Config can't be set");
    let store = setup_store(&config).await?;
    let handler = oneshot(store).await;
    // register_user();
    // login_user();
    // post_question();
    let _ = handler.sender.send(1);
    Ok(())
}
