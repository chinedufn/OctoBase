use std::collections::HashMap;

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
    let collection = "workspaces";
    let document = "xb2UEiYNQEAtut5YP94U";

    let mut url = format!(
        "https://firestore.googleapis.com/v1/projects/pathfinder-52392/databases/(default)/documents/workspaces/xb2UEiYNQEAtut5YP94U"
    );

    println!("{}", url);

    let access_token = "eyJhbGciOiJSUzI1NiIsImtpZCI6ImY4MDljZmYxMTZlNWJhNzQwNzQ1YmZlZGE1OGUxNmU4MmYzZmQ4MDUiLCJ0eXAiOiJKV1QifQ.eyJpc3MiOiJodHRwczovL3NlY3VyZXRva2VuLmdvb2dsZS5jb20vcGF0aGZpbmRlci01MjM5MiIsImF1ZCI6InBhdGhmaW5kZXItNTIzOTIiLCJhdXRoX3RpbWUiOjE2Njg2NzgyODEsInVzZXJfaWQiOiJWNHhRS2tna3J4ZFBXa1NKblU2V0pJblRGQXoyIiwic3ViIjoiVjR4UUtrZ2tyeGRQV2tTSm5VNldKSW5URkF6MiIsImlhdCI6MTY2ODY3ODI4MSwiZXhwIjoxNjY4NjgxODgxLCJlbWFpbCI6Im1pcmFjbGV5aW5AbGl2ZS5jb20iLCJlbWFpbF92ZXJpZmllZCI6ZmFsc2UsImZpcmViYXNlIjp7ImlkZW50aXRpZXMiOnsiZW1haWwiOlsibWlyYWNsZXlpbkBsaXZlLmNvbSJdfSwic2lnbl9pbl9wcm92aWRlciI6InBhc3N3b3JkIn19.Mt0c_VxAJ8putadbwLm3kgac__2PysMj_uxS0fUhkI56B0i7il2tBCXFZJ8098Be4gMUuLJdX3jdKmliiB6BvxYZpm1d-AeDKlbQM06s3zzFyUiE-Hx6l1sNhlUuzx8hkEHF0B0FNtguHPoGZw8wyoafahlf1FIHS_ik0tjs8J0uR172GM6IWqHAK4CIoWGKSRMnnS4buddQn-EfGD_gY5VQsb_f7wjn8Nd6p-in1-_TgNlOYNBznMAVtIHGE_9maeOGIOZZpiDB016asQLeoNlhzOArziM-ZJJCo3I_DGBYL8FYnjnnmZFC1beFvH_y4EumjDu8rbi6XTOjOPgsag";

    let client = reqwest::Client::new();

    let resp = client
        .post(&url)
        .header("Authorization", "Bearer ".to_owned() + &access_token)
        .header("Content-Length", 0)
        // .header("Content-Type", "application/json")
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
