use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, Redirect},
    Form,
};

use serde::Deserialize;
use tera::Context;

use std::sync::Arc;

use crate::{AppState, User};

#[derive(Deserialize, Debug)]
pub struct OpenAiAPIKey {
    api_key: String,
}

#[axum::debug_handler]
pub async fn settings_openai_api_key(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<Option<User>>,
    Form(set_openai_api_key): Form<OpenAiAPIKey>,
) -> Result<Redirect, StatusCode> {
    let id = current_user.unwrap().id;
    sqlx::query!(
        "INSERT INTO settings (user_id, openai_api_key) VALUES (?, ?) ON CONFLICT (user_id) DO UPDATE SET openai_api_key = ?",
        id,
        set_openai_api_key.api_key,
        set_openai_api_key.api_key
    ).execute(&*state.pool).await.unwrap();

    Ok(Redirect::to("/settings"))
}

#[axum::debug_handler]
pub async fn settings(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<Option<User>>,
) -> Result<Html<String>, StatusCode> {
    let key = current_user.as_ref().unwrap().openai_api_key.as_ref();

    let mut context = Context::new();
    context.insert("openai_api_key", &key);

    let settings = state.tera.render("views/settings.html", &context).unwrap();

    let mut context = Context::new();
    context.insert("view", &settings);
    context.insert("current_user", &current_user);
    context.insert("with_footer", &true);
    let rendered = state.tera.render("views/main.html", &context).unwrap();

    Ok(Html(rendered))
}
