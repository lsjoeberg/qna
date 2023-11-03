use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::sync::Arc;
use tokio::sync::RwLock;

use warp::reject::Reject;
use warp::{
    filters::cors::CorsForbidden, http::Method, http::StatusCode, Filter, Rejection, Reply,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
struct QuestionId(String);

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("failed to read questions.json")
    }
}

#[derive(Debug)]
enum QueryError {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    InvalidRange,
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            QueryError::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            QueryError::MissingParameters => write!(f, "Missing parameter"),
            QueryError::InvalidRange => write!(f, "Invalid range"),
        }
    }
}

impl Reject for QueryError {}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, QueryError> {
    if params.contains_key("start") && params.contains_key("end") {
        return Ok(Pagination {
            start: params
                .get("start")
                .unwrap()
                .parse::<usize>()
                .map_err(QueryError::ParseError)?,
            end: params
                .get("end")
                .unwrap()
                .parse::<usize>()
                .map_err(QueryError::ParseError)?,
        });
    }
    Err(QueryError::MissingParameters)
}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl Reply, Rejection> {
    let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
    if !params.is_empty() {
        let mut pagination = extract_pagination(params)?;
        let nq = store.questions.read().await.len();
        // Validate and sanitize parameters.
        if pagination.start > pagination.end || pagination.start > nq - 1 {
            return Err(QueryError::InvalidRange.into());
        }
        pagination.end = min(pagination.end, nq);

        let res = &res[pagination.start..pagination.end];
        Ok(warp::reply::json(&res))
    } else {
        Ok(warp::reply::json(&res))
    }
}

async fn add_question(store: Store, question: Question) -> Result<impl Reply, Rejection> {
    store
        .questions
        .write()
        .await
        .insert(question.id.clone(), question);
    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<QueryError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let routes = get_questions
        .or(add_question)
        .with(cors)
        .recover(return_error);

    warp::serve(routes).run(([0, 0, 0, 0], 7878)).await;
}
