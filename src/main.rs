#![warn(clippy::all)]

use handle_errors::return_error;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{http::Method, Filter};

mod routes;
mod store;
mod types;

#[tokio::main]
async fn main() {
    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "qna=info,warp=error".to_owned());

    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        eprintln!("env var DATABASE_URL not set");
        std::process::exit(1);
    };
    let store = store::Store::new(&database_url).await;
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        // Record and event when the span closes.
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(routes::question::get_questions)
        .with(warp::trace(|info| {
            tracing::info_span!("get_questions request", method = %info.method(), path = %info.path(), id = %uuid::Uuid::new_v4())
        }));

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(routes::question::delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(routes::answer::add_answer);

    let routes = get_questions
        .or(add_question)
        .or(add_answer)
        .or(update_question)
        .or(delete_question)
        .with(cors)
        .with(warp::trace::request())
        .recover(return_error);

    warp::serve(routes).run(([0, 0, 0, 0], 7878)).await;
}
