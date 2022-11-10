mod error;

use anyhow::Ok;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    response::IntoResponse,
    routing::post,
    Error, Json, Router,
};
use error::FailResponseBody;
use http::StatusCode;
use jsonwebtoken as jwt;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use self::error::FirebaseError;

const FIREBASE_API_KEY: &'static str = "AIzaSyAezKJuZZNcR7XUR9Cm9K7GRMj90DrquQM";
const SECRET: &[u8] = b"affine-pathfinder";

#[inline]
fn firebase_auth_url(v: &str, v2: &str) -> String {
    format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:{}?key={}",
        v, v2
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
    pub avatar_url: String,
    pub workspaces: Vec<Workspace>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub workspace_type: WorkspaceType,
    pub avater_url: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkspaceType {
    personal,
    team,
}

pub fn auth_apis(router: Router) -> Router {
    let api_handler = Router::new()
        .route("/sign_in", post(sign_in_email))
        .route("/config/get_user_info", post(get_user_info))
        .route("/config/get_user_info1", post(get_user_info1))
        .route("/config/get_workspace_info", post(get_user_workspace_info))
        .route("/update_workspace", post(update_workspace));

    router.nest("/api", api_handler)
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    kind: String,
    #[serde(rename = "localId")]
    user_id: String, // localId, for identification unique user
    email: String,
    #[serde(default = "default_display_name")]
    display_name: String,
    #[serde(rename = "idToken")]
    id_token: String, // idToken, for authentication
    registered: bool,
}

fn default_display_name() -> String {
    String::from("")
}

impl IntoResponse for LoginResponse {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap();
        (StatusCode::OK, body).into_response()
    }
}

impl std::fmt::Display for LoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "email: {}, password: {}", self.email, self.password)
    }
}

// POST /api/signin
async fn sign_in_email(Json(login): Json<LoginRequest>) -> Json<LoginResponse> {
    let mut url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
        FIREBASE_API_KEY
    );

    let login_info = LoginRequest {
        email: login.email,
        password: login.password,
    };

    let client = reqwest::Client::new();

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&login_info)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let login_response: LoginResponse = serde_json::from_str(&resp).unwrap();
    Json(login_response)
}

// need accessid
// response payload
// id: string
// name: string
// email: string
// avatar_url: string (User's avatar )
// workspace
// POST /api/get_user_info
async fn get_user_info1() -> Json<UserInfo> {
    let api_key = "AIzaSyAezKJuZZNcR7XUR9Cm9K7GRMj90DrquQM";
    let url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={}",
        api_key
    );

    let mut map = HashMap::new();
    map.insert("idToken", "e");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&map)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let Json(workspace) = get_user_workspace_info().await;

    Json(UserInfo {
        name: "Tom".to_string(),
        email: "tom@affine.com".to_string(),
        avatar_url: "https://tom.affine.pro/avatar.png".to_string(),
        workspaces: workspace,
    })
}

async fn get_user_workspace_info() -> Json<Vec<Workspace>> {
    Json(vec![
        Workspace {
            name: "tom's workspace".to_string(),
            workspace_type: WorkspaceType::personal,
            avater_url: "https://workspace.affine.pro/avatar.png".to_string(),
            permissions: vec!["write".to_string()],
        },
        Workspace {
            name: "affine workspace".to_string(),
            workspace_type: WorkspaceType::team,
            avater_url: "https://workspace.affine.pro/avatar.png".to_string(),
            permissions: vec!["read".to_string()],
        },
    ])
}

// POST /api/get_user_info
// aggrate base info adn workspace info
async fn get_user_info() -> impl IntoResponse {
    let userinfo = UserInfo {
        name: "Tom".to_string(),
        email: "tom@affine.com".to_string(),
        avatar_url: "https://tom.affine.pro/avatar.png".to_string(),
        workspaces: vec![
            Workspace {
                name: "tom's workspace".to_string(),
                workspace_type: WorkspaceType::personal,
                avater_url: "https://workspace.affine.pro/avatar.png".to_string(),
                permissions: vec!["write".to_string()],
            },
            Workspace {
                name: "affine workspace".to_string(),
                workspace_type: WorkspaceType::team,
                avater_url: "https://workspace.affine.pro/avatar.png".to_string(),
                permissions: vec!["read".to_string()],
            },
        ],
    };

    println!("{:?}", userinfo);

    (StatusCode::FOUND, Json(userinfo))
}

// request payload
// workspace_id: string
// name: Option(string)
// avatar_url: Option(string)
// public_read: Option(bool) If true, the workspace can be viewed by anyone who gets the link.
async fn update_workspace() -> impl IntoResponse {
    let workspace = Workspace {
        name: "affine workspace".to_string(),
        workspace_type: WorkspaceType::team,
        avater_url: "https://workspace.affine.pro/avatar.png".to_string(),
        permissions: vec!["read".to_string()],
    };

    println!("{:?}", workspace);

    (StatusCode::FOUND, Json(workspace))
}
