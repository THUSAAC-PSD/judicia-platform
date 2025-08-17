use axum::{extract::State, http::StatusCode, Json, Extension};
use bcrypt::{hash, verify, DEFAULT_COST};
use shared::*;

use crate::{auth::create_jwt, AppState};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Check if user already exists
    if state
        .db
        .get_user_by_username(&payload.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    // Check if email already exists
    if state
        .db
        .get_user_by_email(&payload.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    // Hash password
    let hashed_password = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Determine user roles
    let roles = match payload.role.as_deref() {
        Some("admin") => vec!["admin".to_string()],
        Some("contest_admin") => vec!["contest_admin".to_string()],
        Some("superadmin") => vec!["superadmin".to_string()], // Allow superadmin creation
        _ => vec!["contestant".to_string()], // Default role
    };

    // Create user with specified roles
    let user = state
        .db
        .create_user_with_roles(&payload.username, &payload.email, &hashed_password, roles)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

pub async fn register_admin(
    State(state): State<AppState>,
    Json(payload): Json<AdminRegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Validate admin type
    if payload.admin_type != "admin" && payload.admin_type != "superadmin" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if user already exists
    if state
        .db
        .get_user_by_username(&payload.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    // Check if email already exists
    if state
        .db
        .get_user_by_email(&payload.email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    // Hash password
    let hashed_password = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create admin user
    let roles = vec![payload.admin_type];
    let user = state
        .db
        .create_user_with_roles(&payload.username, &payload.email, &hashed_password, roles)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

pub async fn login(
    State(state): State<AppState>,
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
