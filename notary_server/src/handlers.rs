use axum::{extract::State, Json};
use axum_extra::extract::Multipart;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::types::BigDecimal;
use uuid::Uuid;

use crate::{db::DbPool, error::AppError};

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub task_id: String,
    pub performance_threshold: BigDecimal,
    pub dataset_hash: String,
    pub optuna_storage_url: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Submission {
    pub miner_id: Uuid,
    pub task_id: String,
    pub claimed_score: BigDecimal,
    pub artifact_hash: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub nonce: String,
}

#[derive(Serialize)]
pub struct SubmissionStatus {
    pub status: String,
    pub submission_id: String,
    pub estimated_verification_time_seconds: u32,
}

#[axum::debug_handler]
pub async fn get_task(State(pool): State<DbPool>) -> Result<Json<Task>, AppError> {
    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE is_active = TRUE ORDER BY created_at DESC LIMIT 1")
        .fetch_one(&pool)
        .await?;
    Ok(Json(task))
}

#[axum::debug_handler]
pub async fn submit_claim(State(pool): State<DbPool>, mut multipart: Multipart) -> Result<Json<SubmissionStatus>, AppError> {
    let mut payload = None;
    let mut artifact = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await?;

        match name.as_str() {
            "payload" => {
                payload = Some(serde_json::from_slice::<Submission>(&data)?);
            }
            "artifact" => {
                artifact = Some(data);
            }
            _ => (),
        }
    }

    let payload = payload.ok_or_else(|| AppError::MissingField("payload".to_string()))?;
    let _artifact = artifact.ok_or_else(|| AppError::MissingField("artifact".to_string()))?;
    let submission_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO ledger (submission_id, miner_id, task_id, claimed_score, verified_score, artifact_hash, artifact_uri, signature_hex, timestamp, verification_duration_ms, nonce)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(submission_id)
    .bind(payload.miner_id)
    .bind(payload.task_id)
    .bind(payload.claimed_score)
    .bind(BigDecimal::from(0)) // verified_score
    .bind(payload.artifact_hash)
    .bind("") // artifact_uri
    .bind("") // signature_hex
    .bind(payload.timestamp)
    .bind(0) // verification_duration_ms
    .bind(payload.nonce)
    .execute(&pool)
    .await?;

    let status = SubmissionStatus {
        status: "pending_verification".to_string(),
        submission_id: submission_id.to_string(),
        estimated_verification_time_seconds: 180,
    };
    Ok(Json(status))
}
