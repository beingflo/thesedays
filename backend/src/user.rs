use crate::{auth::AuthenticatedUser, error::AppError};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct S3Data {
    endpoint: String,
    region: String,
    access_key: String,
    secret_key: String,
}

pub async fn update_config(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(data): Json<S3Data>,
) -> Result<StatusCode, AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection
        .prepare("UPDATE users SET endpoint = ?1, region = ?2, access_key = ?3, secret_key = ?4 WHERE id = ?5")
        .unwrap();

    stmt.execute([
        data.endpoint,
        data.region,
        data.access_key,
        data.secret_key,
        user.id.to_string(),
    ])?;

    Ok(StatusCode::OK)
}
