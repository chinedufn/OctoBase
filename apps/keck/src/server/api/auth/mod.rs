use axum::{
    routing::{get, post},
    Json, Router,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn auth_apis(router: Router) -> Router {
    let api_handler = Router::new()
        .route("/signin/:login", post(sign_in_email))
        .route("/userinfo", get(user_info));

    router.nest("/api", api_handler)
}

#[derive(Deserialize, Serialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

impl std::fmt::Display for LoginForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "email: {}, password: {}", self.email, self.password)
    }
}

// POST /api/signin
async fn sign_in_email(Json(login): Json<LoginForm>) -> String {
    let api_key = "AIzaSyAezKJuZZNcR7XUR9Cm9K7GRMj90DrquQM";
    let mut url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
        api_key
    );

    println!("login: {}", login);

    let mut map = HashMap::new();
    map.insert("email", "miracleyin@live.com");
    map.insert("password", "19980804");

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

    String::from(resp)
}

// POST /api/userinfo
async fn user_info() -> String {
    let api_key = "AIzaSyAezKJuZZNcR7XUR9Cm9K7GRMj90DrquQM";
    let mut url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={}",
        api_key
    );

    let mut map = HashMap::new();
    map.insert("idToken", "eyJhbGciOiJSUzI1NiIsImtpZCI6InRCME0yQSJ9.eyJpc3MiOiJodHRwczovL2lkZW50aXR5dG9vbGtpdC5nb29nbGUuY29tLyIsImF1ZCI6InBhdGhmaW5kZXItNTIzOTIiLCJpYXQiOjE2Njc5ODQxNzcsImV4cCI6MTY2OTE5Mzc3NywidXNlcl9pZCI6IlY0eFFLa2drcnhkUFdrU0puVTZXSkluVEZBejIiLCJlbWFpbCI6Im1pcmFjbGV5aW5AbGl2ZS5jb20iLCJzaWduX2luX3Byb3ZpZGVyIjoicGFzc3dvcmQiLCJ2ZXJpZmllZCI6ZmFsc2V9.MYWd7LsOZ_2675iRj7oMbJXN2iO5R7rseuKWmXWgWw6wJqW4d2W-eR6Jh8xuQLlM3hNsFF-o-Kq7POtLdCTicSGOXjtIU388cu2HA-GG0Fx0QN_97HOB44aTply37il10al991pMBM6G_xOM_akH0OIl7p65SyzzYN0p0hYIpDiypvbruww5feqq-oRPpvA7lar332WNapFXyS09E_Ey6PyyQBYr9tS8uzuHKrqhDOPpMCo-ouwH3LMKdgJhzWFXj_qYIPzBrkKS7vzYe8E0OkUIj7oKGsjUOTxCVHmqyYQh2DPqJS0n0EDFRWjrHFo99SvV2qmzNVpAtKg-eVWU8g");

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

    String::from(resp)
}

async fn query_workspace() -> String {
    String::from("query workspace")
}
