use serde_json::json;
use worker::{Error as WorkerError, Response};
use worker::kv::KvError;

use crate::jwt::JwtError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    // users
    #[error("user not exists")]
    UserNotExists,
    #[error("user email duplicated")]
    UserEmailDuplicated,

    // auth
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid oauth provider")]
    InvalidOAuthProvider,
    #[error("invalid oauth token")]
    InvalidOAuthToken,

    // challenges
    #[error("challenge not exists")]
    ChallengeNotExists,

    // foodnotes
    #[error("foodnote not exists")]
    FoodnoteNotExists,

    // general
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("server error: {0}")]
    ServerError(String),

    // internals
    #[error("kv error")]
    KvError {
        #[from]
        source: KvError,
    },
    #[error("worker error")]
    WorkerError {
        #[from]
        source: WorkerError,
    },
    #[error("jwt error")]
    JwtError {
        #[from]
        source: JwtError,
    },
}

impl ApiError {
    pub fn to_response(&self) -> Response {
        let message = match self {
            ApiError::UserNotExists => "user not exists",
            ApiError::UserEmailDuplicated => "user email duplicated",
            ApiError::Unauthorized => "unauthorized",
            ApiError::InvalidOAuthProvider => "invalid oauth provider",
            ApiError::InvalidOAuthToken => "invalid oauth token",
            ApiError::ChallengeNotExists => "challenge not exists",
            ApiError::FoodnoteNotExists => "foodnote not exists",
            ApiError::BadRequest(message) => message,
            ApiError::ServerError(message) => message,
            _ => "internal server error",
        };
        let status_code: u16 = match self {
            ApiError::UserNotExists => 404,
            ApiError::UserEmailDuplicated => 406,
            ApiError::Unauthorized => 401,
            ApiError::InvalidOAuthProvider => 400,
            ApiError::InvalidOAuthToken => 400,
            ApiError::ChallengeNotExists => 404,
            ApiError::FoodnoteNotExists => 404,
            ApiError::BadRequest(_) => 400,
            _ => 500,
        };

        let body = json!({ "message": message });

        Response::from_json(&body).unwrap().with_status(status_code)
    }
}
