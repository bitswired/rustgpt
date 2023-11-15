use axum::{
    extract::State,
    http::{HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};

use tera::Context;
use tower_cookies::Cookies;

use std::sync::Arc;

use crate::{AppState, User};

pub fn error_response(code: u16, message: &str) -> Response {
    let to = format!("/error?code={}&message={}", code, message);
    let r = Redirect::to(&to);
    let mut r = r.into_response();
    let h = r.headers_mut();
    h.insert("HX-Redirect", HeaderValue::from_str(&to).unwrap());
    r
}

pub async fn extract_user<B>(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode>
where
    B: Send + 'static,
{
    let session = cookies.get("rust-gpt-session");

    let id = session.map_or(-1, |x| x.value().parse::<i64>().unwrap_or(-1));

    // Get the user
    match sqlx::query_as!(
        User,
        "SELECT users.*, settings.openai_api_key FROM users LEFT JOIN settings ON settings.user_id=users.id WHERE users.id = $1",
        id
    )
    .fetch_one(&*state.pool)
    .await
    {
        Ok(current_user) => {
            // insert the current user into a request extension so the handler can
            // extract it, and make sure `user` is not used after this point
            req.extensions_mut().insert(Some(current_user));
            Ok(next.run(req).await)
        }
        _ => {
            req.extensions_mut().insert(None::<User>);
            Ok(next.run(req).await)
        }
    }
}

pub async fn auth<B>(
    Extension(current_user): Extension<Option<User>>,
    req: Request<B>,
    next: Next<B>,
) -> Response
where
    B: Send + 'static,
{
    let to = format!("/error?code={}&message={}", "401", "Log in");
    let r = Redirect::to(&to);
    let mut r = r.into_response();
    let h = r.headers_mut();
    h.insert("HX-Redirect", HeaderValue::from_str(&to).unwrap());

    match current_user {
        Some(_user) => next.run(req).await,
        _ => error_response(401, "You need to log in to view this page"),
    }
}

pub async fn valid_openai_api_key<B>(
    Extension(current_user): Extension<Option<User>>,
    req: Request<B>,
    next: Next<B>,
) -> Response
where
    B: Send + 'static,
{
    let key = current_user
        .unwrap()
        .openai_api_key
        .unwrap_or(String::new());

    let client = reqwest::Client::new();
    match client
        .get("https://api.openai.com/v1/engines")
        .bearer_auth(&key)
        .send()
        .await
    {
        Ok(res) => {
            if res.status().is_success() {
                next.run(req).await
            } else {
                println!("failure!");
                error_response(403, "You API key is not set or invalid. Go to Settings.")
            }
        }
        Err(_) => error_response(403, "You API key is not set or invalid. Go to Settings"),
    }
}

pub async fn handle_error<B>(
    Extension(current_user): Extension<Option<User>>,
    State(state): State<Arc<AppState>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode>
where
    B: Send + 'static,
{
    let response = next.run(req).await;

    let status_code = response.status().as_u16();
    let status_text = response.status().as_str().to_string();

    match status_code {
        _ if status_code >= 400 => {
            let mut context = Context::new();
            context.insert("status_code", &status_code);
            context.insert("status_text", &status_text);

            let error = state.tera.render("views/error.html", &context).unwrap();

            let mut context = Context::new();
            context.insert("view", &error);
            context.insert("current_user", &current_user);
            context.insert("with_footer", &true);
            let rendered = state.tera.render("views/main.html", &context).unwrap();
            let h = Html(rendered).into_response();
            Ok(h)
        }
        _ => Ok(response),
    }
}
