mod error;
mod user;
mod workspace;

use axum::{
    extract::{FromRequest, Path, RequestParts},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
// use error::{FailResponse, FailResponseError};
use http::{response, uri::Authority, StatusCode};
use reqwest;
use serde::{Deserialize, Serialize};

const FIREBASE_API_KEY: &'static str = "AIzaSyAezKJuZZNcR7XUR9Cm9K7GRMj90DrquQM";
const SECRET: &[u8] = b"affine-pathfinder";

#[inline]
fn firebase_auth_url(v: &str, v2: &str) -> String {
    format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:{}?key={}",
        v, v2
    )
}

pub fn auth_apis(router: Router) -> Router {
    let api_handler = Router::new()
        .route("/gym", get(axum_gym))
        .route("/config/sign_in", post(sign_in_email))
        .route("/config/get_user_info", post(user::user_info))
        .route(
            "/config/get_workspace_info/:collection/:document",
            post(workspace::workspace_info),
        );

    router.nest("/api", api_handler)
}

async fn axum_gym() -> Response {
    (StatusCode::FOUND).into_response()
}

#[derive(Debug, Deserialize, Serialize)]
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

// POST /api/signin
async fn sign_in_email(Json(login): Json<LoginRequest>) -> Response {
    let mut url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
        FIREBASE_API_KEY
    );

    let login_info = LoginRequest {
        email: login.email,
        password: login.password,
    };

    println!("{:?}", login_info);

    let client = reqwest::Client::new();

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&login_info)
        .send()
        .await
        .unwrap();

    if resp.status() != 200 {
        let res = resp.text().await.unwrap();
        (StatusCode::NOT_FOUND, res).into_response()
    } else {
        let res = resp.text().await.unwrap();
        let res = serde_json::from_str::<LoginResponse>(&res).unwrap();
        (StatusCode::OK, Json(res)).into_response()
    }
}
