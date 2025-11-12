use notary_server::{create_app, db, handlers::{Submission, Task}};
use axum::http::StatusCode;
use axum_test::TestServer;
use axum_test::multipart::{MultipartForm, Part};
use serde_json;
use serial_test::serial;
use sqlx::{types::BigDecimal, Executor};
use std::str::FromStr;
use uuid::Uuid;

async fn setup_db(pool: &db::DbPool) {
    // Clean up any previous test data
    pool.execute("TRUNCATE TABLE tasks, miner_keys, ledger RESTART IDENTITY").await.unwrap();

    // Insert a test task
    sqlx::query("INSERT INTO tasks (task_id, performance_threshold, dataset_hash, optuna_storage_url, is_active) VALUES ($1, $2, $3, $4, $5)")
        .bind("test_task")
        .bind(BigDecimal::from_str("0.9").unwrap())
        .bind("test_dataset_hash")
        .bind("test_optuna_storage_url")
        .bind(true)
        .execute(pool)
        .await
        .unwrap();

    // Insert a test miner
    sqlx::query("INSERT INTO miner_keys (miner_id, public_key_hex) VALUES ($1, $2)")
        .bind(Uuid::new_v4())
        .bind("test_public_key")
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
async fn test_get_task() {
    std::env::set_var("DATABASE_URL", "postgres://user:password@localhost:5432/ml_chain");
    let pool = db::create_pool().await.unwrap();
    setup_db(&pool).await;
    let app = create_app(pool);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/api/v1/task").await;

    response.assert_status(StatusCode::OK);
    let task: Task = response.json();
    assert_eq!(task.task_id, "test_task");
}

#[tokio::test]
#[serial]
async fn test_submit_claim() {
    std::env::set_var("DATABASE_URL", "postgres://user:password@localhost:5432/ml_chain");
    let pool = db::create_pool().await.unwrap();
    setup_db(&pool).await;
    let app = create_app(pool.clone());
    let server = TestServer::new(app).unwrap();

    let miner_id: Uuid = sqlx::query_scalar("SELECT miner_id FROM miner_keys LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();

    let submission = Submission {
        miner_id,
        task_id: "test_task".to_string(),
        claimed_score: BigDecimal::from_str("0.99").unwrap(),
        artifact_hash: "test_hash".to_string(),
        timestamp: chrono::Utc::now(),
        nonce: "test_nonce".to_string(),
    };

    let form = MultipartForm::new()
        .add_part("payload", Part::text(serde_json::to_string(&submission).unwrap()))
        .add_part("artifact", Part::bytes(b"test_artifact".to_vec()));

    let response = server.post("/api/v1/submit").multipart(form).await;

    response.assert_status(StatusCode::OK);

    // Verify that the submission was inserted into the ledger
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ledger")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
#[serial]
async fn test_submit_claim_missing_payload() {
    std::env::set_var("DATABASE_URL", "postgres://user:password@localhost:5432/ml_chain");
    let pool = db::create_pool().await.unwrap();
    let app = create_app(pool);
    let server = TestServer::new(app).unwrap();

    let form = MultipartForm::new().add_part("artifact", Part::bytes(b"test_artifact".to_vec()));

    let response = server.post("/api/v1/submit").multipart(form).await;

    response.assert_status(StatusCode::BAD_REQUEST);
}
