use std::collections::HashMap;

use warp::http::StatusCode;
use warp::{Rejection, Reply};

use crate::store::Store;
use crate::types::answer::{Answer, AnswerId};
use crate::types::question::QuestionId;

pub async fn add_answer(
    store: Store,
    params: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    let answer = Answer {
        id: AnswerId(1),
        content: params.get("content").unwrap().to_string(),
        question_id: QuestionId(params.get("id").unwrap().parse::<i32>().unwrap()),
    };
    store
        .answers
        .write()
        .await
        .insert(answer.id.clone(), answer);
    Ok(warp::reply::with_status("Answer added", StatusCode::OK))
}
