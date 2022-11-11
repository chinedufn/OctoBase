use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use super::workspace::{get_workspace_info, Workspace};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub email: String,
    pub avatar_url: String,
    pub workspaces: Vec<Workspace>,
}

async fn get_user_info() -> UserInfo {
    let w1 = get_workspace_info().await;
    let w2 = get_workspace_info().await;

    UserInfo {
        id: "asdasd".into(),
        name: "Tom".into(),
        email: "tom@affine.com".into(),
        avatar_url: "https://tom.affine.pro/avatar.png".into(),
        workspaces: vec![w1, w2],
    }
}

// POST /apis/config/get_user_info
pub async fn user_info() -> Response {
    let user_info = get_user_info().await;

    (StatusCode::FOUND, Json(user_info)).into_response()
}
