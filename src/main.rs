use axum::{http::StatusCode, Router};
use serde::Serialize;
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    types::chrono::NaiveDateTime,
    Pool, Sqlite,
};
use tera::Tera;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod router;
use router::app_router;
use std::{net::SocketAddr, path::Path, sync::Arc, time::Duration};
mod ai;
mod middleware;
use middleware::extract_user;
mod data;
use data::repository::ChatRepository;

use crate::middleware::handle_error;

#[derive(Clone)]
struct AppState {
    pool: Arc<Pool<Sqlite>>,
    tera: Tera,
    chat_repo: ChatRepository,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_path = dotenv::var("DATABASE_PATH").unwrap();
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .create_if_missing(true);

    // setup connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect_with(options)
        .await
        .expect("can't connect to database");

    // Create a new instance of `Migrator` pointing to the migrations folder.
    let migrator = Migrator::new(Path::new(dotenv::var("MIGRATIONS_PATH").unwrap().as_str()))
        .await
        .unwrap();
    // Run the migrations.
    migrator.run(&pool).await.unwrap();

    let pool = Arc::new(pool);

    let chat_repo = ChatRepository { pool: pool.clone() };

    let static_files = ServeDir::new("assets");

    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    let state = AppState {
        pool,
        tera,
        chat_repo,
    };
    let shared_app_state = Arc::new(state);

    // let jdoom = axum::middleware::from_fn_with_state(shared_app_state.clone(), auth);

    // build our application with some routes
    let app = Router::new()
        // .route(
        //     "/",
        //     get(using_connection_pool_extractor).post(using_connection_pool_extractor),
        // )
        // Use `merge` to combine routers
        .nest_service("/assets", static_files)
        .with_state(shared_app_state.clone())
        .nest("/", app_router(shared_app_state.clone()))
        .layer(axum::middleware::from_fn_with_state(
            shared_app_state.clone(),
            handle_error,
        ))
        .layer(axum::middleware::from_fn_with_state(
            shared_app_state.clone(),
            extract_user,
        ))
        .layer(CookieManagerLayer::new());

    // run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, sqlx::FromRow, Serialize, Clone)]
pub struct User {
    id: i64,
    email: String,
    password: String,
    created_at: NaiveDateTime,
    openai_api_key: Option<String>,
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
