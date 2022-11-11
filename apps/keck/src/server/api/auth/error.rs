use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// Unknow errors are rarely used and only used if no other error type matches
    #[error("unknow error")]
    Unknow(String),
    /// An error caused by the http library. This only happens if the http request is badly
    /// formatted (too big, invalid characters) or if the server did strange things
    /// (connection abort, ssl verification error).
    #[error("http request error")]
    Request(reqwest::Error),
    #[error("http status code is not 200, but no Google error response")]
    UnexpectedResponse(reqwest::StatusCode),
    #[error("An error returned by the Firebase API")]
    APIError(usize, String, String),
    #[error("{0} auth error")]
    SignIn(String),
    #[error("firebase api {0} error")]
    API(String),
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::API(err.to_string())
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct FailResponse {
//     pub code: usize,
//     pub message: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct FailResponseError {
//     pub message: String,
//     pub domain: String,
//     pub reason: String,
// }

// #[derive(Debug)]
// pub enum FirebaseError {
//     Unauthorized,
//     InvaildWorkspace,
//     Internal,
// }

// impl IntoResponse for FirebaseError {
//     fn into_response(self) -> Response {
//         let (code, msg) = match self {
//             FirebaseError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
//             FirebaseError::InvaildWorkspace => (StatusCode::NO_CONTENT, "InvaildWorkspace"),
//             FirebaseError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal"),
//         };
//         (code, msg).into_response()
//     }
// }
