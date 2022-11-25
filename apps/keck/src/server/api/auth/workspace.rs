use std::collections::HashMap;

use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    Json,
};
use http::{HeaderMap, StatusCode};
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

// POST /apis/config/get_workspace_info/:collection/:document
pub async fn workspace_info(
    Path((collection, document)): Path<(String, String)>,
    header: HeaderMap,
) -> Response {
    // let collection = "workspaces";
    // let document = "xb2UEiYNQEAtut5YP94U";

    let mut url = format!(
        "https://firestore.googleapis.com/v1/projects/pathfinder-52392/databases/(default)/documents/{}/{}", collection, document
    );

    println!("{}", url);

    let access_token = header
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or("".to_string());

    let client = reqwest::Client::new();

    let resp = client
        .get(&url) // need terminal vpn
        .header("Authorization", "Bearer ".to_owned() + &access_token)
        .send()
        .await
        .unwrap();

    if resp.status() != 200 {
        let res = resp.text().await.unwrap();
        (StatusCode::NOT_FOUND, res).into_response()
    } else {
        let res = resp.text().await.unwrap();
        (StatusCode::OK, res).into_response()
    }
}
