use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use uuid::Uuid;
use shared::*;
use crate::AppState;

/// Assign a user as contest admin
pub async fn assign_contest_admin(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<AssignContestAdminRequest>,
) -> Result<Json<AssignContestAdminResponse>, StatusCode> {
    // Check if the requesting user has permission (is an admin, superadmin, or contest creator)
    let has_permission = user.roles.contains(&"admin".to_string()) 
        || user.roles.contains(&"superadmin".to_string());

    if !has_permission {
        // Check if user is the contest creator
        let contest = state.db.get_contest(req.contest_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;
        
        if contest.created_by != user.id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Verify the target user exists
    let _target_user = state.db.get_user_by_id(req.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verify the contest exists
    let _contest = state.db.get_contest(req.contest_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Assign the contest admin
    let contest_admin = state.db.assign_contest_admin(req.contest_id, req.user_id)
        .await
        .map_err(|e| {
            // Handle duplicate assignment (unique constraint violation)
            if e.to_string().contains("duplicate key") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(Json(AssignContestAdminResponse {
        contest_admin_id: contest_admin.id,
        message: "Contest admin assigned successfully".to_string(),
    }))
}

/// Remove a contest admin assignment
pub async fn remove_contest_admin(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((contest_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    // Check if the requesting user has permission
    let has_permission = user.roles.contains(&"admin".to_string()) 
        || user.roles.contains(&"superadmin".to_string());

    if !has_permission {
        // Check if user is the contest creator
        let contest = state.db.get_contest(contest_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;
        
        if contest.created_by != user.id {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let removed = state.db.remove_contest_admin(contest_id, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// List all admins for a contest
pub async fn list_contest_admins(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(contest_id): Path<Uuid>,
) -> Result<Json<ContestAdminListResponse>, StatusCode> {
    // Verify the contest exists
    let _contest = state.db.get_contest(contest_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if the requesting user has permission to view contest admins
    let has_permission = user.roles.contains(&"admin".to_string()) 
        || user.roles.contains(&"superadmin".to_string())
        || state.db.is_contest_admin(contest_id, user.id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }

    let contest_admins = state.db.list_contest_admins(contest_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ContestAdminListResponse { contest_admins }))
}

/// Get contests administered by the current user
pub async fn get_administered_contests(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<Contest>>, StatusCode> {
    let contests = state.db.get_user_administered_contests(user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(contests))
}

/// Check if a user is a contest admin
pub async fn check_contest_admin(
    State(state): State<AppState>,
    Extension(requesting_user): Extension<User>,
    Path((contest_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<bool>, StatusCode> {
    // Check if requesting user has permission
    let has_permission = requesting_user.roles.contains(&"admin".to_string()) 
        || requesting_user.roles.contains(&"superadmin".to_string())
        || requesting_user.id == user_id  // Users can check their own admin status
        || state.db.is_contest_admin(contest_id, requesting_user.id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !has_permission {
        return Err(StatusCode::FORBIDDEN);
    }

    let is_admin = state.db.is_contest_admin(contest_id, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(is_admin))
}
