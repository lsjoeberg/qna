use std::fmt::Formatter;

use tracing::{event, instrument, Level};
use warp::{
    body::BodyDeserializeError, cors::CorsForbidden, http::StatusCode, reject::Reject, Rejection,
    Reply,
};

#[derive(Debug)]
pub enum QueryError {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    InvalidRange,
    DataBaseQueryError,
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            QueryError::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            QueryError::MissingParameters => write!(f, "Missing parameter"),
            QueryError::InvalidRange => write!(f, "Invalid range"),
            QueryError::DataBaseQueryError => {
                write!(f, "Cannot update, invalid data.")
            }
        }
    }
}

impl Reject for QueryError {}

#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(crate::QueryError::DataBaseQueryError) = r.find() {
        event!(Level::ERROR, "Database query error");
        Ok(warp::reply::with_status(
            crate::QueryError::DataBaseQueryError.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        event!(Level::ERROR, "Cannot deserialize request body: {}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        event!(Level::ERROR, "CORS forbidded error {}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else {
        event!(Level::WARN, "Request route was not found");
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}
