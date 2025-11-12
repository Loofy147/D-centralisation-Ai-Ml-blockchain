use axum::{
    routing::{get, post},
    Router,
};

pub mod db;
pub mod error;
pub mod handlers;

use db::DbPool;

pub fn create_app(pool: DbPool) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/v1/task", get(handlers::get_task))
        .route("/api/v1/submit", post(handlers::submit_claim))
        .with_state(pool)
}
