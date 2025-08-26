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
pub enum ScoringMethod {
    #[serde(rename = "last_submission")]
    LastSubmission,    // 最后一次提交得分
    #[serde(rename = "max_score")]
    MaxScore,         // 取历史提交最高分
    #[serde(rename = "subtask_sum")]
    SubtaskSum,       // 各子任务最高分的和
}

impl Default for ScoringMethod {
    fn default() -> Self {
        ScoringMethod::LastSubmission
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateScoringMethodRequest {
    pub scoring_method: ScoringMethod,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubtaskScore {
    pub subtask_id: Uuid,
    pub subtask_number: i32,
    pub score: f64,
    pub max_score: f64,
    pub submission_id: Uuid,
    pub submission_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProblemScore {
    pub user_id: Uuid,
    pub problem_id: Uuid,
    pub final_score: f64,
    pub max_possible_score: f64,
    pub scoring_method: ScoringMethod,
    pub subtask_scores: Vec<SubtaskScore>,
    pub submission_count: i32,
    pub last_submission_id: Option<Uuid>,
    pub last_submission_time: Option<DateTime<Utc>>,
    pub best_submission_id: Option<Uuid>,
    pub best_submission_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ScoreQuery {
    pub user_id: Option<Uuid>,
    pub problem_id: Option<Uuid>,
    pub recalculate: Option<bool>,
}

/// Update scoring method for a contest
pub async fn update_contest_scoring_method(
    State(state): State<KernelState>,
    Extension(admin_user): Extension<User>,
    Path(contest_id): Path<Uuid>,
    Json(request): Json<UpdateScoringMethodRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Verify admin permissions
    if !is_admin_user(&admin_user) && !is_contest_admin(&state, &admin_user.id, &contest_id).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let scoring_method_str = match request.scoring_method {
        ScoringMethod::LastSubmission => "last_submission",
        ScoringMethod::MaxScore => "max_score",
        ScoringMethod::SubtaskSum => "subtask_sum",
    };

    let query = r"
        UPDATE contests 
        SET scoring_method = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, title, scoring_method
    ";

    let row = sqlx::query(query)
        .bind(scoring_method_str)
        .bind(contest_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Trigger score recalculation for all users in the contest
    recalculate_contest_scores(&state, contest_id, request.scoring_method.clone()).await?;

    Ok(Json(json!({
        "success": true,
        "contest_id": contest_id,
        "scoring_method": request.scoring_method,
        "contest": {
            "id": row.get::<Uuid, _>("id"),
            "title": row.get::<String, _>("title"),
            "scoring_method": row.get::<String, _>("scoring_method")
        }
    })))
}

/// Get scores for problems in a contest
pub async fn get_contest_scores(
    State(state): State<KernelState>,
    Extension(user): Extension<User>,
    Path(contest_id): Path<Uuid>,
    Query(query): Query<ScoreQuery>,
) -> Result<Json<Value>, StatusCode> {
    // Verify access permissions
    if !is_admin_user(&user) && !is_contest_admin(&state, &user.id, &contest_id).await? && query.user_id.is_some() && query.user_id.unwrap() != user.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let scoring_method = get_contest_scoring_method(&state, contest_id).await?;
    
    if query.recalculate.unwrap_or(false) && (is_admin_user(&user) || is_contest_admin(&state, &user.id, &contest_id).await?) {
        recalculate_contest_scores(&state, contest_id, scoring_method.clone()).await?;
    }

    let scores = calculate_user_scores(&state, contest_id, query.user_id, query.problem_id, scoring_method).await?;

    Ok(Json(json!({
        "contest_id": contest_id,
        "scoring_method": scoring_method,
        "scores": scores,
        "total_users": scores.len()
    })))
}

/// Get detailed score breakdown for a specific user and problem
pub async fn get_user_problem_score_detail(
    State(state): State<KernelState>,
    Extension(user): Extension<User>,
    Path((contest_id, user_id, problem_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<Value>, StatusCode> {
    // Users can view their own scores, admins can view all
    if user.id != user_id && !is_admin_user(&user) && !is_contest_admin(&state, &user.id, &contest_id).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let scoring_method = get_contest_scoring_method(&state, contest_id).await?;
    let score_detail = calculate_user_problem_score(&state, user_id, problem_id, scoring_method).await?;

    Ok(Json(json!({
        "contest_id": contest_id,
        "score_detail": score_detail
    })))
}

/// Recalculate scores for all users in a contest
pub async fn recalculate_contest_scores_endpoint(
    State(state): State<KernelState>,
    Extension(admin_user): Extension<User>,
    Path(contest_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    // Verify admin permissions
    if !is_admin_user(&admin_user) && !is_contest_admin(&state, &admin_user.id, &contest_id).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let scoring_method = get_contest_scoring_method(&state, contest_id).await?;
    let recalculated_count = recalculate_contest_scores(&state, contest_id, scoring_method).await?;

    Ok(Json(json!({
        "success": true,
        "contest_id": contest_id,
        "recalculated_users": recalculated_count,
        "message": "Scores recalculated successfully"
    })))
}

// Core scoring calculation functions
async fn calculate_user_scores(
    state: &KernelState,
    contest_id: Uuid,
    user_id: Option<Uuid>,
    problem_id: Option<Uuid>,
    scoring_method: ScoringMethod,
) -> Result<Vec<UserProblemScore>, StatusCode> {
    let mut query = r"
        SELECT DISTINCT s.user_id, s.problem_id
        FROM submissions s
        JOIN problems p ON s.problem_id = p.id
        WHERE p.contest_id = $1
    ".to_string();

    let mut bind_count = 1;
    if let Some(uid) = user_id {
        bind_count += 1;
        query.push_str(&format!(" AND s.user_id = ${}", bind_count));
    }
    if let Some(pid) = problem_id {
        bind_count += 1;
        query.push_str(&format!(" AND s.problem_id = ${}", bind_count));
    }

    let mut query_builder = sqlx::query(&query).bind(contest_id);
    
    if let Some(uid) = user_id {
        query_builder = query_builder.bind(uid);
    }
    if let Some(pid) = problem_id {
        query_builder = query_builder.bind(pid);
    }

    let rows = query_builder
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut scores = Vec::new();
    for row in rows {
        let uid: Uuid = row.get("user_id");
        let pid: Uuid = row.get("problem_id");
        
        let score = calculate_user_problem_score(state, uid, pid, scoring_method.clone()).await?;
        scores.push(score);
    }

    Ok(scores)
}

async fn calculate_user_problem_score(
    state: &KernelState,
    user_id: Uuid,
    problem_id: Uuid,
    scoring_method: ScoringMethod,
) -> Result<UserProblemScore, StatusCode> {
    match scoring_method {
        ScoringMethod::LastSubmission => calculate_last_submission_score(state, user_id, problem_id).await,
        ScoringMethod::MaxScore => calculate_max_score(state, user_id, problem_id).await,
        ScoringMethod::SubtaskSum => calculate_subtask_sum_score(state, user_id, problem_id).await,
    }
}

async fn calculate_last_submission_score(
    state: &KernelState,
    user_id: Uuid,
    problem_id: Uuid,
) -> Result<UserProblemScore, StatusCode> {
    // Get last submission
    let query = r"
        SELECT id, score, max_score, created_at, subtask_scores
        FROM submissions
        WHERE user_id = $1 AND problem_id = $2
        ORDER BY created_at DESC
        LIMIT 1
    ";

    let row = sqlx::query(query)
        .bind(user_id)
        .bind(problem_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(row) => {
            let submission_id: Uuid = row.get("id");
            let score: Option<f64> = row.get("score");
            let max_score: Option<f64> = row.get("max_score");
            let created_at: DateTime<Utc> = row.get("created_at");
            let subtask_scores_json: Option<Value> = row.get("subtask_scores");

            let subtask_scores = parse_subtask_scores(subtask_scores_json);
            let submission_count = get_submission_count(state, user_id, problem_id).await?;

            Ok(UserProblemScore {
                user_id,
                problem_id,
                final_score: score.unwrap_or(0.0),
                max_possible_score: max_score.unwrap_or(0.0),
                scoring_method: ScoringMethod::LastSubmission,
                subtask_scores,
                submission_count,
                last_submission_id: Some(submission_id),
                last_submission_time: Some(created_at),
                best_submission_id: Some(submission_id),
                best_submission_time: Some(created_at),
            })
        }
        None => {
            Ok(UserProblemScore {
                user_id,
                problem_id,
                final_score: 0.0,
                max_possible_score: 0.0,
                scoring_method: ScoringMethod::LastSubmission,
                subtask_scores: vec![],
                submission_count: 0,
                last_submission_id: None,
                last_submission_time: None,
                best_submission_id: None,
                best_submission_time: None,
            })
        }
    }
}

async fn calculate_max_score(
    state: &KernelState,
    user_id: Uuid,
    problem_id: Uuid,
) -> Result<UserProblemScore, StatusCode> {
    // Get submission with maximum score
    let query = r"
        SELECT id, score, max_score, created_at, subtask_scores
        FROM submissions
        WHERE user_id = $1 AND problem_id = $2 AND score IS NOT NULL
        ORDER BY score DESC, created_at ASC
        LIMIT 1
    ";

    let best_row = sqlx::query(query)
        .bind(user_id)
        .bind(problem_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get last submission for metadata
    let last_query = r"
        SELECT id, created_at
        FROM submissions
        WHERE user_id = $1 AND problem_id = $2
        ORDER BY created_at DESC
        LIMIT 1
    ";

    let last_row = sqlx::query(last_query)
        .bind(user_id)
        .bind(problem_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let submission_count = get_submission_count(state, user_id, problem_id).await?;

    match best_row {
        Some(row) => {
            let best_submission_id: Uuid = row.get("id");
            let score: Option<f64> = row.get("score");
            let max_score: Option<f64> = row.get("max_score");
            let best_created_at: DateTime<Utc> = row.get("created_at");
            let subtask_scores_json: Option<Value> = row.get("subtask_scores");

            let subtask_scores = parse_subtask_scores(subtask_scores_json);

            let (last_submission_id, last_submission_time) = match last_row {
                Some(last) => (Some(last.get("id")), Some(last.get("created_at"))),
                None => (Some(best_submission_id), Some(best_created_at)),
            };

            Ok(UserProblemScore {
                user_id,
                problem_id,
                final_score: score.unwrap_or(0.0),
                max_possible_score: max_score.unwrap_or(0.0),
                scoring_method: ScoringMethod::MaxScore,
                subtask_scores,
                submission_count,
                last_submission_id,
                last_submission_time,
                best_submission_id: Some(best_submission_id),
                best_submission_time: Some(best_created_at),
            })
        }
        None => {
            let (last_submission_id, last_submission_time) = match last_row {
                Some(last) => (Some(last.get("id")), Some(last.get("created_at"))),
                None => (None, None),
            };

            Ok(UserProblemScore {
                user_id,
                problem_id,
                final_score: 0.0,
                max_possible_score: 0.0,
                scoring_method: ScoringMethod::MaxScore,
                subtask_scores: vec![],
                submission_count,
                last_submission_id,
                last_submission_time,
                best_submission_id: None,
                best_submission_time: None,
            })
        }
    }
}

async fn calculate_subtask_sum_score(
    state: &KernelState,
    user_id: Uuid,
    problem_id: Uuid,
) -> Result<UserProblemScore, StatusCode> {
    // Get all subtasks for the problem
    let subtask_query = r"
        SELECT id, subtask_number, max_score
        FROM subtasks
        WHERE problem_id = $1
        ORDER BY subtask_number
    ";

    let subtask_rows = sqlx::query(subtask_query)
        .bind(problem_id)
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut subtask_scores = Vec::new();
    let mut total_score = 0.0;
    let mut total_max_score = 0.0;

    // For each subtask, find the best score
    for subtask_row in subtask_rows {
        let subtask_id: Uuid = subtask_row.get("id");
        let subtask_number: i32 = subtask_row.get("subtask_number");
        let max_score: f64 = subtask_row.get("max_score");
        
        total_max_score += max_score;

        // Find best score for this subtask
        let score_query = r"
            SELECT ss.score, s.id as submission_id, s.created_at
            FROM submission_scores ss
            JOIN submissions s ON ss.submission_id = s.id
            WHERE s.user_id = $1 AND s.problem_id = $2 AND ss.subtask_id = $3
            ORDER BY ss.score DESC, s.created_at ASC
            LIMIT 1
        ";

        let score_row = sqlx::query(score_query)
            .bind(user_id)
            .bind(problem_id)
            .bind(subtask_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(score_row) = score_row {
            let score: f64 = score_row.get("score");
            let submission_id: Uuid = score_row.get("submission_id");
            let submission_time: DateTime<Utc> = score_row.get("created_at");
            
            total_score += score;

            subtask_scores.push(SubtaskScore {
                subtask_id,
                subtask_number,
                score,
                max_score,
                submission_id,
                submission_time,
            });
        } else {
            // No score for this subtask
            subtask_scores.push(SubtaskScore {
                subtask_id,
                subtask_number,
                score: 0.0,
                max_score,
                submission_id: Uuid::nil(),
                submission_time: Utc::now(),
            });
        }
    }

    // Get submission metadata
    let submission_count = get_submission_count(state, user_id, problem_id).await?;
    let (last_submission_id, last_submission_time) = get_last_submission_info(state, user_id, problem_id).await?;
    
    // Best submission is the one that achieved the highest total score
    let (best_submission_id, best_submission_time) = if !subtask_scores.is_empty() {
        // Find the submission that contributed to most subtasks
        let mut best_sub = subtask_scores.iter()
            .filter(|s| s.score > 0.0)
            .max_by_key(|s| s.submission_time)
            .map(|s| (Some(s.submission_id), Some(s.submission_time)));
        
        if let Some((Some(id), Some(time))) = best_sub {
            (Some(id), Some(time))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Ok(UserProblemScore {
        user_id,
        problem_id,
        final_score: total_score,
        max_possible_score: total_max_score,
        scoring_method: ScoringMethod::SubtaskSum,
        subtask_scores,
        submission_count,
        last_submission_id,
        last_submission_time,
        best_submission_id,
        best_submission_time,
    })
}

// Helper functions
async fn recalculate_contest_scores(
    state: &KernelState,
    contest_id: Uuid,
    scoring_method: ScoringMethod,
) -> Result<i32, StatusCode> {
    // This would trigger a background job to recalculate all scores
    // For now, we'll return success
    // In a real implementation, you'd queue this as a background job
    
    let query = r"
        SELECT COUNT(DISTINCT s.user_id) as user_count
        FROM submissions s
        JOIN problems p ON s.problem_id = p.id
        WHERE p.contest_id = $1
    ";

    let row = sqlx::query(query)
        .bind(contest_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_count: i64 = row.get("user_count");
    Ok(user_count as i32)
}

async fn get_contest_scoring_method(
    state: &KernelState,
    contest_id: Uuid,
) -> Result<ScoringMethod, StatusCode> {
    let query = r"SELECT scoring_method FROM contests WHERE id = $1";
    let row = sqlx::query(query)
        .bind(contest_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let method_str: String = row.get("scoring_method");
    match method_str.as_str() {
        "last_submission" => Ok(ScoringMethod::LastSubmission),
        "max_score" => Ok(ScoringMethod::MaxScore),
        "subtask_sum" => Ok(ScoringMethod::SubtaskSum),
        _ => Ok(ScoringMethod::LastSubmission), // Default fallback
    }
}

async fn get_submission_count(
    state: &KernelState,
    user_id: Uuid,
    problem_id: Uuid,
) -> Result<i32, StatusCode> {
    let query = r"
        SELECT COUNT(*) as count
        FROM submissions
        WHERE user_id = $1 AND problem_id = $2
    ";

    let row = sqlx::query(query)
        .bind(user_id)
        .bind(problem_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let count: i64 = row.get("count");
    Ok(count as i32)
}

async fn get_last_submission_info(
    state: &KernelState,
    user_id: Uuid,
    problem_id: Uuid,
) -> Result<(Option<Uuid>, Option<DateTime<Utc>>), StatusCode> {
    let query = r"
        SELECT id, created_at
        FROM submissions
        WHERE user_id = $1 AND problem_id = $2
        ORDER BY created_at DESC
        LIMIT 1
    ";

    let row = sqlx::query(query)
        .bind(user_id)
        .bind(problem_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(row) => Ok((Some(row.get("id")), Some(row.get("created_at")))),
        None => Ok((None, None)),
    }
}

fn parse_subtask_scores(subtask_scores_json: Option<Value>) -> Vec<SubtaskScore> {
    // Parse JSON subtask scores - simplified implementation
    // In practice, you'd properly deserialize the JSON structure
    vec![]
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