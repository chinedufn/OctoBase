use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct FailResponse {
    pub code: usize,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailResponseError {
    pub message: String,
    pub domain: String,
    pub reason: String,
}

#[derive(Debug)]
pub enum FirebaseError {
    Unauthorized,
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
