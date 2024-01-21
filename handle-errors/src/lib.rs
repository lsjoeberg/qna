use std::fmt::Formatter;

use warp::{
    body::BodyDeserializeError, cors::CorsForbidden, http::StatusCode, reject::Reject, Rejection,
    Reply,
};

use sqlx::error::Error as SqlxError;

#[derive(Debug)]
pub enum QueryError {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    InvalidRange,
    QuestionNotFound,
    DataBaseQueryError(SqlxError),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            QueryError::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            QueryError::MissingParameters => write!(f, "Missing parameter"),
            QueryError::InvalidRange => write!(f, "Invalid range"),
            QueryError::QuestionNotFound => write!(f, "Question not found"),
            QueryError::DataBaseQueryError(ref err) => {
                write!(f, "Query could not be executed: {}", err)
            }
        }
    }
}

impl Reject for QueryError {}

pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<QueryError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
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
