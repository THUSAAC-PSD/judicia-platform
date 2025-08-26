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
use sqlx::Row;

use crate::KernelState;
use shared::User;

// Helper function to check if user is admin
fn is_admin_user(user: &User) -> bool {
    user.roles.contains(&"admin".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContestExtension {
    pub id: Uuid,
    pub contest_id: Uuid,
    pub user_id: Uuid,
    pub extra_minutes: i32,
    pub reason: String,
    pub granted_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateExtensionRequest {
    pub user_id: Option<Uuid>, // None = apply to all users in contest
    pub extra_minutes: i32,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateExtensionRequest {
    pub extra_minutes: i32,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ExtensionQuery {
    pub user_id: Option<Uuid>,
}

/// Create a time extension for a contest
pub async fn create_contest_extension(
    State(state): State<KernelState>,
    Extension(admin_user): Extension<User>,
    Path(contest_id): Path<Uuid>,
    Json(request): Json<CreateExtensionRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Verify admin permissions
    if !is_admin_user(&admin_user) && !is_contest_admin(&state, &admin_user.id, &contest_id).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    // Validate contest exists
    if !contest_exists(&state, &contest_id).await? {
        return Err(StatusCode::NOT_FOUND);
    }

    match request.user_id {
        Some(user_id) => {
            // Individual user extension
            create_individual_extension(&state, contest_id, user_id, &request, &admin_user).await
        }
        None => {
            // Contest-wide extension (update contest table)
            create_contest_wide_extension(&state, contest_id, &request, &admin_user).await
        }
    }
}

/// Get all extensions for a contest
pub async fn get_contest_extensions(
    State(state): State<KernelState>,
    Extension(user): Extension<User>,
    Path(contest_id): Path<Uuid>,
    Query(query): Query<ExtensionQuery>,
) -> Result<Json<Value>, StatusCode> {
    // Verify permissions
    if !is_admin_user(&user) && !is_contest_admin(&state, &user.id, &contest_id).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let extensions = get_extensions_from_db(&state, contest_id, query.user_id).await?;
    let contest_extension = get_contest_wide_extension(&state, contest_id).await?;

    Ok(Json(json!({
        "individual_extensions": extensions,
        "contest_wide_extension": contest_extension,
        "total_count": extensions.len()
    })))
}

/// Update an existing extension
pub async fn update_contest_extension(
    State(state): State<KernelState>,
    Extension(admin_user): Extension<User>,
    Path((contest_id, extension_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateExtensionRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Verify admin permissions
    if !is_admin_user(&admin_user) && !is_contest_admin(&state, &admin_user.id, &contest_id).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let query = r"
        UPDATE contest_extensions 
        SET extra_minutes = $1, reason = $2, updated_at = NOW()
        WHERE id = $3 AND contest_id = $4
        RETURNING id, contest_id, user_id, extra_minutes, reason, granted_by, created_at, updated_at
    ";

    let row = sqlx::query(query)
        .bind(request.extra_minutes)
        .bind(&request.reason)
        .bind(extension_id)
        .bind(contest_id)
        .fetch_optional(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(row) => {
            let extension = ContestExtension {
                id: row.get("id"),
                contest_id: row.get("contest_id"),
                user_id: row.get("user_id"),
                extra_minutes: row.get("extra_minutes"),
                reason: row.get("reason"),
                granted_by: row.get("granted_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };

            Ok(Json(json!({
                "success": true,
                "extension": extension
            })))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete an extension
pub async fn delete_contest_extension(
    State(state): State<KernelState>,
    Extension(admin_user): Extension<User>,
    Path((contest_id, extension_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, StatusCode> {
    // Verify admin permissions
    if !is_admin_user(&admin_user) && !is_contest_admin(&state, &admin_user.id, &contest_id).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let query = r"DELETE FROM contest_extensions WHERE id = $1 AND contest_id = $2";
    
    let result = sqlx::query(query)
        .bind(extension_id)
        .bind(contest_id)
        .execute(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() > 0 {
        Ok(Json(json!({
            "success": true,
            "message": "Extension deleted successfully"
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Get effective contest time for a user (including extensions)
pub async fn get_user_contest_time(
    State(state): State<KernelState>,
    Extension(user): Extension<User>,
    Path((contest_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, StatusCode> {
    // Users can query their own time, admins can query anyone
    if user.id != user_id && !is_admin_user(&user) && !is_contest_admin(&state, &user.id, &contest_id).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let contest_time = get_effective_contest_time(&state, contest_id, user_id).await?;
    
    Ok(Json(json!({
        "contest_id": contest_id,
        "user_id": user_id,
        "original_start_time": contest_time.original_start,
        "original_end_time": contest_time.original_end,
        "effective_end_time": contest_time.effective_end,
        "total_extra_minutes": contest_time.total_extra_minutes,
        "individual_extra_minutes": contest_time.individual_extra_minutes,
        "contest_wide_extra_minutes": contest_time.contest_wide_extra_minutes
    })))
}

// Helper functions
async fn create_individual_extension(
    state: &KernelState,
    contest_id: Uuid,
    user_id: Uuid,
    request: &CreateExtensionRequest,
    admin_user: &User,
) -> Result<Json<Value>, StatusCode> {
    // Verify user exists and is in contest
    if !user_exists(&state, &user_id).await? {
        return Err(StatusCode::BAD_REQUEST);
    }

    let query = r"
        INSERT INTO contest_extensions (contest_id, user_id, extra_minutes, reason, granted_by)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (contest_id, user_id)
        DO UPDATE SET extra_minutes = $3, reason = $4, granted_by = $5, updated_at = NOW()
        RETURNING id, contest_id, user_id, extra_minutes, reason, granted_by, created_at, updated_at
    ";

    let row = sqlx::query(query)
        .bind(contest_id)
        .bind(user_id)
        .bind(request.extra_minutes)
        .bind(&request.reason)
        .bind(admin_user.id)
        .fetch_one(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let extension = ContestExtension {
        id: row.get("id"),
        contest_id: row.get("contest_id"),
        user_id: row.get("user_id"),
        extra_minutes: row.get("extra_minutes"),
        reason: row.get("reason"),
        granted_by: row.get("granted_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(json!({
        "success": true,
        "extension": extension,
        "type": "individual"
    })))
}

async fn create_contest_wide_extension(
    state: &KernelState,
    contest_id: Uuid,
    request: &CreateExtensionRequest,
    _admin_user: &User,
) -> Result<Json<Value>, StatusCode> {
    let query = r"
        UPDATE contests 
        SET extra_time_minutes = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, title, extra_time_minutes, start_time, end_time
    ";

    let row = sqlx::query(query)
        .bind(request.extra_minutes)
        .bind(contest_id)
        .fetch_one(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "success": true,
        "contest_id": contest_id,
        "extra_minutes": request.extra_minutes,
        "reason": request.reason,
        "type": "contest_wide",
        "contest": {
            "id": row.get::<Uuid, _>("id"),
            "title": row.get::<String, _>("title"),
            "extra_time_minutes": row.get::<i32, _>("extra_time_minutes"),
            "start_time": row.get::<DateTime<Utc>, _>("start_time"),
            "end_time": row.get::<DateTime<Utc>, _>("end_time")
        }
    })))
}

#[derive(Debug)]
pub struct EffectiveContestTime {
    pub original_start: DateTime<Utc>,
    pub original_end: DateTime<Utc>,
    pub effective_end: DateTime<Utc>,
    pub total_extra_minutes: i32,
    pub individual_extra_minutes: i32,
    pub contest_wide_extra_minutes: i32,
}

async fn get_effective_contest_time(
    state: &KernelState,
    contest_id: Uuid,
    user_id: Uuid,
) -> Result<EffectiveContestTime, StatusCode> {
    // Get contest info
    let contest_query = r"SELECT start_time, end_time, extra_time_minutes FROM contests WHERE id = $1";
    let contest_row = sqlx::query(contest_query)
        .bind(contest_id)
        .fetch_one(state.db.pool())
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let original_start: DateTime<Utc> = contest_row.get("start_time");
    let original_end: DateTime<Utc> = contest_row.get("end_time");
    let contest_wide_extra: i32 = contest_row.get("extra_time_minutes");

    // Get individual extension
    let ext_query = r"SELECT extra_minutes FROM contest_extensions WHERE contest_id = $1 AND user_id = $2";
    let individual_extra = sqlx::query(ext_query)
        .bind(contest_id)
        .bind(user_id)
        .fetch_optional(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|row| row.get::<i32, _>("extra_minutes"))
        .unwrap_or(0);

    let total_extra = contest_wide_extra + individual_extra;
    let effective_end = original_end + chrono::Duration::minutes(total_extra as i64);

    Ok(EffectiveContestTime {
        original_start,
        original_end,
        effective_end,
        total_extra_minutes: total_extra,
        individual_extra_minutes: individual_extra,
        contest_wide_extra_minutes: contest_wide_extra,
    })
}

async fn get_extensions_from_db(
    state: &KernelState,
    contest_id: Uuid,
    user_id: Option<Uuid>,
) -> Result<Vec<ContestExtension>, StatusCode> {
    let mut query = r"
        SELECT ce.id, ce.contest_id, ce.user_id, ce.extra_minutes, ce.reason, 
               ce.granted_by, ce.created_at, ce.updated_at,
               u.username, u.display_name
        FROM contest_extensions ce
        JOIN users u ON ce.user_id = u.id
        WHERE ce.contest_id = $1
    ".to_string();

    let mut params = vec![contest_id.to_string()];
    
    if let Some(uid) = user_id {
        query.push_str(" AND ce.user_id = $2");
        params.push(uid.to_string());
    }
    
    query.push_str(" ORDER BY ce.created_at DESC");

    let rows = sqlx::query(&query)
        .bind(contest_id)
        .fetch_all(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let extensions = rows
        .into_iter()
        .map(|row| ContestExtension {
            id: row.get("id"),
            contest_id: row.get("contest_id"),
            user_id: row.get("user_id"),
            extra_minutes: row.get("extra_minutes"),
            reason: row.get("reason"),
            granted_by: row.get("granted_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect();

    Ok(extensions)
}

async fn get_contest_wide_extension(
    state: &KernelState,
    contest_id: Uuid,
) -> Result<i32, StatusCode> {
    let query = r"SELECT extra_time_minutes FROM contests WHERE id = $1";
    let row = sqlx::query(query)
        .bind(contest_id)
        .fetch_one(state.db.pool())
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(row.get("extra_time_minutes"))
}

// Utility functions
async fn contest_exists(state: &KernelState, contest_id: &Uuid) -> Result<bool, StatusCode> {
    let query = r"SELECT 1 FROM contests WHERE id = $1";
    let exists = sqlx::query(query)
        .bind(contest_id)
        .fetch_optional(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    
    Ok(exists)
}

async fn user_exists(state: &KernelState, user_id: &Uuid) -> Result<bool, StatusCode> {
    let query = r"SELECT 1 FROM users WHERE id = $1";
    let exists = sqlx::query(query)
        .bind(user_id)
        .fetch_optional(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    
    Ok(exists)
}

async fn is_contest_admin(state: &KernelState, user_id: &Uuid, contest_id: &Uuid) -> Result<bool, StatusCode> {
    let query = r"SELECT 1 FROM contest_admins WHERE user_id = $1 AND contest_id = $2";
    let is_admin = sqlx::query(query)
        .bind(user_id)
        .bind(contest_id)
        .fetch_optional(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    
    Ok(is_admin)
}