use warp::http::StatusCode;
use warp::{Rejection, Reply};

use crate::account::Session;
use crate::profanity::check_profanity;
use crate::store::Store;
use crate::types::answer::NewAnswer;

pub async fn add_answer(
    session: Session,
    store: Store,
    new_answer: NewAnswer,
) -> Result<impl Reply, Rejection> {
    let account_id = session.account_id;
    let content = match check_profanity(new_answer.content).await {
        Ok(res) => res,
        Err(err) => return Err(warp::reject::custom(err)),
    };
    let answer = NewAnswer {
        content,
        question_id: new_answer.question_id,
    };

    match store.add_answer(answer, account_id).await {
        Ok(_) => Ok(warp::reply::with_status("Answer added", StatusCode::OK)),
        Err(err) => Err(warp::reject::custom(err)),
    }
}
