use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form, Json,
};

use serde::Deserialize;
use tera::Context;
use tower_cookies::{Cookie, Cookies};

use std::sync::Arc;

use crate::{AppState, User};

pub async fn login(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("name", "World");
    let home = state.tera.render("views/login.html", &context).unwrap();

    let mut context = Context::new();
    context.insert("view", &home);
    let rendered = state.tera.render("views/main.html", &context).unwrap();

    Html(rendered)
}

#[derive(Debug)]
pub enum LogInError {
    InvalidCredentials,
    DatabaseError(String),
}

impl IntoResponse for LogInError {
    fn into_response(self) -> Response {
        match self {
            LogInError::InvalidCredentials => (
                StatusCode::BAD_REQUEST,
                Json("Invalid Username or Password"),
            )
                .into_response(),
            LogInError::DatabaseError(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(message)).into_response()
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct LogIn {
    email: String,
    password: String,
}

#[axum::debug_handler]
pub async fn login_form(
    cookies: Cookies,
    state: State<Arc<AppState>>,
    Form(log_in): Form<LogIn>,
) -> Result<Redirect, LogInError> {
    // Verify password
    let user = sqlx::query_as!(
        User,
        "SELECT users.*, settings.openai_api_key FROM users LEFT JOIN settings ON settings.user_id=users.id WHERE users.email = $1",
        log_in.email,
    ).fetch_one(&*state.pool).await
    .map_err(|_| LogInError::InvalidCredentials)?;

    if user.password != log_in.password {
        return Err(LogInError::InvalidCredentials);
    }

    let cookie = Cookie::build("rust-gpt-session", user.id.to_string())
        // .domain("www.rust-lang.org")
        .path("/")
        // .secure(true)
        .http_only(true)
        .finish();
    cookies.add(cookie);

    Ok(Redirect::to("/"))
}

pub async fn signup(State(state): State<Arc<AppState>>) -> Html<String> {
    // TODO: Hash password
    let mut context = Context::new();
    context.insert("name", "World");
    let home = state.tera.render("views/signup.html", &context).unwrap();

    let mut context = Context::new();
    context.insert("view", &home);
    let rendered = state.tera.render("views/main.html", &context).unwrap();

    Html(rendered)
}

#[derive(Debug)]
pub enum SignUpError {
    PasswordMismatch,
    DatabaseError(String),
}

impl IntoResponse for SignUpError {
    fn into_response(self) -> Response {
        match self {
            SignUpError::PasswordMismatch => {
                (StatusCode::BAD_REQUEST, Json("Passwords do not match.")).into_response()
            }
            SignUpError::DatabaseError(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(message)).into_response()
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct SignUp {
    email: String,
    password: String,
    password_confirmation: String,
}

#[axum::debug_handler]
pub async fn form_signup(
    state: State<Arc<AppState>>,
    Form(sign_up): Form<SignUp>,
) -> Result<Redirect, SignUpError> {
    if sign_up.password != sign_up.password_confirmation {
        return Err(SignUpError::PasswordMismatch);
    }

    // insert into db
    match sqlx::query!(
        "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id",
        sign_up.email,
        sign_up.password
    )
    .fetch_one(&*state.pool)
    .await
    {
        Ok(_) => Ok(Redirect::to("/login")),
        Err(_e) => {
            // Handle database error, for example, a unique constraint violation
            Err(SignUpError::DatabaseError("Dat".to_string()))
        }
    }
}

#[axum::debug_handler]
pub async fn logout(cookies: Cookies) -> Result<Redirect, StatusCode> {
    let mut cookie = Cookie::build("rust-gpt-session", "")
        // .domain("www.rust-lang.org")
        .path("/")
        // .secure(true)
        .http_only(true)
        .finish();
    cookie.make_removal();

    cookies.add(cookie);

    Ok(Redirect::to("/"))
}
