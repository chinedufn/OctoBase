use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub workspace_type: WorkspaceType,
    pub avater_url: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WorkspaceType {
    personal,
    team,
}

pub async fn get_workspace_info() -> Workspace {
    Workspace {
        id: "fasdf".into(),
        name: "affine workspace".into(),
        workspace_type: WorkspaceType::team,
        avater_url: "https://workspace.affine.pro/avatar.png".into(),
        permissions: vec!["read".into()],
    }
}

// POST /apis/config/get_workspace_info
pub async fn workspace_info() -> Response {
    let workspace = get_workspace_info().await;

    (StatusCode::FOUND, Json(workspace)).into_response()
}
