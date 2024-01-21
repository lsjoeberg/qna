use std::collections::HashMap;

use tracing::{event, info, instrument, Level};
use warp::http::StatusCode;
use warp::{Rejection, Reply};

use crate::store::Store;
use crate::types::pagination::{extract_pagination, Pagination};
use crate::types::question::{NewQuestion, Question};

#[instrument]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl Reply, Rejection> {
    let mut pagination = Pagination::default();
    if !params.is_empty() {
        event!(Level::INFO, pagination = true);
        pagination = extract_pagination(params)?;
    }

    info!(pagination = false);
    match store
        .get_questions(pagination.limit, pagination.offset)
        .await
    {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(err) => return Err(warp::reject::custom(err)),
    }
}

pub async fn add_question(
    store: Store,
    new_question: NewQuestion,
) -> Result<impl Reply, Rejection> {
    if let Err(err) = store.add_question(new_question).await {
        return Err(warp::reject::custom(err));
    }
    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}

pub async fn update_question(
    id: i32,
    store: Store,
    question: Question,
) -> Result<impl Reply, Rejection> {
    let res = match store.update_question(question, id).await {
        Ok(res) => res,
        Err(err) => return Err(warp::reject::custom(err)),
    };
    Ok(warp::reply::json(&res))
}

pub async fn delete_question(id: i32, store: Store) -> Result<impl Reply, Rejection> {
    if let Err(err) = store.delete_question(id).await {
        return Err(warp::reject::custom(err));
    }
    Ok(warp::reply::with_status(
        format!("Question: {} deleted", id),
        StatusCode::OK,
    ))
}
