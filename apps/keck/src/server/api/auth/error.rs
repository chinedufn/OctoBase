use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct FailResponseBody {
    code: usize,
    message: String,
    errors: Vec<FailResponseError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FailResponseError {
    message: String,
    domain: String,
    reason: String,
}

#[derive(Debug)]
pub enum FirebaseError {
    Unauthorized(),
    InvaildWorkspace,
    Internal,
}

impl IntoResponse for FirebaseError {
    fn into_response(self) -> Response {
        let (code, msg) = match self {
            FirebaseError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            FirebaseError::InvaildWorkspace => (StatusCode::NO_CONTENT, "InvaildWorkspace"),
            FirebaseError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal"),
        };
        (code, msg).into_response()
    }
}
