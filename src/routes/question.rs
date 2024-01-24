use std::collections::HashMap;

use tracing::{event, info, instrument, Level};
use warp::http::StatusCode;
use warp::{Rejection, Reply};

use crate::account::Session;
use crate::profanity::check_profanity;
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
    session: Session,
    store: Store,
    new_question: NewQuestion,
) -> Result<impl Reply, Rejection> {
    let account_id = session.account_id;
    let title = match check_profanity(new_question.title).await {
        Ok(res) => res,
        Err(err) => return Err(warp::reject::custom(err)),
    };
    let content = match check_profanity(new_question.content).await {
        Ok(res) => res,
        Err(err) => return Err(warp::reject::custom(err)),
    };
    let question = NewQuestion {
        title,
        content,
        tags: new_question.tags,
    };

    if let Err(err) = store.add_question(question, account_id).await {
        return Err(warp::reject::custom(err));
    }
    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}

pub async fn update_question(
    id: i32,
    session: Session,
    store: Store,
    question: Question,
) -> Result<impl Reply, Rejection> {
    let account_id = session.account_id;
    if !store.is_question_owner(id, &account_id).await? {
        return Err(warp::reject::custom(handle_errors::Error::Unauthorized));
    }
    let title = match check_profanity(question.title).await {
        Ok(res) => res,
        Err(err) => return Err(warp::reject::custom(err)),
    };
    let content = match check_profanity(question.content).await {
        Ok(res) => res,
        Err(err) => return Err(warp::reject::custom(err)),
    };
    let question = Question {
        id: question.id,
        title,
        content,
        tags: question.tags,
    };

    let res = match store.update_question(question, id, account_id).await {
        Ok(res) => res,
        Err(err) => return Err(warp::reject::custom(err)),
    };
    Ok(warp::reply::json(&res))
}

pub async fn delete_question(
    id: i32,
    session: Session,
    store: Store,
) -> Result<impl Reply, Rejection> {
    let account_id = session.account_id;
    if !store.is_question_owner(id, &account_id).await? {
        return Err(warp::reject::custom(handle_errors::Error::Unauthorized));
    }
    if let Err(err) = store.delete_question(id, account_id).await {
        return Err(warp::reject::custom(err));
    }
    Ok(warp::reply::with_status(
        format!("Question: {} deleted", id),
        StatusCode::OK,
    ))
}
