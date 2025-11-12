#[cfg(not(test))]
use dotenvy::dotenv;
use notary_server::{create_app, db};

#[tokio::main]
async fn main() {
    #[cfg(not(test))]
    dotenv().ok();

    let pool = db::create_pool().await.expect("Failed to create database pool");

    let app = create_app(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
