use crate::{
    auth::AuthenticatedUser,
    error::AppError,
    user::S3Data,
    utils::{get_bucket, get_file_name},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

const MAX_FILES_PER_REQUEST: u32 = 32;
const UPLOAD_LINK_TIMEOUT_SEC: u32 = 600;

#[derive(Serialize, Deserialize)]
pub struct UploadRequest {
    number: u32,
}

#[derive(Serialize, Deserialize)]
pub struct FileGroup {
    small: String,
    medium: String,
    original: String,
}

pub async fn upload_images(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(data): Json<UploadRequest>,
) -> Result<(StatusCode, Json<Vec<FileGroup>>), AppError> {
    if data.number > MAX_FILES_PER_REQUEST {
        return Err(AppError::Status(StatusCode::BAD_REQUEST));
    }
    let connection = state.conn.lock().await;

    let config: S3Data = match user.config {
        Some(c) => c,
        None => return Err(AppError::Status(StatusCode::BAD_REQUEST)),
    };

    let bucket = get_bucket(config)?;

    let mut files = Vec::new();
    for _ in 0..data.number {
        let small = get_file_name();
        let medium = get_file_name();
        let original = get_file_name();

        let url_small = bucket.presign_put(format!("/{}", small), UPLOAD_LINK_TIMEOUT_SEC, None)?;
        let url_medium =
            bucket.presign_put(format!("/{}", medium), UPLOAD_LINK_TIMEOUT_SEC, None)?;
        let url_original =
            bucket.presign_put(format!("/{}", original), UPLOAD_LINK_TIMEOUT_SEC, None)?;

        files.push(FileGroup {
            small: url_small,
            medium: url_medium,
            original: url_original,
        });

        connection.execute(
            "INSERT INTO images (filename_small, filename_medium, filename_original, user_id) VALUES (?1, ?2, ?3, ?4)",
            (&small, &medium, &original, &user.id),
        )?;
    }

    Ok((StatusCode::OK, Json(files)))
}

pub async fn get_images(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<FileGroup>>), AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection.prepare(
        "
            SELECT filename_small, filename_medium, filename_original 
            FROM images
            WHERE user_id = ?1
        ",
    )?;

    let file_groups = stmt.query_map([user.id], |row| {
        Ok(FileGroup {
            small: row.get(0)?,
            medium: row.get(1)?,
            original: row.get(2)?,
        })
    })?;

    let files = file_groups.collect::<Result<_, _>>()?;

    Ok((StatusCode::OK, Json(files)))
}

pub async fn get_image(
    user: AuthenticatedUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Redirect, AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection.prepare(
        "
            SELECT filename_small, filename_medium, filename_original 
            FROM images
            WHERE user_id = ?1 AND (filename_small = ?2 OR filename_medium = ?2 OR filename_original = ?2)
        ",
    )?;

    let mut file_groups = stmt.query_map([&user.id.to_string(), &id], |row| {
        Ok(FileGroup {
            small: row.get(0)?,
            medium: row.get(1)?,
            original: row.get(2)?,
        })
    })?;

    let config: S3Data = match user.config {
        Some(c) => c,
        None => return Err(AppError::Status(StatusCode::NOT_FOUND)),
    };

    let bucket = get_bucket(config)?;

    let group = file_groups.next();

    if let Some(group) = group {
        let group = group?;
        let mut name = "";
        if group.small == id {
            name = &group.small;
        }
        if group.medium == id {
            name = &group.medium;
        }
        if group.original == id {
            name = &group.original;
        }
        let url = bucket.presign_get(format!("/{}", name), UPLOAD_LINK_TIMEOUT_SEC, None)?;
        return Ok(Redirect::temporary(&url));
    }

    return Err(AppError::Status(StatusCode::NOT_FOUND));
}
