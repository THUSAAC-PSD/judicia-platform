use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use sqlx::Row;

use crate::KernelState;
use shared::User;

// Helper function to check if user is admin
fn is_admin_user(user: &User) -> bool {
    user.roles.contains(&"admin".to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RejudgeType {
    #[serde(rename = "full")]
    Full,        // 完整重新评测 (重新编译+执行)
    #[serde(rename = "score_only")]
    ScoreOnly,   // 仅重新计分 (不重新执行)
    #[serde(rename = "compile_only")]
    CompileOnly, // 仅重新编译
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RejudgeStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RejudgeRequest {
    pub contest_id: Option<Uuid>,
    pub problem_id: Option<Uuid>,
    pub submission_ids: Vec<Uuid>,
    pub rejudge_type: RejudgeType,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RejudgeJob {
    pub id: Uuid,
    pub contest_id: Option<Uuid>,
    pub problem_id: Option<Uuid>,
    pub submission_ids: Vec<Uuid>,
    pub rejudge_type: RejudgeType,
    pub reason: String,
    pub status: RejudgeStatus,
    pub requested_by: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub progress: RejudgeProgress,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RejudgeProgress {
    pub total_submissions: i32,
    pub completed_submissions: i32,
    pub failed_submissions: i32,
    pub current_submission: Option<Uuid>,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejudgeQuery {
    pub status: Option<RejudgeStatus>,
    pub contest_id: Option<Uuid>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Create a new rejudge request
pub async fn create_rejudge_request(
    State(state): State<KernelState>,
    Extension(admin_user): Extension<User>,
    Json(request): Json<RejudgeRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Verify admin permissions
    if !is_admin_user(&admin_user) {
        // Check if user is contest admin for the specific contest
        if let Some(contest_id) = request.contest_id {
            if !is_contest_admin(&state, &admin_user.id, &contest_id).await? {
                return Err(StatusCode::FORBIDDEN);
            }
        } else {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Validate submission IDs exist
    validate_submission_ids(&state, &request.submission_ids).await?;

    // Create rejudge job
    let rejudge_job = create_rejudge_job(&state, request, &admin_user).await?;

    // Queue the rejudge job for processing
    queue_rejudge_job(&state, &rejudge_job).await?;

    Ok(Json(json!({
        "success": true,
        "rejudge_job": rejudge_job,
        "message": "Rejudge request created successfully"
    })))
}

/// Get rejudge job status
pub async fn get_rejudge_job(
    State(state): State<KernelState>,
    Extension(user): Extension<User>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let job = get_rejudge_job_from_db(&state, job_id).await?;

    // Check permissions
    if !is_admin_user(&user) && job.requested_by != user.id {
        if let Some(contest_id) = job.contest_id {
            if !is_contest_admin(&state, &user.id, &contest_id).await? {
                return Err(StatusCode::FORBIDDEN);
            }
        } else {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    Ok(Json(json!({
        "rejudge_job": job
    })))
}

/// List rejudge jobs
pub async fn list_rejudge_jobs(
    State(state): State<KernelState>,
    Extension(user): Extension<User>,
    Query(query): Query<RejudgeQuery>,
) -> Result<Json<Value>, StatusCode> {
    // Only admins can list all rejudge jobs
    if !is_admin_user(&user) && query.contest_id.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    // If contest_id is specified, check contest admin permissions
    if let Some(contest_id) = query.contest_id {
        if !is_admin_user(&user) && !is_contest_admin(&state, &user.id, &contest_id).await? {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let jobs = list_rejudge_jobs_from_db(&state, &query, user.id, is_admin_user(&user)).await?;

    Ok(Json(json!({
        "rejudge_jobs": jobs,
        "total": jobs.len()
    })))
}

/// Cancel a rejudge job
pub async fn cancel_rejudge_job(
    State(state): State<KernelState>,
    Extension(admin_user): Extension<User>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let job = get_rejudge_job_from_db(&state, job_id).await?;

    // Check permissions
    if !is_admin_user(&admin_user) && job.requested_by != admin_user.id {
        if let Some(contest_id) = job.contest_id {
            if !is_contest_admin(&state, &admin_user.id, &contest_id).await? {
                return Err(StatusCode::FORBIDDEN);
            }
        } else {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Can only cancel pending jobs
    if !matches!(job.status, RejudgeStatus::Pending) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Update job status to failed
    let query = r"
        UPDATE rejudge_requests 
        SET status = 'failed', completed_at = NOW()
        WHERE id = $1
    ";

    sqlx::query(query)
        .bind(job_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "success": true,
        "message": "Rejudge job cancelled successfully"
    })))
}

/// Batch rejudge submissions by criteria
pub async fn batch_rejudge_submissions(
    State(state): State<KernelState>,
    Extension(admin_user): Extension<User>,
    Json(request): Json<BatchRejudgeRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Verify admin permissions
    if !is_admin_user(&admin_user) {
        if let Some(contest_id) = request.contest_id {
            if !is_contest_admin(&state, &admin_user.id, &contest_id).await? {
                return Err(StatusCode::FORBIDDEN);
            }
        } else {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Find submissions matching criteria
    let submission_ids = find_submissions_by_criteria(&state, &request).await?;

    if submission_ids.is_empty() {
        return Ok(Json(json!({
            "success": false,
            "message": "No submissions found matching the criteria"
        })));
    }

    // Create rejudge request
    let rejudge_request = RejudgeRequest {
        contest_id: request.contest_id,
        problem_id: request.problem_id,
        submission_ids,
        rejudge_type: request.rejudge_type,
        reason: request.reason,
    };

    let rejudge_job = create_rejudge_job(&state, rejudge_request, &admin_user).await?;
    queue_rejudge_job(&state, &rejudge_job).await?;

    Ok(Json(json!({
        "success": true,
        "rejudge_job": rejudge_job,
        "message": format!("Batch rejudge created for {} submissions", rejudge_job.submission_ids.len())
    })))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchRejudgeRequest {
    pub contest_id: Option<Uuid>,
    pub problem_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub verdict: Option<String>,
    pub language_id: Option<Uuid>,
    pub submission_time_from: Option<DateTime<Utc>>,
    pub submission_time_to: Option<DateTime<Utc>>,
    pub rejudge_type: RejudgeType,
    pub reason: String,
}

// Implementation functions
async fn create_rejudge_job(
    state: &KernelState,
    request: RejudgeRequest,
    admin_user: &User,
) -> Result<RejudgeJob, StatusCode> {
    let job_id = Uuid::new_v4();
    let submission_ids_json = serde_json::to_value(&request.submission_ids)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rejudge_type_str = match request.rejudge_type {
        RejudgeType::Full => "full",
        RejudgeType::ScoreOnly => "score_only",
        RejudgeType::CompileOnly => "compile_only",
    };

    let query = r"
        INSERT INTO rejudge_requests (
            id, contest_id, problem_id, submission_ids, rejudge_type, reason, 
            status, requested_by, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, NOW())
        RETURNING id, contest_id, problem_id, submission_ids, rejudge_type, reason,
                  status, requested_by, started_at, completed_at, created_at
    ";

    let row = sqlx::query(query)
        .bind(job_id)
        .bind(request.contest_id)
        .bind(request.problem_id)
        .bind(&submission_ids_json)
        .bind(rejudge_type_str)
        .bind(&request.reason)
        .bind(admin_user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let submission_ids: Vec<Uuid> = serde_json::from_value(row.get("submission_ids"))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(RejudgeJob {
        id: row.get("id"),
        contest_id: row.get("contest_id"),
        problem_id: row.get("problem_id"),
        submission_ids: submission_ids.clone(),
        rejudge_type: request.rejudge_type,
        reason: row.get("reason"),
        status: RejudgeStatus::Pending,
        requested_by: row.get("requested_by"),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        created_at: row.get("created_at"),
        progress: RejudgeProgress {
            total_submissions: submission_ids.len() as i32,
            completed_submissions: 0,
            failed_submissions: 0,
            current_submission: None,
            errors: vec![],
        },
    })
}

async fn queue_rejudge_job(state: &KernelState, job: &RejudgeJob) -> Result<(), StatusCode> {
    // In a real implementation, this would queue the job to a background worker
    // For now, we'll just mark it as queued
    println!("Rejudge job queued: {} ({} submissions)", job.id, job.submission_ids.len());
    Ok(())
}

async fn get_rejudge_job_from_db(state: &KernelState, job_id: Uuid) -> Result<RejudgeJob, StatusCode> {
    let query = r"
        SELECT id, contest_id, problem_id, submission_ids, rejudge_type, reason,
               status, requested_by, started_at, completed_at, created_at
        FROM rejudge_requests
        WHERE id = $1
    ";

    let row = sqlx::query(query)
        .bind(job_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let submission_ids: Vec<Uuid> = serde_json::from_value(row.get("submission_ids"))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rejudge_type = match row.get::<String, _>("rejudge_type").as_str() {
        "full" => RejudgeType::Full,
        "score_only" => RejudgeType::ScoreOnly,
        "compile_only" => RejudgeType::CompileOnly,
        _ => RejudgeType::Full,
    };

    let status = match row.get::<String, _>("status").as_str() {
        "pending" => RejudgeStatus::Pending,
        "in_progress" => RejudgeStatus::InProgress,
        "completed" => RejudgeStatus::Completed,
        "failed" => RejudgeStatus::Failed,
        _ => RejudgeStatus::Pending,
    };

    Ok(RejudgeJob {
        id: row.get("id"),
        contest_id: row.get("contest_id"),
        problem_id: row.get("problem_id"),
        submission_ids: submission_ids.clone(),
        rejudge_type,
        reason: row.get("reason"),
        status,
        requested_by: row.get("requested_by"),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        created_at: row.get("created_at"),
        progress: RejudgeProgress {
            total_submissions: submission_ids.len() as i32,
            completed_submissions: 0, // Would need to calculate this
            failed_submissions: 0,
            current_submission: None,
            errors: vec![],
        },
    })
}

async fn list_rejudge_jobs_from_db(
    state: &KernelState,
    query: &RejudgeQuery,
    user_id: Uuid,
    is_admin: bool,
) -> Result<Vec<RejudgeJob>, StatusCode> {
    let mut sql_query = r"
        SELECT id, contest_id, problem_id, submission_ids, rejudge_type, reason,
               status, requested_by, started_at, completed_at, created_at
        FROM rejudge_requests
        WHERE 1=1
    ".to_string();

    let mut bind_params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + 'static>> = vec![];
    let mut param_count = 0;

    if let Some(status) = &query.status {
        param_count += 1;
        let status_str = match status {
            RejudgeStatus::Pending => "pending",
            RejudgeStatus::InProgress => "in_progress", 
            RejudgeStatus::Completed => "completed",
            RejudgeStatus::Failed => "failed",
        };
        sql_query.push_str(&format!(" AND status = ${}", param_count));
        bind_params.push(Box::new(status_str.to_string()));
    }

    if let Some(contest_id) = query.contest_id {
        param_count += 1;
        sql_query.push_str(&format!(" AND contest_id = ${}", param_count));
        bind_params.push(Box::new(contest_id));
    }

    if !is_admin {
        param_count += 1;
        sql_query.push_str(&format!(" AND requested_by = ${}", param_count));
        bind_params.push(Box::new(user_id));
    }

    sql_query.push_str(" ORDER BY created_at DESC");

    if let Some(limit) = query.limit {
        param_count += 1;
        sql_query.push_str(&format!(" LIMIT ${}", param_count));
        bind_params.push(Box::new(limit));
    }

    if let Some(offset) = query.offset {
        param_count += 1;
        sql_query.push_str(&format!(" OFFSET ${}", param_count));
        bind_params.push(Box::new(offset));
    }

    // Execute query - simplified version
    let rows = sqlx::query(&sql_query)
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut jobs = Vec::new();
    for row in rows {
        let submission_ids: Vec<Uuid> = serde_json::from_value(row.get("submission_ids"))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let rejudge_type = match row.get::<String, _>("rejudge_type").as_str() {
            "full" => RejudgeType::Full,
            "score_only" => RejudgeType::ScoreOnly,
            "compile_only" => RejudgeType::CompileOnly,
            _ => RejudgeType::Full,
        };

        let status = match row.get::<String, _>("status").as_str() {
            "pending" => RejudgeStatus::Pending,
            "in_progress" => RejudgeStatus::InProgress,
            "completed" => RejudgeStatus::Completed,
            "failed" => RejudgeStatus::Failed,
            _ => RejudgeStatus::Pending,
        };

        jobs.push(RejudgeJob {
            id: row.get("id"),
            contest_id: row.get("contest_id"),
            problem_id: row.get("problem_id"),
            submission_ids: submission_ids.clone(),
            rejudge_type,
            reason: row.get("reason"),
            status,
            requested_by: row.get("requested_by"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
            created_at: row.get("created_at"),
            progress: RejudgeProgress {
                total_submissions: submission_ids.len() as i32,
                completed_submissions: 0,
                failed_submissions: 0,
                current_submission: None,
                errors: vec![],
            },
        });
    }

    Ok(jobs)
}

async fn find_submissions_by_criteria(
    state: &KernelState,
    request: &BatchRejudgeRequest,
) -> Result<Vec<Uuid>, StatusCode> {
    let mut query = r"
        SELECT s.id
        FROM submissions s
        LEFT JOIN problems p ON s.problem_id = p.id
        WHERE 1=1
    ".to_string();

    // Build dynamic query based on criteria
    // This is a simplified version - in practice, you'd use a proper query builder

    if let Some(contest_id) = request.contest_id {
        query.push_str(" AND p.contest_id = $1");
    }

    if let Some(problem_id) = request.problem_id {
        query.push_str(" AND s.problem_id = $2");
    }

    // Execute the query and return submission IDs
    let rows = sqlx::query(&query)
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let submission_ids = rows.into_iter()
        .map(|row| row.get::<Uuid, _>("id"))
        .collect();

    Ok(submission_ids)
}

async fn validate_submission_ids(state: &KernelState, submission_ids: &[Uuid]) -> Result<(), StatusCode> {
    if submission_ids.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if all submission IDs exist
    let query = r"SELECT COUNT(*) as count FROM submissions WHERE id = ANY($1)";
    let ids: Vec<String> = submission_ids.iter().map(|id| id.to_string()).collect();
    
    let row = sqlx::query(query)
        .bind(&ids)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let found_count: i64 = row.get("count");
    
    if found_count != submission_ids.len() as i64 {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(())
}

async fn is_contest_admin(state: &KernelState, user_id: &Uuid, contest_id: &Uuid) -> Result<bool, StatusCode> {
    let query = r"SELECT 1 FROM contest_admins WHERE user_id = $1 AND contest_id = $2";
    let is_admin = sqlx::query(query)
        .bind(user_id)
        .bind(contest_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    
    Ok(is_admin)
}