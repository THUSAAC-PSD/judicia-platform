use axum::{extract::State, http::StatusCode, Json, Extension};
use bcrypt::{hash, verify, DEFAULT_COST};
use shared::*;
use serde_json::json;

use crate::{auth::create_jwt, KernelState};

pub async fn register(
    State(state): State<KernelState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Check if user already exists
    if state
        .db
        .get_user_by_username(&payload.username)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Database error"}))))?
        .is_some()
    {
        return Err((StatusCode::CONFLICT, Json(json!({"message":"Username already taken"}))));
    }

    // Check if email already exists
    if state
        .db
        .get_user_by_email(&payload.email)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Database error"}))))?
        .is_some()
    {
        return Err((StatusCode::CONFLICT, Json(json!({"message":"Email already registered"}))));
    }

    // Hash password
    let hashed_password = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Failed to hash password"}))))?;

    // Public registration always creates a contestant account
    let roles = vec!["contestant".to_string()];

    // Create user with specified roles
    let user = state
        .db
        .create_user_with_roles(&payload.username, &payload.email, &hashed_password, roles)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Failed to create user"}))))?;

    // Create JWT token
    let token = create_jwt(user.id, &state.config.jwt_secret)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Failed to create token"}))))?;

    let user_profile = UserProfile {
        id: user.id,
        username: user.username,
        email: user.email,
        roles: user.roles,
    };

    Ok(Json(AuthResponse {
        token,
        user: user_profile,
    }))
}

pub async fn register_admin(
    State(state): State<KernelState>,
    Extension(requesting_user): Extension<User>,
    Json(payload): Json<AdminRegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Only superadmin can create admin/superadmin users
    if !requesting_user.roles.contains(&"superadmin".to_string()) {
        return Err((StatusCode::FORBIDDEN, Json(json!({"message":"Superadmin access required"}))));
    }
    // Validate admin type
    if payload.admin_type != "admin" && payload.admin_type != "superadmin" {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"message":"Invalid admin type"}))));
    }

    // Check if user already exists
    if state
        .db
        .get_user_by_username(&payload.username)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Database error"}))))?
        .is_some()
    {
        return Err((StatusCode::CONFLICT, Json(json!({"message":"Username already taken"}))));
    }

    // Check if email already exists
    if state
        .db
        .get_user_by_email(&payload.email)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Database error"}))))?
        .is_some()
    {
        return Err((StatusCode::CONFLICT, Json(json!({"message":"Email already registered"}))));
    }

    // Hash password
    let hashed_password = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Failed to hash password"}))))?;

    // Create admin user with requested admin_type
    let roles = vec![payload.admin_type];
    let user = state
        .db
        .create_user_with_roles(&payload.username, &payload.email, &hashed_password, roles)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Failed to create user"}))))?;

    // Create JWT token
    let token = create_jwt(user.id, &state.config.jwt_secret)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Failed to create token"}))))?;

    let user_profile = UserProfile {
        id: user.id,
        username: user.username,
        email: user.email,
        roles: user.roles,
    };

    Ok(Json(AuthResponse {
        token,
        user: user_profile,
    }))
}

pub async fn login(
    State(state): State<KernelState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Get user by email
    let user = state
        .db
        .get_user_by_email(&payload.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify password
    if !verify(&payload.password, &user.hashed_password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Create JWT token
    let token = create_jwt(user.id, &state.config.jwt_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_profile = UserProfile {
        id: user.id,
        username: user.username,
        email: user.email,
        roles: user.roles,
    };

    Ok(Json(AuthResponse {
        token,
        user: user_profile,
    }))
}
pub async fn me(
    Extension(user): Extension<User>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user_profile = UserProfile {
        id: user.id,
        username: user.username,
        email: user.email,
        roles: user.roles,
    };

    Ok(Json(UserResponse {
        user: user_profile,
    }))
}

#[derive(serde::Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<KernelState>,
    Extension(user): Extension<User>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if payload.new_password.len() < 6 {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"message":"New password must be at least 6 characters"}))));
    }

    let is_valid = verify(&payload.current_password, &user.hashed_password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Internal error"}))))?;
    if !is_valid {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"message":"Current password is incorrect"}))));
    }

    let hashed = hash(&payload.new_password, DEFAULT_COST)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Failed to hash password"}))))?;

    state
        .db
        .update_user_password(user.id, &hashed)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message":"Failed to update password"}))))?;

    Ok(Json(json!({"message":"Password updated successfully"})))
}
