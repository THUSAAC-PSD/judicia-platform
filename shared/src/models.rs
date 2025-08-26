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

// Plugin system models
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Plugin {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub plugin_type: String, // 'contest', 'problem', 'utility'
    pub wasm_path: String,
    pub config_schema: serde_json::Value,
    pub capabilities: Vec<String>,
    pub status: String, // 'installed', 'active', 'disabled', 'error'
    pub installed_at: DateTime<Utc>,
    pub last_loaded_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PluginPermission {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub capability: String,
    pub database_access_level: String, // 'none', 'read_only', 'read_write', 'schema_admin'
    pub rate_limit_requests_per_second: i32,
    pub rate_limit_db_queries_per_minute: i32,
    pub rate_limit_events_per_minute: i32,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PluginHttpRoute {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub path: String,
    pub method: String,
    pub handler_function: String,
    pub required_permission: Option<String>,
    pub rate_limit_override: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PluginUiRoute {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub scope: String, // 'contest', 'problem', 'admin', 'global'
    pub path: String,
    pub component: String,
    pub required_permission: Option<String>,
    pub nav_link: bool,
    pub nav_text: Option<String>,
    pub nav_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkerNode {
    pub id: Uuid,
    pub node_id: String,
    pub host_address: String,
    pub port: i32,
    pub capabilities: Vec<String>,
    pub max_concurrent_jobs: i32,
    pub current_load: i32,
    pub status: String, // 'online', 'offline', 'maintenance', 'overloaded'
    pub last_heartbeat: DateTime<Utc>,
    pub registered_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub source_plugin_id: Option<Uuid>,
    pub source_user_id: Option<Uuid>,
    pub source_contest_id: Option<Uuid>,
    pub source_submission_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub processed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JudgingQueueItem {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub claimed_by: Option<String>,
    pub max_retries: i32,
    pub retry_count: i32,
    pub status: String, // 'pending', 'claimed', 'completed', 'failed', 'retrying'
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
}