use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use http::StatusCode;
use jsonwebtoken as jwt;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const FIREBASE_API_KEY: &'static str = "AIzaSyAezKJuZZNcR7XUR9Cm9K7GRMj90DrquQM";
const SECRET: &[u8] = b"affine-pathfinder";

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
    pub space_type: String,
    pub avater_url: String,
    pub permissions: Vec<String>,
}

enum WorkspaceType {
    personal,
    team,
}

pub fn auth_apis(router: Router) -> Router {
    let api_handler = Router::new()
        .route("/sign_in", post(sign_in_email))
        .route("/get_user_info", post(get_user_info))
        .route("/update_workspace", post(update_workspace));

    router.nest("/api", api_handler)
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub struct LoginResponse {
    token: String,
}

impl std::fmt::Display for LoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "email: {}, password: {}", self.email, self.password)
    }
}

// POST /api/signin
async fn sign_in_email(Json(login): Json<LoginRequest>) -> impl IntoResponse {
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

    // expire

    (StatusCode::FOUND, Json(resp))
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

    Json(UserInfo {
        name: "Tom".to_string(),
        email: "tom@affine.com".to_string(),
        avatar_url: "https://tom.affine.pro/avatar.png".to_string(),
        workspaces: vec![
            Workspace {
                name: "tom's workspace".to_string(),
                space_type: "personal".to_string(),
                avater_url: "https://workspace.affine.pro/avatar.png".to_string(),
                permissions: vec!["write".to_string()],
            },
            Workspace {
                name: "affine workspace".to_string(),
                space_type: "team".to_string(),
                avater_url: "https://workspace.affine.pro/avatar.png".to_string(),
                permissions: vec!["read".to_string()],
            },
        ],
    })
}

async fn get_user_base_info() -> impl IntoResponse {}

async fn get_user_workspace_info() -> impl IntoResponse {}

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
                space_type: "personal".to_string(),
                avater_url: "https://workspace.affine.pro/avatar.png".to_string(),
                permissions: vec!["write".to_string()],
            },
            Workspace {
                name: "affine workspace".to_string(),
                space_type: "team".to_string(),
                avater_url: "https://workspace.affine.pro/avatar.png".to_string(),
                permissions: vec!["read".to_string()],
            },
        ],
    };

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
        space_type: "team".to_string(),
        avater_url: "https://workspace.affine.pro/avatar.png".to_string(),
        permissions: vec!["read".to_string()],
    };

    println!("{:?}", workspace);

    (StatusCode::FOUND, Json(workspace))
}

#[derive(Debug)]
pub enum AuthError {
    Auth,
    Internal,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (code, msg) = match self {
            AuthError::Auth => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        };
        (code, msg).into_response()
    }
}
