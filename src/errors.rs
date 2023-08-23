use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::StatusCode,
    reject::Reject,
    Rejection, Reply,
};
use tracing::instrument;
use serde_json::json;
use std::collections::HashMap;
use std::convert::Infallible;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("init database: {0}")]
    InitDatabase(#[source]  sqlx::Error),

    #[error("parse config: {0}")]
    ConfigParse(#[source] std::num::ParseIntError),

    #[error("query database: {0}")]
    DatabaseQuery(#[from] sqlx::Error),

    #[error("record not found")]
    RecordNotFound,

    #[error("edit conflict")]
    EditConflict,

    #[error("input validation failed")]
    Validation(HashMap<&'static str, &'static str>),
    
    #[error("you must be authenticated to access this resource")]
    AuthenticationRequired,

    #[error("your user account must be activated to access this resource")]
    InactiveAccount,

    #[error("invalid or missing authentication token")]
    InvalidAuthenticationToken,

    #[error("your user account doesn't have the necessary permissions to access this resource")]
    Unauthorized,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("template render {0}")]
    Render(#[from] askama::Error),

    #[error("duplicate email")]
    DuplicateEmail,

    #[error("other kind unexpected error {0}")]
    UnexpectedError(#[from] anyhow::Error),
}


impl Reject for Error {}


#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(my) = r.find::<Error>() {
        tracing::error!("my custom: {}", my);
        let mut status = StatusCode::INTERNAL_SERVER_ERROR;
        let r = "the server encountered a problem and could not process your request";
        let mut msg = json!({"error": r}).to_string(); 

        match my {
            Error::Validation(e) => {
                status = StatusCode::UNPROCESSABLE_ENTITY;
                msg = json!({"error":  e}).to_string();
            }
            Error::RecordNotFound => {
                status = StatusCode::NOT_FOUND;
                msg = json!({"error": "the requested resource could not be found"}).to_string();
            }
            Error::EditConflict => {
                status = StatusCode::CONFLICT;
                let r = "unable to update the record due to an edit conflict, please try again";
                msg = json!({"error": r}).to_string();
            }
            Error::InvalidCredentials | Error::InvalidAuthenticationToken | Error::AuthenticationRequired =>  {
                status = StatusCode::UNAUTHORIZED;
                msg = json!({"error": my.to_string()}).to_string();
            }
            Error::InactiveAccount | Error::Unauthorized   =>  {
                status = StatusCode::FORBIDDEN;
                msg = json!({"error": my.to_string()}).to_string();
            }
            _ =>  {
                
            },
        }
        Ok(warp::reply::with_status(
            msg,
            status,
        )) 
    } else if let Some(error) = r.find::<CorsForbidden>() {
        tracing::error!("CORS forbidden error: {}", error);
        Ok(warp::reply::with_status(
            json!({"error":  error.to_string()}).to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        tracing::error!("Cannot deserizalize request body: {}", error);
        Ok(warp::reply::with_status(
            json!({"error":  error.to_string()}).to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else {
        tracing::warn!("Requested route was not found");
        Ok(warp::reply::with_status(
             json!({"error": "the requested resource could not be found"}).to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}


