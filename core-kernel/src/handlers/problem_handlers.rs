use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use shared::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::KernelState;

#[derive(Deserialize)]
pub struct ListProblemsQuery {
    contest_id: Option<Uuid>,
}

pub async fn list_problems(
    State(state): State<KernelState>,
    Query(query): Query<ListProblemsQuery>,
) -> Result<Json<Vec<Problem>>, StatusCode> {
    let problems = state
        .db
        .list_problems(query.contest_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(problems))
}

pub async fn get_problem(
    State(state): State<KernelState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Problem>, StatusCode> {
    let problem = state
        .db
        .get_problem(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(problem))
}

pub async fn create_problem(
    State(state): State<KernelState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateProblemRequest>,
) -> Result<Json<Problem>, StatusCode> {
    // Check if user is admin
    if !user.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let problem = state
        .db
        .create_problem(&payload, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(problem))
}