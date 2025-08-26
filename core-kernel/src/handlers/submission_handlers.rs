use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use shared::*;
use uuid::Uuid;

use crate::KernelState;

pub async fn submit_code(
    State(state): State<KernelState>,
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

    // Create evaluation job
    let evaluation_job = evaluation_engine::job_queue::EvaluationJob {
        id: uuid::Uuid::new_v4(),
        submission_id: submission.id,
        problem_id: payload.problem_id,
        language_id: payload.language_id,
        source_code: payload.source_code,
        priority: 1, // Normal priority
        timeout_ms: 5000, // 5 seconds default
        memory_limit_kb: 256 * 1024, // 256MB default
        test_case_count: 1, // Will be determined by problem
        created_at: chrono::Utc::now(),
        retry_count: 0,
        max_retries: 3,
        metadata: serde_json::json!({}),
    };

    // Send to queue
    state
        .kernel
        .job_queue()
        .submit_job(evaluation_job)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SubmissionResponse {
        submission_id: submission.id,
    }))
}

pub async fn get_submission(
    State(state): State<KernelState>,
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
    State(state): State<KernelState>,
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