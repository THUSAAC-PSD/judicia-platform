use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Problem {
    pub id: Uuid,
    pub title: String,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub statement: String, // Markdown
    pub difficulty: String,
    pub time_limit_ms: i32,
    pub memory_limit_kb: i32,
    pub question_type_id: Uuid,
    pub metadata: serde_json::Value, // JSONB for extensibility
    pub points: i32,
    pub contest_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuestionTypeModel {
    pub id: Uuid,
    pub name: String, // e.g., "ioi-standard", "output-only"
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Language {
    pub id: Uuid,
    pub name: String, // e.g., "C++17", "Python 3.9"
    pub version: String,
    pub compile_command: Option<String>,
    pub run_command: String,
    pub file_extension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Submission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub problem_id: Uuid,
    pub language_id: Uuid,
    pub source_code: String,
    pub submitted_at: DateTime<Utc>,
    pub status: String, // Maps to JudgeStatus
    pub verdict: Option<String>, // Maps to Verdict
    pub execution_time_ms: Option<i32>,
    pub execution_memory_kb: Option<i32>,
    pub contest_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestCase {
    pub id: Uuid,
    pub problem_id: Uuid,
    pub input_data: String,
    pub output_data: String,
    pub is_sample: bool,
    pub order_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubmissionResult {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub test_case_id: Uuid,
    pub verdict: String, // Maps to Verdict
    pub execution_time_ms: Option<i32>,
    pub execution_memory_kb: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Contest {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration: i32, // seconds
    pub created_by: Uuid,
    pub participant_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContestAdmin {
    pub id: Uuid,
    pub contest_id: Uuid,
    pub user_id: Uuid,
    pub assigned_at: DateTime<Utc>,
}

// API Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<String>, // Optional role: "admin", "contest_admin", or defaults to "contestant"
}

#[derive(Debug, Deserialize)]
pub struct AdminRegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub admin_type: String, // "admin" or "superadmin"
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserProfile,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub user: UserProfile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmissionRequest {
    pub problem_id: Uuid,
    pub language_id: Uuid,
    pub source_code: String,
}

#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    pub submission_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateContestRequest {
    pub title: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub duration: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateProblemRequest {
    pub title: String,
    pub statement: String,
    pub difficulty: String,
    pub time_limit_ms: i32,
    pub memory_limit_kb: i32,
    pub question_type_id: Uuid,
    pub metadata: serde_json::Value,
    pub points: i32,
    pub contest_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AssignContestAdminRequest {
    pub contest_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct AssignContestAdminResponse {
    pub contest_admin_id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ContestAdminListResponse {
    pub contest_admins: Vec<ContestAdminWithUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContestAdminWithUser {
    pub id: Uuid,
    pub contest_id: Uuid,
    pub user_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub username: String,
    pub email: String,
}