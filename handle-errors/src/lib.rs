use std::fmt::Formatter;

use argon2::Error as ArgonError;
use reqwest::Error as ReqwestError;
use reqwest_middleware::Error as MiddlewareReqwestError;
use tracing::{event, instrument, Level};
use warp::{
    body::BodyDeserializeError, cors::CorsForbidden, http::StatusCode, reject::Reject, Rejection,
    Reply,
};

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    WrongPassword,
    CannotDecryptToken,
    Unauthorized,
    ArgonLibraryError(ArgonError),
    InvalidRange,
    DataBaseQueryError(sqlx::Error),
    ReqwestAPIError(ReqwestError),
    MiddlewareReqwestAPIError(MiddlewareReqwestError),
    ClientError(APILayerError),
    ServerError(APILayerError),
}

#[derive(Debug, Clone)]
pub struct APILayerError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for APILayerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Status: {}, Message: {}", self.status, self.message)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Error::MissingParameters => write!(f, "Missing parameter"),
            Error::WrongPassword => write!(f, "Wrong password"),
            Error::CannotDecryptToken => write!(f, "Cannot decrypt token"),
            Error::Unauthorized => write!(f, "Unauthorized to change the resource"),
            Error::ArgonLibraryError(_) => write!(f, "Cannot verify password"),
            Error::InvalidRange => write!(f, "Invalid range"),
            Error::DataBaseQueryError(_) => {
                write!(f, "Cannot update, invalid data.")
            }
            Error::ReqwestAPIError(ref err) => {
                write!(f, "External API error: {}", err)
            }
            Error::MiddlewareReqwestAPIError(ref err) => {
                write!(f, "External API error: {}", err)
            }
            Error::ClientError(ref err) => {
                write!(f, "External Client error: {}", err)
            }
            Error::ServerError(ref err) => {
                write!(f, "External Server error: {}", err)
            }
        }
    }
}

impl Reject for Error {}
impl Reject for APILayerError {}

const DUPLICATE_KEY: u32 = 23505;

#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(crate::Error::DataBaseQueryError(err)) = r.find() {
        event!(Level::ERROR, "Database query error");
        match err {
            sqlx::Error::Database(err) => {
                if err.code().unwrap().parse::<u32>().unwrap() == DUPLICATE_KEY {
                    Ok(warp::reply::with_status(
                        "Account already exists".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                } else {
                    Ok(warp::reply::with_status(
                        "Cannot update data".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                }
            }
            _ => Ok(warp::reply::with_status(
                "Cannot update data".to_string(),
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
        }
    } else if let Some(crate::Error::ReqwestAPIError(err)) = r.find() {
        event!(Level::ERROR, "{err}");
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::Unauthorized) = r.find() {
        event!(Level::ERROR, "Not matching account id");
        Ok(warp::reply::with_status(
            "Unauthorized to change the resource".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(crate::Error::WrongPassword) = r.find() {
        event!(Level::ERROR, "Entered wrong password");
        Ok(warp::reply::with_status(
            "Wrong E-mail/Password combination".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(crate::Error::MiddlewareReqwestAPIError(err)) = r.find() {
        event!(Level::ERROR, "{err}");
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::ClientError(err)) = r.find() {
        event!(Level::ERROR, "{err}");
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::ServerError(err)) = r.find() {
        event!(Level::ERROR, "{err}");
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
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
