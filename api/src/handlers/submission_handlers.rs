use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use shared::*;
use uuid::Uuid;

use crate::AppState;

pub async fn submit_code(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<SubmissionRequest>,
) -> Result<Json<SubmissionResponse>, StatusCode> {
    // Verify problem exists
    let _problem = state
        .db
        .get_problem(payload.problem_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Verify language exists
    let _language = state
        .db
        .get_language(payload.language_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Create submission
    let submission = state
        .db
        .create_submission(&payload, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create judging job
    let judging_job = JudgingJob {
        submission_id: submission.id,
        user_id: user.id,
        problem_id: payload.problem_id,
        language_id: payload.language_id,
        source_code: payload.source_code,
    };

    // Send to queue
    state
        .queue
        .publish_judging_job(&judging_job)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SubmissionResponse {
        submission_id: submission.id,
    }))
}

pub async fn get_submission(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
) -> Result<Json<Submission>, StatusCode> {
    let submission = state
        .db
        .get_submission(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if user owns the submission or is admin
    if submission.user_id != user.id && !user.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(submission))
}

pub async fn get_problem_submissions(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(problem_id): Path<Uuid>,
) -> Result<Json<Vec<Submission>>, StatusCode> {
    let submissions = state
        .db
        .list_problem_submissions(problem_id, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(submissions))
}