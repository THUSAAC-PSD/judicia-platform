use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use shared::*;
use uuid::Uuid;

use crate::KernelState;

pub async fn list_contests(
    State(state): State<KernelState>,
) -> Result<Json<Vec<Contest>>, StatusCode> {
    let contests = state
        .db
        .list_contests()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(contests))
}

pub async fn get_contest(
    State(state): State<KernelState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Contest>, StatusCode> {
    let contest = state
        .db
        .get_contest(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(contest))
}

pub async fn create_contest(
    State(state): State<KernelState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateContestRequest>,
) -> Result<Json<Contest>, StatusCode> {
    // Check if user is admin
    if !user.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let contest = state
        .db
        .create_contest(&payload, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(contest))
}

pub async fn get_contest_problems(
    State(state): State<KernelState>,
    Path(contest_id): Path<Uuid>,
) -> Result<Json<Vec<Problem>>, StatusCode> {
    let problems = state
        .db
        .list_problems(Some(contest_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(problems))
}