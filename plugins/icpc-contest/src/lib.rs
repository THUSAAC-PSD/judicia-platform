//! ICPC Contest Plugin for Judicia Platform
//!
//! This plugin provides comprehensive ICPC-style contest management including:
//! - Contest registration and team management
//! - Real-time scoreboard with ICPC ranking rules
//! - Balloon notifications for first solves
//! - Clarification system
//! - Freeze/unfreeze functionality
//! - Contest time management and penalties
//! - Team standings and problem statistics

use judicia_sdk::prelude::*;
use judicia_sdk::{
    HttpRequest, HttpResponse, PluginError, PluginResult, Notification, NotificationUrgency, NotificationType,
    Plugin, PluginInfo, database_query, register_http_route, send_notification, emit_platform_event
};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use indexmap::IndexMap;

/// ICPC Contest Plugin
pub struct IcpcContestPlugin {
    // Plugin state
    contest_cache: HashMap<Uuid, ContestData>,
    scoreboard_cache: HashMap<Uuid, ScoreboardData>,
}

impl Plugin for IcpcContestPlugin {
    fn new() -> Self {
        Self {
            contest_cache: HashMap::new(),
            scoreboard_cache: HashMap::new(),
        }
    }
    
    fn metadata(&self) -> PluginInfo {
        PluginInfo {
            name: "icpc-contest".to_string(),
            version: "1.0.0".to_string(),
            author: "Judicia Platform Team".to_string(),
            description: "ICPC-style contest management with real-time scoreboard and team management".to_string(),
            capabilities: vec![
                "ReadContests".to_string(),
                "WriteContests".to_string(),
                "ReadProblems".to_string(),
                "ReadSubmissions".to_string(),
                "WriteSubmissions".to_string(),
                "ReadUsers".to_string(),
                "EmitEvent".to_string(),
                "SubscribeEvents".to_string(),
                "RegisterComponents".to_string(),
                "RegisterRoutes".to_string(),
                "SendNotifications".to_string(),
                "FileStorage".to_string(),
                "AccessDatabase".to_string(),
            ],
            dependencies: vec![],
            frontend_components: vec![
                "ScoreBoard".to_string(),
                "ContestDashboard".to_string(),
                "TeamRegistration".to_string(),
                "ClarificationSystem".to_string(),
                "ContestTimer".to_string(),
                "ProblemStatistics".to_string(),
                "BalloonQueue".to_string(),
                "ContestAdmin".to_string(),
            ],
            api_routes: vec![
                "/api/icpc/contests".to_string(),
                "/api/icpc/contests/{id}".to_string(),
                "/api/icpc/contests/{id}/scoreboard".to_string(),
                "/api/icpc/contests/{id}/teams".to_string(),
                "/api/icpc/contests/{id}/register".to_string(),
                "/api/icpc/contests/{id}/clarifications".to_string(),
                "/api/icpc/contests/{id}/freeze".to_string(),
                "/api/icpc/contests/{id}/unfreeze".to_string(),
                "/api/icpc/contests/{id}/balloons".to_string(),
                "/api/icpc/contests/{id}/statistics".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContestData {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub freeze_time: Option<DateTime<Utc>>,
    pub unfreeze_time: Option<DateTime<Utc>>,
    pub is_frozen: bool,
    pub penalty_minutes: i32,
    pub problems: Vec<ContestProblem>,
    pub teams: Vec<TeamData>,
    pub config: IcpcConfig,
    pub status: ContestStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContestProblem {
    pub id: Uuid,
    pub letter: String,
    pub title: String,
    pub color: String,
    pub first_solve_team: Option<Uuid>,
    pub first_solve_time: Option<DateTime<Utc>>,
    pub solve_count: i32,
    pub attempt_count: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamData {
    pub id: Uuid,
    pub name: String,
    pub organization: String,
    pub members: Vec<String>,
    pub coach: Option<String>,
    pub registration_time: DateTime<Utc>,
    pub is_hidden: bool,
    pub rank: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IcpcConfig {
    pub max_teams: Option<i32>,
    pub problems_per_page: i32,
    pub enable_clarifications: bool,
    pub enable_balloons: bool,
    pub auto_freeze_duration_minutes: Option<i32>,
    pub show_pending_submissions: bool,
    pub penalty_per_wrong_submission: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ContestStatus {
    NotStarted,
    Running,
    Frozen,
    Finished,
    Cancelled,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScoreboardData {
    pub contest_id: Uuid,
    pub last_updated: DateTime<Utc>,
    pub is_frozen: bool,
    pub standings: Vec<TeamStanding>,
    pub problem_stats: Vec<ProblemStats>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamStanding {
    pub rank: i32,
    pub team_id: Uuid,
    pub team_name: String,
    pub organization: String,
    pub solved_count: i32,
    pub total_time: i32, // in minutes
    pub problem_results: IndexMap<String, ProblemResult>,
    pub last_solve_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProblemResult {
    pub problem_letter: String,
    pub status: ProblemStatus,
    pub attempts: i32,
    pub solve_time_minutes: Option<i32>,
    pub penalty_time: i32,
    pub is_first_solve: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProblemStatus {
    NotAttempted,
    Attempted,
    Solved,
    FirstSolve,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProblemStats {
    pub problem_letter: String,
    pub problem_title: String,
    pub total_attempts: i32,
    pub total_solves: i32,
    pub first_solve_team: Option<String>,
    pub first_solve_time: Option<DateTime<Utc>>,
    pub solve_percentage: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClarificationRequest {
    pub id: Uuid,
    pub contest_id: Uuid,
    pub team_id: Uuid,
    pub problem_letter: Option<String>,
    pub question: String,
    pub answer: Option<String>,
    pub is_public: bool,
    pub timestamp: DateTime<Utc>,
    pub answered_by: Option<Uuid>,
    pub answered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BalloonDelivery {
    pub id: Uuid,
    pub contest_id: Uuid,
    pub team_id: Uuid,
    pub problem_letter: String,
    pub color: String,
    pub solve_time: DateTime<Utc>,
    pub delivered: bool,
    pub delivered_at: Option<DateTime<Utc>>,
    pub delivered_by: Option<String>,
}

#[async_trait(?Send)]
impl PluginMethods for IcpcContestPlugin {
    async fn on_initialize(&mut self, context: &PluginContext) -> PluginResult<()> {
        info!("ICPC Contest plugin initialized for context: {:?}", context.plugin_id);
        
        // Register contest management components
        self.register_components().await?;
        
        // Register HTTP routes
        self.register_routes().await?;
        
        // Subscribe to relevant events
        self.setup_event_listeners().await?;
        
        // Load active contests into cache
        self.load_active_contests().await?;
        
        Ok(())
    }

    async fn on_event(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        match event.event_type.as_str() {
            "submission.created" => self.handle_submission_created(event).await?,
            "judging.completed" => self.handle_judging_completed(event).await?,
            "contest.created" => self.handle_contest_created(event).await?,
            "contest.updated" => self.handle_contest_updated(event).await?,
            "contest.started" => self.handle_contest_started(event).await?,
            "contest.finished" => self.handle_contest_finished(event).await?,
            "team.registered" => self.handle_team_registered(event).await?,
            _ => debug!("Unhandled event: {}", event.event_type),
        }
        Ok(())
    }

    async fn on_cleanup(&mut self) -> PluginResult<()> {
        info!("ICPC Contest plugin cleaning up");
        self.contest_cache.clear();
        self.scoreboard_cache.clear();
        Ok(())
    }

    async fn on_http_request(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        match request.method.as_str() {
            "GET" => match request.path.as_str() {
                "/api/icpc/contests" => self.handle_list_contests(request).await,
                path if path.starts_with("/api/icpc/contests/") => {
                    let parts: Vec<&str> = path.split('/').collect();
                    if parts.len() >= 5 {
                        let contest_id = parts[4];
                        match parts.get(5) {
                            Some(&"scoreboard") => self.handle_get_scoreboard(request, contest_id).await,
                            Some(&"teams") => self.handle_get_teams(request, contest_id).await,
                            Some(&"clarifications") => self.handle_get_clarifications(request, contest_id).await,
                            Some(&"balloons") => self.handle_get_balloons(request, contest_id).await,
                            Some(&"statistics") => self.handle_get_statistics(request, contest_id).await,
                            None => self.handle_get_contest(request, contest_id).await,
                            _ => Err(PluginError::NotImplemented("Route not found".into())),
                        }
                    } else {
                        Err(PluginError::InvalidInput("Invalid contest path".into()))
                    }
                },
                _ => Err(PluginError::NotImplemented("Route not found".into())),
            },
            "POST" => match request.path.as_str() {
                "/api/icpc/contests" => self.handle_create_contest(request).await,
                path if path.contains("/register") => self.handle_team_registration(request).await,
                path if path.contains("/clarifications") => self.handle_create_clarification(request).await,
                path if path.contains("/freeze") => self.handle_freeze_contest(request).await,
                path if path.contains("/unfreeze") => self.handle_unfreeze_contest(request).await,
                _ => Err(PluginError::NotImplemented("Route not found".into())),
            },
            "PUT" => {
                if request.path.contains("/balloons/") {
                    self.handle_mark_balloon_delivered(request).await
                } else {
                    Err(PluginError::NotImplemented("Route not found".into()))
                }
            },
            _ => Err(PluginError::NotImplemented("Method not supported".into())),
        }
    }

    async fn on_render(&self, component: &str, props: &serde_json::Value) -> PluginResult<String> {
        match component {
            "ScoreBoard" => self.render_scoreboard(props).await,
            "ContestDashboard" => self.render_contest_dashboard(props).await,
            "TeamRegistration" => self.render_team_registration(props).await,
            "ClarificationSystem" => self.render_clarification_system(props).await,
            "ContestTimer" => self.render_contest_timer(props).await,
            "ProblemStatistics" => self.render_problem_statistics(props).await,
            "BalloonQueue" => self.render_balloon_queue(props).await,
            "ContestAdmin" => self.render_contest_admin(props).await,
            _ => Err(PluginError::NotImplemented(format!("Component '{}' not found", component))),
        }
    }
}

impl IcpcContestPlugin {
    async fn register_components(&mut self) -> PluginResult<()> {
        info!("Registering ICPC contest components");
        Ok(())
    }

    async fn register_routes(&mut self) -> PluginResult<()> {
        info!("Registering ICPC contest routes");
        
        // Contest management routes
        register_http_route("GET", "/api/icpc/contests", "handle_list_contests").await?;
        register_http_route("POST", "/api/icpc/contests", "handle_create_contest").await?;
        register_http_route("GET", "/api/icpc/contests/{id}", "handle_get_contest").await?;
        
        // Contest-specific routes
        register_http_route("GET", "/api/icpc/contests/{id}/scoreboard", "handle_get_scoreboard").await?;
        register_http_route("GET", "/api/icpc/contests/{id}/teams", "handle_get_teams").await?;
        register_http_route("POST", "/api/icpc/contests/{id}/register", "handle_team_registration").await?;
        register_http_route("GET", "/api/icpc/contests/{id}/clarifications", "handle_get_clarifications").await?;
        register_http_route("POST", "/api/icpc/contests/{id}/clarifications", "handle_create_clarification").await?;
        register_http_route("POST", "/api/icpc/contests/{id}/freeze", "handle_freeze_contest").await?;
        register_http_route("POST", "/api/icpc/contests/{id}/unfreeze", "handle_unfreeze_contest").await?;
        register_http_route("GET", "/api/icpc/contests/{id}/balloons", "handle_get_balloons").await?;
        register_http_route("PUT", "/api/icpc/contests/{id}/balloons/{balloon_id}", "handle_mark_balloon_delivered").await?;
        register_http_route("GET", "/api/icpc/contests/{id}/statistics", "handle_get_statistics").await?;
        
        Ok(())
    }

    async fn setup_event_listeners(&mut self) -> PluginResult<()> {
        info!("Setting up ICPC contest event listeners");
        Ok(())
    }

    async fn load_active_contests(&mut self) -> PluginResult<()> {
        info!("Loading active contests into cache");
        
        let query = DatabaseQuery {
            query: r"
                SELECT id, title, description, start_time, end_time, freeze_time, 
                       unfreeze_time, is_frozen, penalty_minutes, contest_config, status
                FROM contests 
                WHERE status IN ('not_started', 'running', 'frozen')
                ORDER BY start_time
            ".to_string(),
            parameters: vec![],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        
        if let Some(rows) = result.as_array() {
            for row in rows {
                if let Some(contest_id_str) = row.get("id").and_then(|v| v.as_str()) {
                    if let Ok(contest_id) = Uuid::parse_str(contest_id_str) {
                        let contest_data = self.parse_contest_from_row(row).await?;
                        self.contest_cache.insert(contest_id, contest_data);
                    }
                }
            }
        }
        
        info!("Loaded {} active contests", self.contest_cache.len());
        Ok(())
    }

    async fn parse_contest_from_row(&self, row: &Value) -> PluginResult<ContestData> {
        let contest_id = Uuid::parse_str(row["id"].as_str().unwrap_or(""))
            .map_err(|e| PluginError::InvalidInput(format!("Invalid contest UUID: {}", e)))?;
        
        let start_time = chrono::DateTime::parse_from_rfc3339(row["start_time"].as_str().unwrap_or(""))
            .map_err(|e| PluginError::InvalidInput(format!("Invalid start time: {}", e)))?
            .with_timezone(&Utc);
        
        let end_time = chrono::DateTime::parse_from_rfc3339(row["end_time"].as_str().unwrap_or(""))
            .map_err(|e| PluginError::InvalidInput(format!("Invalid end time: {}", e)))?
            .with_timezone(&Utc);
        
        let freeze_time = row.get("freeze_time")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        let unfreeze_time = row.get("unfreeze_time")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        // Load problems and teams for this contest
        let problems = self.load_contest_problems(contest_id).await?;
        let teams = self.load_contest_teams(contest_id).await?;
        
        // Parse contest config
        let config: IcpcConfig = if let Some(config_value) = row.get("contest_config") {
            serde_json::from_value(config_value.clone()).unwrap_or_default()
        } else {
            IcpcConfig::default()
        };
        
        let status = match row["status"].as_str().unwrap_or("not_started") {
            "running" => ContestStatus::Running,
            "frozen" => ContestStatus::Frozen,
            "finished" => ContestStatus::Finished,
            "cancelled" => ContestStatus::Cancelled,
            _ => ContestStatus::NotStarted,
        };
        
        Ok(ContestData {
            id: contest_id,
            title: row["title"].as_str().unwrap_or("").to_string(),
            description: row["description"].as_str().unwrap_or("").to_string(),
            start_time,
            end_time,
            freeze_time,
            unfreeze_time,
            is_frozen: row["is_frozen"].as_bool().unwrap_or(false),
            penalty_minutes: row["penalty_minutes"].as_i64().unwrap_or(20) as i32,
            problems,
            teams,
            config,
            status,
        })
    }

    async fn load_contest_problems(&self, contest_id: Uuid) -> PluginResult<Vec<ContestProblem>> {
        let query = DatabaseQuery {
            query: r"
                SELECT p.id, cp.problem_letter, p.title, cp.balloon_color,
                       cp.first_solve_team_id, cp.first_solve_time
                FROM contest_problems cp
                JOIN problems p ON cp.problem_id = p.id
                WHERE cp.contest_id = $1
                ORDER BY cp.problem_letter
            ".to_string(),
            parameters: vec![serde_json::to_value(contest_id.to_string())?],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        let mut problems = Vec::new();
        
        if let Some(rows) = result.as_array() {
            for row in rows {
                let problem_id = Uuid::parse_str(row["id"].as_str().unwrap_or(""))
                    .map_err(|e| PluginError::InvalidInput(format!("Invalid problem UUID: {}", e)))?;
                
                let first_solve_team = row.get("first_solve_team_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok());
                
                let first_solve_time = row.get("first_solve_time")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc));
                
                problems.push(ContestProblem {
                    id: problem_id,
                    letter: row["problem_letter"].as_str().unwrap_or("A").to_string(),
                    title: row["title"].as_str().unwrap_or("").to_string(),
                    color: row["balloon_color"].as_str().unwrap_or("red").to_string(),
                    first_solve_team,
                    first_solve_time,
                    solve_count: 0, // Will be computed later
                    attempt_count: 0, // Will be computed later
                });
            }
        }
        
        Ok(problems)
    }

    async fn load_contest_teams(&self, contest_id: Uuid) -> PluginResult<Vec<TeamData>> {
        let query = DatabaseQuery {
            query: r"
                SELECT t.id, t.name, t.organization, t.members, t.coach,
                       ct.registration_time, ct.is_hidden
                FROM contest_teams ct
                JOIN teams t ON ct.team_id = t.id
                WHERE ct.contest_id = $1
                ORDER BY t.name
            ".to_string(),
            parameters: vec![serde_json::to_value(contest_id.to_string())?],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        let mut teams = Vec::new();
        
        if let Some(rows) = result.as_array() {
            for row in rows {
                let team_id = Uuid::parse_str(row["id"].as_str().unwrap_or(""))
                    .map_err(|e| PluginError::InvalidInput(format!("Invalid team UUID: {}", e)))?;
                
                let registration_time = chrono::DateTime::parse_from_rfc3339(row["registration_time"].as_str().unwrap_or(""))
                    .map_err(|e| PluginError::InvalidInput(format!("Invalid registration time: {}", e)))?
                    .with_timezone(&Utc);
                
                let members: Vec<String> = row.get("members")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                    .unwrap_or_default();
                
                teams.push(TeamData {
                    id: team_id,
                    name: row["name"].as_str().unwrap_or("").to_string(),
                    organization: row["organization"].as_str().unwrap_or("").to_string(),
                    members,
                    coach: row.get("coach").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    registration_time,
                    is_hidden: row["is_hidden"].as_bool().unwrap_or(false),
                    rank: None, // Will be computed during scoreboard generation
                });
            }
        }
        
        Ok(teams)
    }

    // Event handlers

    async fn handle_submission_created(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        debug!("Processing submission created event: {:?}", event);
        
        if let Some(contest_id) = event.payload.get("contest_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
            // Invalidate scoreboard cache for this contest
            self.scoreboard_cache.remove(&contest_id);
            
            // Emit scoreboard update event
            emit_platform_event(&PlatformEvent {
                id: Uuid::new_v4(),
                event_type: "icpc.scoreboard.update_needed".to_string(),
                source_plugin_id: Some(Uuid::new_v4()), // Should be our plugin ID
                timestamp: Utc::now(),
                payload: serde_json::json!({
                    "contest_id": contest_id,
                    "reason": "submission_created"
                }),
                metadata: HashMap::new(),
            }).await?;
        }
        
        Ok(())
    }

    async fn handle_judging_completed(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        debug!("Processing judging completed event: {:?}", event);
        
        if let Some(contest_id) = event.payload.get("contest_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
            // Check if this is a first solve
            if let Some(verdict) = event.payload.get("verdict").and_then(|v| v.as_str()) {
                if verdict == "AC" || verdict == "Accepted" {
                    self.handle_accepted_submission(event, contest_id).await?;
                }
            }
            
            // Update scoreboard
            self.update_scoreboard(contest_id).await?;
        }
        
        Ok(())
    }

    async fn handle_accepted_submission(&mut self, event: &PlatformEvent, contest_id: Uuid) -> PluginResult<()> {
        let team_id = event.payload.get("team_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        
        let problem_id = event.payload.get("problem_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        
        if let (Some(team_id), Some(problem_id)) = (team_id, problem_id) {
            // Check if this is a first solve
            let is_first_solve = self.check_if_first_solve(contest_id, problem_id).await?;
            
            if is_first_solve {
                // Update contest cache
                if let Some(contest) = self.contest_cache.get_mut(&contest_id) {
                    if let Some(problem) = contest.problems.iter_mut().find(|p| p.id == problem_id) {
                        problem.first_solve_team = Some(team_id);
                        problem.first_solve_time = Some(Utc::now());
                    }
                }
                
                // Create balloon delivery
                if let Some(contest) = self.contest_cache.get(&contest_id) {
                    if contest.config.enable_balloons {
                        self.create_balloon_delivery(contest_id, team_id, problem_id).await?;
                    }
                }
                
                // Send first solve notification
                self.send_first_solve_notification(contest_id, team_id, problem_id).await?;
                
                // Emit first solve event
                emit_platform_event(&PlatformEvent {
                    id: Uuid::new_v4(),
                    event_type: "icpc.first_solve".to_string(),
                    source_plugin_id: Some(Uuid::new_v4()), // Should be our plugin ID
                    timestamp: Utc::now(),
                    payload: serde_json::json!({
                        "contest_id": contest_id,
                        "team_id": team_id,
                        "problem_id": problem_id
                    }),
                    metadata: HashMap::new(),
                }).await?;
            }
        }
        
        Ok(())
    }

    async fn check_if_first_solve(&self, contest_id: Uuid, problem_id: Uuid) -> PluginResult<bool> {
        let query = DatabaseQuery {
            query: r"
                SELECT COUNT(*) as solve_count
                FROM submissions s
                JOIN contest_teams ct ON s.team_id = ct.team_id
                WHERE ct.contest_id = $1 
                  AND s.problem_id = $2 
                  AND s.verdict = 'AC'
            ".to_string(),
            parameters: vec![
                serde_json::to_value(contest_id.to_string())?,
                serde_json::to_value(problem_id.to_string())?,
            ],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        
        if let Some(rows) = result.as_array() {
            if let Some(row) = rows.first() {
                let solve_count = row["solve_count"].as_i64().unwrap_or(0);
                return Ok(solve_count == 1); // First solve if count is 1
            }
        }
        
        Ok(false)
    }

    async fn create_balloon_delivery(&self, contest_id: Uuid, team_id: Uuid, problem_id: Uuid) -> PluginResult<()> {
        // Get problem letter and color
        let (problem_letter, color) = if let Some(contest) = self.contest_cache.get(&contest_id) {
            if let Some(problem) = contest.problems.iter().find(|p| p.id == problem_id) {
                (problem.letter.clone(), problem.color.clone())
            } else {
                return Err(PluginError::InvalidInput("Problem not found in contest".into()));
            }
        } else {
            return Err(PluginError::InvalidInput("Contest not found".into()));
        };
        
        let balloon = BalloonDelivery {
            id: Uuid::new_v4(),
            contest_id,
            team_id,
            problem_letter,
            color,
            solve_time: Utc::now(),
            delivered: false,
            delivered_at: None,
            delivered_by: None,
        };
        
        // Store balloon in database
        let query = DatabaseQuery {
            query: r"
                INSERT INTO balloon_deliveries 
                (id, contest_id, team_id, problem_letter, color, solve_time, delivered)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            ".to_string(),
            parameters: vec![
                serde_json::to_value(balloon.id.to_string())?,
                serde_json::to_value(balloon.contest_id.to_string())?,
                serde_json::to_value(balloon.team_id.to_string())?,
                serde_json::to_value(&balloon.problem_letter)?,
                serde_json::to_value(&balloon.color)?,
                serde_json::to_value(balloon.solve_time.to_rfc3339())?,
                serde_json::to_value(balloon.delivered)?,
            ],
            timeout_ms: Some(5000),
        };
        
        database_query(&query).await?;
        
        // Emit balloon event
        emit_platform_event(&PlatformEvent {
            id: Uuid::new_v4(),
            event_type: "icpc.balloon.created".to_string(),
            source_plugin_id: Some(Uuid::new_v4()), // Should be our plugin ID
            timestamp: Utc::now(),
            payload: serde_json::to_value(&balloon)?,
            metadata: HashMap::new(),
        }).await?;
        
        Ok(())
    }

    async fn send_first_solve_notification(&self, contest_id: Uuid, team_id: Uuid, problem_id: Uuid) -> PluginResult<()> {
        // Get team and problem info
        let team_name = if let Some(contest) = self.contest_cache.get(&contest_id) {
            contest.teams.iter().find(|t| t.id == team_id).map(|t| t.name.clone())
        } else {
            None
        }.unwrap_or_else(|| "Unknown Team".to_string());
        
        let problem_title = if let Some(contest) = self.contest_cache.get(&contest_id) {
            contest.problems.iter().find(|p| p.id == problem_id).map(|p| format!("Problem {}: {}", p.letter, p.title))
        } else {
            None
        }.unwrap_or_else(|| "Unknown Problem".to_string());
        
        // Send notification to contest admins
        let notification = Notification {
            recipient_id: Uuid::nil(), // Broadcast to all contest admins
            title: "🎉 First Solve!".to_string(),
            message: format!("{} achieved the first solve for {}!", team_name, problem_title),
            notification_type: NotificationType::Success,
            urgency: NotificationUrgency::High,
            metadata: [
                ("contest_id".to_string(), serde_json::json!(contest_id.to_string())),
                ("team_id".to_string(), serde_json::json!(team_id.to_string())),
                ("problem_id".to_string(), serde_json::json!(problem_id.to_string())),
                ("notification_type".to_string(), serde_json::json!("first_solve")),
            ].iter().cloned().collect(),
        };
        
        send_notification(&notification).await?;
        Ok(())
    }

    async fn update_scoreboard(&mut self, contest_id: Uuid) -> PluginResult<()> {
        info!("Updating scoreboard for contest {}", contest_id);
        
        // Generate fresh scoreboard data
        let scoreboard = self.generate_scoreboard(contest_id).await?;
        
        // Cache the result
        self.scoreboard_cache.insert(contest_id, scoreboard);
        
        // Emit scoreboard update event
        emit_platform_event(&PlatformEvent {
            id: Uuid::new_v4(),
            event_type: "icpc.scoreboard.updated".to_string(),
            source_plugin_id: Some(Uuid::new_v4()), // Should be our plugin ID
            timestamp: Utc::now(),
            payload: serde_json::json!({
                "contest_id": contest_id
            }),
            metadata: HashMap::new(),
        }).await?;
        
        Ok(())
    }

    async fn generate_scoreboard(&self, contest_id: Uuid) -> PluginResult<ScoreboardData> {
        let contest = self.contest_cache.get(&contest_id)
            .ok_or_else(|| PluginError::InvalidInput("Contest not found".into()))?;
        
        // Get all submissions for this contest
        let query = DatabaseQuery {
            query: r"
                SELECT s.team_id, s.problem_id, s.verdict, s.submitted_at, 
                       s.execution_time, cp.problem_letter, t.name as team_name,
                       t.organization
                FROM submissions s
                JOIN contest_teams ct ON s.team_id = ct.team_id
                JOIN contest_problems cp ON s.problem_id = cp.problem_id AND cp.contest_id = ct.contest_id
                JOIN teams t ON s.team_id = t.id
                WHERE ct.contest_id = $1 
                  AND s.submitted_at >= $2
                  AND s.submitted_at <= $3
                ORDER BY s.team_id, s.problem_id, s.submitted_at
            ".to_string(),
            parameters: vec![
                serde_json::to_value(contest_id.to_string())?,
                serde_json::to_value(contest.start_time.to_rfc3339())?,
                serde_json::to_value(contest.end_time.to_rfc3339())?,
            ],
            timeout_ms: Some(10000),
        };
        
        let result = database_query(&query).await?;
        
        // Build team standings
        let mut team_standings: HashMap<Uuid, TeamStanding> = HashMap::new();
        
        if let Some(rows) = result.as_array() {
            for row in rows {
                let team_id = Uuid::parse_str(row["team_id"].as_str().unwrap_or("")).unwrap();
                let problem_id = Uuid::parse_str(row["problem_id"].as_str().unwrap_or("")).unwrap();
                let problem_letter = row["problem_letter"].as_str().unwrap_or("A").to_string();
                let verdict = row["verdict"].as_str().unwrap_or("QU");
                let submitted_at = chrono::DateTime::parse_from_rfc3339(row["submitted_at"].as_str().unwrap_or(""))
                    .unwrap()
                    .with_timezone(&Utc);
                
                // Initialize team standing if not exists
                let team_standing = team_standings.entry(team_id).or_insert_with(|| {
                    TeamStanding {
                        rank: 0,
                        team_id,
                        team_name: row["team_name"].as_str().unwrap_or("").to_string(),
                        organization: row["organization"].as_str().unwrap_or("").to_string(),
                        solved_count: 0,
                        total_time: 0,
                        problem_results: IndexMap::new(),
                        last_solve_time: None,
                    }
                });
                
                // Initialize problem result if not exists
                let problem_result = team_standing.problem_results.entry(problem_letter.clone()).or_insert_with(|| {
                    ProblemResult {
                        problem_letter: problem_letter.clone(),
                        status: ProblemStatus::NotAttempted,
                        attempts: 0,
                        solve_time_minutes: None,
                        penalty_time: 0,
                        is_first_solve: false,
                    }
                });
                
                problem_result.attempts += 1;
                
                if verdict == "AC" || verdict == "Accepted" {
                    if problem_result.status != ProblemStatus::Solved {
                        // First accept for this problem
                        problem_result.status = ProblemStatus::Solved;
                        let solve_time_minutes = (submitted_at.timestamp() - contest.start_time.timestamp()) / 60;
                        problem_result.solve_time_minutes = Some(solve_time_minutes as i32);
                        problem_result.penalty_time = (problem_result.attempts - 1) * contest.penalty_minutes;
                        
                        // Check if this is the first solve for this problem
                        if let Some(problem) = contest.problems.iter().find(|p| p.id == problem_id) {
                            if problem.first_solve_team == Some(team_id) {
                                problem_result.is_first_solve = true;
                                problem_result.status = ProblemStatus::FirstSolve;
                            }
                        }
                        
                        team_standing.solved_count += 1;
                        team_standing.total_time += solve_time_minutes as i32 + problem_result.penalty_time;
                        team_standing.last_solve_time = Some(submitted_at);
                    }
                } else if problem_result.status == ProblemStatus::NotAttempted {
                    problem_result.status = ProblemStatus::Attempted;
                }
            }
        }
        
        // Convert to sorted vector
        let mut standings: Vec<TeamStanding> = team_standings.into_values().collect();
        
        // Sort by ICPC rules: more problems solved, then less total time, then last solve time
        standings.sort_by(|a, b| {
            b.solved_count.cmp(&a.solved_count)
                .then_with(|| a.total_time.cmp(&b.total_time))
                .then_with(|| a.last_solve_time.cmp(&b.last_solve_time))
        });
        
        // Assign ranks
        for (index, standing) in standings.iter_mut().enumerate() {
            standing.rank = (index + 1) as i32;
        }
        
        // Generate problem statistics
        let problem_stats = self.generate_problem_statistics(contest_id, &standings).await?;
        
        Ok(ScoreboardData {
            contest_id,
            last_updated: Utc::now(),
            is_frozen: contest.is_frozen,
            standings,
            problem_stats,
        })
    }

    async fn generate_problem_statistics(&self, contest_id: Uuid, standings: &[TeamStanding]) -> PluginResult<Vec<ProblemStats>> {
        let contest = self.contest_cache.get(&contest_id)
            .ok_or_else(|| PluginError::InvalidInput("Contest not found".into()))?;
        
        let mut stats = Vec::new();
        
        for problem in &contest.problems {
            let mut total_attempts = 0;
            let mut total_solves = 0;
            
            for standing in standings {
                if let Some(result) = standing.problem_results.get(&problem.letter) {
                    total_attempts += result.attempts;
                    if matches!(result.status, ProblemStatus::Solved | ProblemStatus::FirstSolve) {
                        total_solves += 1;
                    }
                }
            }
            
            let solve_percentage = if total_attempts > 0 {
                (total_solves as f64 / standings.len() as f64) * 100.0
            } else {
                0.0
            };
            
            let first_solve_team = problem.first_solve_team
                .and_then(|team_id| contest.teams.iter().find(|t| t.id == team_id))
                .map(|team| team.name.clone());
            
            stats.push(ProblemStats {
                problem_letter: problem.letter.clone(),
                problem_title: problem.title.clone(),
                total_attempts,
                total_solves,
                first_solve_team,
                first_solve_time: problem.first_solve_time,
                solve_percentage,
            });
        }
        
        Ok(stats)
    }

    // Additional event handlers
    async fn handle_contest_created(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        debug!("Contest created event: {:?}", event);
        
        if let Some(contest_id) = event.payload.get("contest_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
            // Load the new contest into cache
            if let Ok(contest_data) = self.load_contest_by_id(contest_id).await {
                self.contest_cache.insert(contest_id, contest_data);
            }
        }
        
        Ok(())
    }

    async fn handle_contest_updated(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        debug!("Contest updated event: {:?}", event);
        
        if let Some(contest_id) = event.payload.get("contest_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
            // Reload contest from database
            if let Ok(contest_data) = self.load_contest_by_id(contest_id).await {
                self.contest_cache.insert(contest_id, contest_data);
                
                // Invalidate scoreboard cache
                self.scoreboard_cache.remove(&contest_id);
            }
        }
        
        Ok(())
    }

    async fn handle_contest_started(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        debug!("Contest started event: {:?}", event);
        
        if let Some(contest_id) = event.payload.get("contest_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
            // Update contest status in cache
            if let Some(contest) = self.contest_cache.get_mut(&contest_id) {
                contest.status = ContestStatus::Running;
            }
            
            // Initialize empty scoreboard
            self.update_scoreboard(contest_id).await?;
        }
        
        Ok(())
    }

    async fn handle_contest_finished(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        debug!("Contest finished event: {:?}", event);
        
        if let Some(contest_id) = event.payload.get("contest_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
            // Update contest status in cache
            if let Some(contest) = self.contest_cache.get_mut(&contest_id) {
                contest.status = ContestStatus::Finished;
                contest.is_frozen = false; // Automatically unfreeze when contest ends
            }
            
            // Generate final scoreboard
            self.update_scoreboard(contest_id).await?;
            
            // Generate final results and awards
            self.generate_final_results(contest_id).await?;
        }
        
        Ok(())
    }

    async fn handle_team_registered(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        debug!("Team registered event: {:?}", event);
        
        if let Some(contest_id) = event.payload.get("contest_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
            // Reload contest teams
            // Reload contest teams
            let teams = self.load_contest_teams(contest_id).await?;
            if let Some(contest) = self.contest_cache.get_mut(&contest_id) {
                contest.teams = teams;
            }
            
            // Send welcome notification to team
            if let Some(team_id) = event.payload.get("team_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
                self.send_team_registration_notification(contest_id, team_id).await?;
            }
        }
        
        Ok(())
    }

    async fn load_contest_by_id(&self, contest_id: Uuid) -> PluginResult<ContestData> {
        let query = DatabaseQuery {
            query: r"
                SELECT id, title, description, start_time, end_time, freeze_time, 
                       unfreeze_time, is_frozen, penalty_minutes, contest_config, status
                FROM contests 
                WHERE id = $1
            ".to_string(),
            parameters: vec![serde_json::to_value(contest_id.to_string())?],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        
        if let Some(rows) = result.as_array() {
            if let Some(row) = rows.first() {
                return self.parse_contest_from_row(row).await;
            }
        }
        
        Err(PluginError::InvalidInput("Contest not found".into()))
    }

    async fn generate_final_results(&self, contest_id: Uuid) -> PluginResult<()> {
        info!("Generating final results for contest {}", contest_id);
        
        if let Some(scoreboard) = self.scoreboard_cache.get(&contest_id) {
            // Send final results notification
            let top_teams: Vec<String> = scoreboard.standings
                .iter()
                .take(3)
                .map(|s| format!("{}. {}", s.rank, s.team_name))
                .collect();
            
            let notification = Notification {
                recipient_id: Uuid::nil(), // Broadcast
                title: "🏆 Contest Finished!".to_string(),
                message: format!("Final Results:\n{}", top_teams.join("\n")),
                notification_type: NotificationType::Info,
                urgency: NotificationUrgency::Normal,
                metadata: [
                    ("contest_id".to_string(), serde_json::json!(contest_id.to_string())),
                    ("notification_type".to_string(), serde_json::json!("contest_finished")),
                ].iter().cloned().collect(),
            };
            
            send_notification(&notification).await?;
        }
        
        Ok(())
    }

    async fn send_team_registration_notification(&self, contest_id: Uuid, team_id: Uuid) -> PluginResult<()> {
        let team_name = if let Some(contest) = self.contest_cache.get(&contest_id) {
            contest.teams.iter().find(|t| t.id == team_id).map(|t| t.name.clone())
        } else {
            None
        }.unwrap_or_else(|| "Unknown Team".to_string());
        
        let contest_title = self.contest_cache.get(&contest_id)
            .map(|c| c.title.clone())
            .unwrap_or_else(|| "Unknown Contest".to_string());
        
        let notification = Notification {
            recipient_id: team_id, // Send to team
            title: "✅ Registration Confirmed".to_string(),
            message: format!("Team '{}' has been successfully registered for '{}'", team_name, contest_title),
            notification_type: NotificationType::Success,
            urgency: NotificationUrgency::Normal,
            metadata: [
                ("contest_id".to_string(), serde_json::json!(contest_id.to_string())),
                ("team_id".to_string(), serde_json::json!(team_id.to_string())),
                ("notification_type".to_string(), serde_json::json!("team_registration")),
            ].iter().cloned().collect(),
        };
        
        send_notification(&notification).await?;
        Ok(())
    }

    // HTTP Request Handlers (Part 1)
    
    async fn handle_list_contests(&self, _request: &HttpRequest) -> PluginResult<HttpResponse> {
        let contests: Vec<&ContestData> = self.contest_cache.values().collect();
        
        let response = serde_json::json!({
            "contests": contests,
            "total": contests.len()
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_create_contest(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let body: Value = serde_json::from_str(request.body.as_ref().unwrap_or(&String::new()))
            .map_err(|e| PluginError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        
        // Validate required fields
        let title = body.get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::InvalidInput("Missing 'title' field".to_string()))?;
        
        let start_time = body.get("start_time")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| PluginError::InvalidInput("Invalid 'start_time' field".to_string()))?;
        
        let end_time = body.get("end_time")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| PluginError::InvalidInput("Invalid 'end_time' field".to_string()))?;
        
        let contest_id = Uuid::new_v4();
        
        // Create contest in database
        let query = DatabaseQuery {
            query: r"
                INSERT INTO contests 
                (id, title, description, start_time, end_time, penalty_minutes, contest_config, status)
                VALUES ($1, $2, $3, $4, $5, $6, $7, 'not_started')
            ".to_string(),
            parameters: vec![
                serde_json::to_value(contest_id.to_string())?,
                serde_json::to_value(title)?,
                serde_json::to_value(body.get("description").and_then(|v| v.as_str()).unwrap_or(""))?,
                serde_json::to_value(start_time.to_rfc3339())?,
                serde_json::to_value(end_time.to_rfc3339())?,
                serde_json::to_value(body.get("penalty_minutes").and_then(|v| v.as_i64()).unwrap_or(20))?,
                body.get("config").cloned().unwrap_or(serde_json::json!({})),
            ],
            timeout_ms: Some(5000),
        };
        
        database_query(&query).await?;
        
        let response = serde_json::json!({
            "contest_id": contest_id,
            "message": "Contest created successfully"
        });
        
        Ok(HttpResponse {
            status_code: 201,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_get_contest(&self, _request: &HttpRequest, contest_id_str: &str) -> PluginResult<HttpResponse> {
        let contest_id = Uuid::parse_str(contest_id_str)
            .map_err(|e| PluginError::InvalidInput(format!("Invalid contest UUID: {}", e)))?;
        
        if let Some(contest) = self.contest_cache.get(&contest_id) {
            let response = serde_json::json!({
                "contest": contest
            });
            
            Ok(HttpResponse {
                status_code: 200,
                headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
                body: response.to_string(),
                content_type: "application/json".to_string(),
            })
        } else {
            Ok(HttpResponse {
                status_code: 404,
                headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
                body: serde_json::json!({"error": "Contest not found"}).to_string(),
                content_type: "application/json".to_string(),
            })
        }
    }

    async fn handle_get_scoreboard(&self, _request: &HttpRequest, contest_id_str: &str) -> PluginResult<HttpResponse> {
        let contest_id = Uuid::parse_str(contest_id_str)
            .map_err(|e| PluginError::InvalidInput(format!("Invalid contest UUID: {}", e)))?;
        
        // Get cached scoreboard or generate new one
        let scoreboard = if let Some(cached) = self.scoreboard_cache.get(&contest_id) {
            cached.clone()
        } else {
            // Generate fresh scoreboard
            self.generate_scoreboard(contest_id).await?
        };
        
        let response = serde_json::json!({
            "scoreboard": scoreboard
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_get_teams(&self, _request: &HttpRequest, contest_id_str: &str) -> PluginResult<HttpResponse> {
        let contest_id = Uuid::parse_str(contest_id_str)
            .map_err(|e| PluginError::InvalidInput(format!("Invalid contest UUID: {}", e)))?;
        
        if let Some(contest) = self.contest_cache.get(&contest_id) {
            let response = serde_json::json!({
                "teams": contest.teams,
                "total": contest.teams.len()
            });
            
            Ok(HttpResponse {
                status_code: 200,
                headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
                body: response.to_string(),
                content_type: "application/json".to_string(),
            })
        } else {
            Ok(HttpResponse {
                status_code: 404,
                headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
                body: serde_json::json!({"error": "Contest not found"}).to_string(),
                content_type: "application/json".to_string(),
            })
        }
    }

    // Additional HTTP handlers will continue in the next part...
    async fn handle_team_registration(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let body: Value = serde_json::from_str(request.body.as_ref().unwrap_or(&String::new()))
            .map_err(|e| PluginError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        
        // Extract contest_id from path
        let contest_id = self.extract_contest_id_from_path(&request.path)?;
        
        // Validate team registration data
        let team_name = body.get("team_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::InvalidInput("Missing 'team_name' field".to_string()))?;
        
        let organization = body.get("organization")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let members: Vec<String> = body.get("members")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default();
        
        let team_id = Uuid::new_v4();
        
        // Register team in database
        let query = DatabaseQuery {
            query: r"
                INSERT INTO teams (id, name, organization, members, coach)
                VALUES ($1, $2, $3, $4, $5)
            ".to_string(),
            parameters: vec![
                serde_json::to_value(team_id.to_string())?,
                serde_json::to_value(team_name)?,
                serde_json::to_value(organization)?,
                serde_json::to_value(&members)?,
                body.get("coach").cloned().unwrap_or(Value::Null),
            ],
            timeout_ms: Some(5000),
        };
        
        database_query(&query).await?;
        
        // Add team to contest
        let query = DatabaseQuery {
            query: r"
                INSERT INTO contest_teams (contest_id, team_id, registration_time, is_hidden)
                VALUES ($1, $2, NOW(), false)
            ".to_string(),
            parameters: vec![
                serde_json::to_value(contest_id.to_string())?,
                serde_json::to_value(team_id.to_string())?,
            ],
            timeout_ms: Some(5000),
        };
        
        database_query(&query).await?;
        
        // Emit team registered event
        emit_platform_event(&PlatformEvent {
            id: Uuid::new_v4(),
            event_type: "team.registered".to_string(),
            source_plugin_id: Some(Uuid::new_v4()), // Should be our plugin ID
            timestamp: Utc::now(),
            payload: serde_json::json!({
                "contest_id": contest_id,
                "team_id": team_id,
                "team_name": team_name
            }),
            metadata: HashMap::new(),
        }).await?;
        
        let response = serde_json::json!({
            "team_id": team_id,
            "message": "Team registered successfully"
        });
        
        Ok(HttpResponse {
            status_code: 201,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_get_clarifications(&self, _request: &HttpRequest, contest_id_str: &str) -> PluginResult<HttpResponse> {
        let contest_id = Uuid::parse_str(contest_id_str)
            .map_err(|e| PluginError::InvalidInput(format!("Invalid contest UUID: {}", e)))?;
        
        let query = DatabaseQuery {
            query: r"
                SELECT c.*, t.name as team_name
                FROM clarifications c
                JOIN teams t ON c.team_id = t.id
                WHERE c.contest_id = $1
                ORDER BY c.timestamp DESC
            ".to_string(),
            parameters: vec![serde_json::to_value(contest_id.to_string())?],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        let mut clarifications = Vec::new();
        
        if let Some(rows) = result.as_array() {
            for row in rows {
                let clarification = ClarificationRequest {
                    id: Uuid::parse_str(row["id"].as_str().unwrap_or("")).unwrap(),
                    contest_id,
                    team_id: Uuid::parse_str(row["team_id"].as_str().unwrap_or("")).unwrap(),
                    problem_letter: row.get("problem_letter").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    question: row["question"].as_str().unwrap_or("").to_string(),
                    answer: row.get("answer").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    is_public: row["is_public"].as_bool().unwrap_or(false),
                    timestamp: chrono::DateTime::parse_from_rfc3339(row["timestamp"].as_str().unwrap_or("")).unwrap().with_timezone(&Utc),
                    answered_by: row.get("answered_by").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
                    answered_at: row.get("answered_at").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)),
                };
                clarifications.push(clarification);
            }
        }
        
        let response = serde_json::json!({
            "clarifications": clarifications
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_create_clarification(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let body: Value = serde_json::from_str(request.body.as_ref().unwrap_or(&String::new()))
            .map_err(|e| PluginError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        
        let contest_id = self.extract_contest_id_from_path(&request.path)?;
        
        let team_id = body.get("team_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| PluginError::InvalidInput("Invalid team_id".to_string()))?;
        
        let question = body.get("question")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::InvalidInput("Missing question".to_string()))?;
        
        let clarification_id = Uuid::new_v4();
        
        let query = DatabaseQuery {
            query: r"
                INSERT INTO clarifications 
                (id, contest_id, team_id, problem_letter, question, is_public, timestamp)
                VALUES ($1, $2, $3, $4, $5, false, NOW())
            ".to_string(),
            parameters: vec![
                serde_json::to_value(clarification_id.to_string())?,
                serde_json::to_value(contest_id.to_string())?,
                serde_json::to_value(team_id.to_string())?,
                body.get("problem_letter").cloned().unwrap_or(Value::Null),
                serde_json::to_value(question)?,
            ],
            timeout_ms: Some(5000),
        };
        
        database_query(&query).await?;
        
        // Send notification to contest admins
        let notification = Notification {
            recipient_id: Uuid::nil(), // Broadcast to admins
            title: "📋 New Clarification Request".to_string(),
            message: format!("A team has submitted a new clarification request: {}", question),
            notification_type: NotificationType::Info,
            urgency: NotificationUrgency::Normal,
            metadata: [
                ("contest_id".to_string(), serde_json::json!(contest_id.to_string())),
                ("clarification_id".to_string(), serde_json::json!(clarification_id.to_string())),
            ].iter().cloned().collect(),
        };
        
        send_notification(&notification).await?;
        
        let response = serde_json::json!({
            "clarification_id": clarification_id,
            "message": "Clarification submitted successfully"
        });
        
        Ok(HttpResponse {
            status_code: 201,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_freeze_contest(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let contest_id = self.extract_contest_id_from_path(&request.path)?;
        
        // Update contest freeze status
        let query = DatabaseQuery {
            query: r"
                UPDATE contests 
                SET is_frozen = true, freeze_time = NOW()
                WHERE id = $1
            ".to_string(),
            parameters: vec![serde_json::to_value(contest_id.to_string())?],
            timeout_ms: Some(5000),
        };
        
        database_query(&query).await?;
        
        // Update cache
        if let Some(contest) = self.contest_cache.get_mut(&contest_id) {
            contest.is_frozen = true;
            contest.freeze_time = Some(Utc::now());
        }
        
        // Emit freeze event
        emit_platform_event(&PlatformEvent {
            id: Uuid::new_v4(),
            event_type: "icpc.contest.frozen".to_string(),
            source_plugin_id: Some(Uuid::new_v4()), // Should be our plugin ID
            timestamp: Utc::now(),
            payload: serde_json::json!({
                "contest_id": contest_id
            }),
            metadata: HashMap::new(),
        }).await?;
        
        let response = serde_json::json!({
            "message": "Contest frozen successfully"
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_unfreeze_contest(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let contest_id = self.extract_contest_id_from_path(&request.path)?;
        
        // Update contest freeze status
        let query = DatabaseQuery {
            query: r"
                UPDATE contests 
                SET is_frozen = false, unfreeze_time = NOW()
                WHERE id = $1
            ".to_string(),
            parameters: vec![serde_json::to_value(contest_id.to_string())?],
            timeout_ms: Some(5000),
        };
        
        database_query(&query).await?;
        
        // Update cache
        if let Some(contest) = self.contest_cache.get_mut(&contest_id) {
            contest.is_frozen = false;
            contest.unfreeze_time = Some(Utc::now());
        }
        
        // Update scoreboard with all results
        self.update_scoreboard(contest_id).await?;
        
        // Emit unfreeze event
        emit_platform_event(&PlatformEvent {
            id: Uuid::new_v4(),
            event_type: "icpc.contest.unfrozen".to_string(),
            source_plugin_id: Some(Uuid::new_v4()), // Should be our plugin ID
            timestamp: Utc::now(),
            payload: serde_json::json!({
                "contest_id": contest_id
            }),
            metadata: HashMap::new(),
        }).await?;
        
        let response = serde_json::json!({
            "message": "Contest unfrozen successfully"
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_get_balloons(&self, _request: &HttpRequest, contest_id_str: &str) -> PluginResult<HttpResponse> {
        let contest_id = Uuid::parse_str(contest_id_str)
            .map_err(|e| PluginError::InvalidInput(format!("Invalid contest UUID: {}", e)))?;
        
        let query = DatabaseQuery {
            query: r"
                SELECT b.*, t.name as team_name
                FROM balloon_deliveries b
                JOIN teams t ON b.team_id = t.id
                WHERE b.contest_id = $1
                ORDER BY b.solve_time DESC
            ".to_string(),
            parameters: vec![serde_json::to_value(contest_id.to_string())?],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        let mut balloons = Vec::new();
        
        if let Some(rows) = result.as_array() {
            for row in rows {
                let balloon = serde_json::from_value::<BalloonDelivery>(row.clone())?;
                // Add team name to response
                balloons.push(serde_json::json!({
                    "balloon": balloon,
                    "team_name": row["team_name"].as_str().unwrap_or("")
                }));
            }
        }
        
        let response = serde_json::json!({
            "balloons": balloons
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_mark_balloon_delivered(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        // Extract balloon_id from path
        let path_parts: Vec<&str> = request.path.split('/').collect();
        let balloon_id = path_parts.get(6)
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| PluginError::InvalidInput("Invalid balloon ID".to_string()))?;
        
        let body: Value = serde_json::from_str(request.body.as_ref().unwrap_or(&String::new()))
            .map_err(|e| PluginError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        
        let delivered_by = body.get("delivered_by")
            .and_then(|v| v.as_str())
            .unwrap_or("admin");
        
        let query = DatabaseQuery {
            query: r"
                UPDATE balloon_deliveries 
                SET delivered = true, delivered_at = NOW(), delivered_by = $1
                WHERE id = $2
            ".to_string(),
            parameters: vec![
                serde_json::to_value(delivered_by)?,
                serde_json::to_value(balloon_id.to_string())?,
            ],
            timeout_ms: Some(5000),
        };
        
        database_query(&query).await?;
        
        let response = serde_json::json!({
            "message": "Balloon marked as delivered"
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_get_statistics(&self, _request: &HttpRequest, contest_id_str: &str) -> PluginResult<HttpResponse> {
        let contest_id = Uuid::parse_str(contest_id_str)
            .map_err(|e| PluginError::InvalidInput(format!("Invalid contest UUID: {}", e)))?;
        
        // Get scoreboard to compute statistics
        let scoreboard = if let Some(cached) = self.scoreboard_cache.get(&contest_id) {
            cached.clone()
        } else {
            self.generate_scoreboard(contest_id).await?
        };
        
        // Compute contest statistics
        let total_teams = scoreboard.standings.len();
        let total_submissions = scoreboard.standings.iter()
            .map(|s| s.problem_results.values().map(|r| r.attempts).sum::<i32>())
            .sum::<i32>();
        
        let total_solves = scoreboard.standings.iter()
            .map(|s| s.solved_count)
            .sum::<i32>();
        
        let response = serde_json::json!({
            "contest_id": contest_id,
            "statistics": {
                "total_teams": total_teams,
                "total_submissions": total_submissions,
                "total_solves": total_solves,
                "problem_statistics": scoreboard.problem_stats,
                "last_updated": scoreboard.last_updated
            }
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    fn extract_contest_id_from_path(&self, path: &str) -> PluginResult<Uuid> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 5 {
            Uuid::parse_str(parts[4])
                .map_err(|e| PluginError::InvalidInput(format!("Invalid contest UUID: {}", e)))
        } else {
            Err(PluginError::InvalidInput("Invalid contest path".into()))
        }
    }

    // Component rendering methods will continue...
    async fn render_scoreboard(&self, props: &serde_json::Value) -> PluginResult<String> {
        let contest_id = props.get("contest_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| PluginError::InvalidInput("Missing contest_id".into()))?;
        
        let scoreboard = if let Some(cached) = self.scoreboard_cache.get(&contest_id) {
            cached.clone()
        } else {
            self.generate_scoreboard(contest_id).await?
        };
        
        let freeze_indicator = if scoreboard.is_frozen {
            "<div class=\"freeze-indicator\">🥶 SCOREBOARD FROZEN</div>"
        } else {
            ""
        };
        
        let mut standings_html = String::new();
        for standing in &scoreboard.standings {
            let mut problems_html = String::new();
            
            for problem in &scoreboard.problem_stats {
                if let Some(result) = standing.problem_results.get(&problem.problem_letter) {
                    let status_class = match result.status {
                        ProblemStatus::NotAttempted => "not-attempted",
                        ProblemStatus::Attempted => "attempted",
                        ProblemStatus::Solved => "solved",
                        ProblemStatus::FirstSolve => "first-solve",
                    };
                    
                    let cell_content = match result.status {
                        ProblemStatus::NotAttempted => "".to_string(),
                        ProblemStatus::Attempted => format!("-{}", result.attempts),
                        ProblemStatus::Solved | ProblemStatus::FirstSolve => {
                            if let Some(time) = result.solve_time_minutes {
                                if result.attempts > 1 {
                                    format!("{}:{:02}<br/>(-{})", time / 60, time % 60, result.attempts - 1)
                                } else {
                                    format!("{}:{:02}", time / 60, time % 60)
                                }
                            } else {
                                "AC".to_string()
                            }
                        }
                    };
                    
                    problems_html.push_str(&format!(
                        "<td class=\"problem-cell {}\">{}</td>",
                        status_class, cell_content
                    ));
                } else {
                    problems_html.push_str("<td class=\"problem-cell not-attempted\"></td>");
                }
            }
            
            standings_html.push_str(&format!(
                "<tr class=\"team-row\">
                    <td class=\"rank\">{}</td>
                    <td class=\"team-name\">{}</td>
                    <td class=\"organization\">{}</td>
                    <td class=\"solved\">{}</td>
                    <td class=\"penalty\">{}</td>
                    {}
                </tr>",
                standing.rank,
                standing.team_name,
                standing.organization,
                standing.solved_count,
                standing.total_time,
                problems_html
            ));
        }
        
        let problem_headers: String = scoreboard.problem_stats
            .iter()
            .map(|p| format!("<th class=\"problem-header\">{}</th>", p.problem_letter))
            .collect();
        
        let html = format!(r#"
            <div class="icpc-scoreboard">
                {}
                <div class="scoreboard-header">
                    <h2>Contest Scoreboard</h2>
                    <div class="last-updated">Last updated: {}</div>
                </div>
                
                <table class="scoreboard-table">
                    <thead>
                        <tr>
                            <th class="rank-header">Rank</th>
                            <th class="team-header">Team</th>
                            <th class="org-header">Organization</th>
                            <th class="solved-header">Solved</th>
                            <th class="penalty-header">Time</th>
                            {}
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
            
            <style>
                .icpc-scoreboard {{
                    width: 100%;
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                }}
                
                .freeze-indicator {{
                    background: #17a2b8;
                    color: white;
                    padding: 10px;
                    text-align: center;
                    font-weight: bold;
                    margin-bottom: 20px;
                    border-radius: 5px;
                }}
                
                .scoreboard-header {{
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 20px;
                }}
                
                .scoreboard-header h2 {{
                    margin: 0;
                    color: #333;
                }}
                
                .last-updated {{
                    color: #666;
                    font-size: 0.9em;
                }}
                
                .scoreboard-table {{
                    width: 100%;
                    border-collapse: collapse;
                    background: white;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                }}
                
                .scoreboard-table th {{
                    background: #343a40;
                    color: white;
                    padding: 12px 8px;
                    text-align: center;
                    font-weight: 600;
                    border: 1px solid #dee2e6;
                }}
                
                .scoreboard-table td {{
                    padding: 8px;
                    text-align: center;
                    border: 1px solid #dee2e6;
                    vertical-align: middle;
                }}
                
                .team-row:nth-child(odd) {{
                    background-color: #f8f9fa;
                }}
                
                .team-row:hover {{
                    background-color: #e9ecef;
                }}
                
                .rank {{
                    font-weight: bold;
                    width: 60px;
                }}
                
                .team-name {{
                    text-align: left;
                    font-weight: 600;
                    min-width: 200px;
                }}
                
                .organization {{
                    text-align: left;
                    color: #666;
                    min-width: 150px;
                }}
                
                .solved, .penalty {{
                    font-weight: bold;
                    width: 80px;
                }}
                
                .problem-cell {{
                    width: 80px;
                    font-size: 0.85em;
                    font-weight: 500;
                }}
                
                .not-attempted {{
                    background: #fff;
                }}
                
                .attempted {{
                    background: #ffeaa7;
                    color: #d63031;
                }}
                
                .solved {{
                    background: #55efc4;
                    color: #00b894;
                }}
                
                .first-solve {{
                    background: #fd79a8;
                    color: #e84393;
                    position: relative;
                }}
                
                .first-solve::after {{
                    content: "🥇";
                    position: absolute;
                    top: 2px;
                    right: 2px;
                    font-size: 0.8em;
                }}
                
                .problem-header {{
                    width: 80px;
                    font-size: 0.9em;
                }}
            </style>
        "#, 
        freeze_indicator, 
        scoreboard.last_updated.format("%Y-%m-%d %H:%M:%S UTC"),
        problem_headers,
        standings_html);
        
        Ok(html)
    }

    async fn render_contest_dashboard(&self, props: &serde_json::Value) -> PluginResult<String> {
        let contest_id = props.get("contest_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| PluginError::InvalidInput("Missing contest_id".into()))?;
        
        let contest = self.contest_cache.get(&contest_id)
            .ok_or_else(|| PluginError::InvalidInput("Contest not found".into()))?;
        
        let status_badge = match contest.status {
            ContestStatus::NotStarted => "<span class=\"status-badge not-started\">Not Started</span>",
            ContestStatus::Running => "<span class=\"status-badge running\">Running</span>",
            ContestStatus::Frozen => "<span class=\"status-badge frozen\">Frozen</span>",
            ContestStatus::Finished => "<span class=\"status-badge finished\">Finished</span>",
            ContestStatus::Cancelled => "<span class=\"status-badge cancelled\">Cancelled</span>",
        };
        
        let now = Utc::now();
        let time_info = if contest.status == ContestStatus::Running {
            let remaining = contest.end_time.signed_duration_since(now);
            if remaining.num_seconds() > 0 {
                format!("Time Remaining: {}h {}m", 
                    remaining.num_hours(), 
                    remaining.num_minutes() % 60)
            } else {
                "Contest Ended".to_string()
            }
        } else if contest.status == ContestStatus::NotStarted {
            let until_start = contest.start_time.signed_duration_since(now);
            if until_start.num_seconds() > 0 {
                format!("Starts in: {}h {}m", 
                    until_start.num_hours(), 
                    until_start.num_minutes() % 60)
            } else {
                "Starting Soon".to_string()
            }
        } else {
            "Contest Finished".to_string()
        };
        
        let html = format!(r#"
            <div class="contest-dashboard">
                <div class="contest-header">
                    <h1>{}</h1>
                    {}
                </div>
                
                <div class="contest-info">
                    <div class="info-card">
                        <h3>📅 Schedule</h3>
                        <p><strong>Start:</strong> {}</p>
                        <p><strong>End:</strong> {}</p>
                        <p><strong>Duration:</strong> {} hours</p>
                    </div>
                    
                    <div class="info-card">
                        <h3>⏰ Status</h3>
                        <p><strong>{}</strong></p>
                        <div class="time-info">{}</div>
                    </div>
                    
                    <div class="info-card">
                        <h3>👥 Teams</h3>
                        <p><strong>Registered:</strong> {}</p>
                        <p><strong>Problems:</strong> {}</p>
                        <p><strong>Penalty:</strong> {} min</p>
                    </div>
                    
                    <div class="info-card">
                        <h3>🎈 Features</h3>
                        <p>Balloons: {}</p>
                        <p>Clarifications: {}</p>
                        <p>Frozen: {}</p>
                    </div>
                </div>
                
                <div class="contest-description">
                    <h3>Description</h3>
                    <p>{}</p>
                </div>
            </div>
            
            <style>
                .contest-dashboard {{
                    max-width: 1200px;
                    margin: 0 auto;
                    padding: 20px;
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                }}
                
                .contest-header {{
                    text-align: center;
                    margin-bottom: 30px;
                }}
                
                .contest-header h1 {{
                    margin: 0 0 10px 0;
                    color: #333;
                }}
                
                .status-badge {{
                    padding: 8px 16px;
                    border-radius: 20px;
                    font-weight: bold;
                    text-transform: uppercase;
                    font-size: 0.9em;
                }}
                
                .not-started {{ background: #ffeaa7; color: #d63031; }}
                .running {{ background: #55efc4; color: #00b894; }}
                .frozen {{ background: #74b9ff; color: #0984e3; }}
                .finished {{ background: #fd79a8; color: #e84393; }}
                .cancelled {{ background: #ddd; color: #666; }}
                
                .contest-info {{
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
                    gap: 20px;
                    margin-bottom: 30px;
                }}
                
                .info-card {{
                    background: white;
                    padding: 20px;
                    border-radius: 10px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    border-left: 4px solid #007bff;
                }}
                
                .info-card h3 {{
                    margin: 0 0 15px 0;
                    color: #333;
                    font-size: 1.1em;
                }}
                
                .info-card p {{
                    margin: 8px 0;
                    color: #666;
                }}
                
                .time-info {{
                    font-size: 1.2em;
                    font-weight: bold;
                    color: #007bff;
                    margin-top: 10px;
                }}
                
                .contest-description {{
                    background: white;
                    padding: 20px;
                    border-radius: 10px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                }}
                
                .contest-description h3 {{
                    margin: 0 0 15px 0;
                    color: #333;
                }}
                
                .contest-description p {{
                    color: #666;
                    line-height: 1.6;
                }}
            </style>
        "#, 
        contest.title,
        status_badge,
        contest.start_time.format("%Y-%m-%d %H:%M UTC"),
        contest.end_time.format("%Y-%m-%d %H:%M UTC"),
        (contest.end_time - contest.start_time).num_hours(),
        contest.status as u8,
        time_info,
        contest.teams.len(),
        contest.problems.len(),
        contest.penalty_minutes,
        if contest.config.enable_balloons { "✅" } else { "❌" },
        if contest.config.enable_clarifications { "✅" } else { "❌" },
        if contest.is_frozen { "🥶 Yes" } else { "❌ No" },
        contest.description);
        
        Ok(html)
    }

    async fn render_contest_timer(&self, props: &serde_json::Value) -> PluginResult<String> {
        let contest_id = props.get("contest_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| PluginError::InvalidInput("Missing contest_id".into()))?;
        
        let contest = self.contest_cache.get(&contest_id)
            .ok_or_else(|| PluginError::InvalidInput("Contest not found".into()))?;
        
        let html = format!(r#"
            <div class="contest-timer" data-contest-id="{}">
                <div class="timer-display" id="timer-display">
                    Loading...
                </div>
                <div class="timer-status" id="timer-status">
                    {}
                </div>
            </div>
            
            <script>
                (function() {{
                    const startTime = new Date('{}');
                    const endTime = new Date('{}');
                    const display = document.getElementById('timer-display');
                    const status = document.getElementById('timer-status');
                    
                    function updateTimer() {{
                        const now = new Date();
                        
                        if (now < startTime) {{
                            const diff = startTime - now;
                            const hours = Math.floor(diff / (1000 * 60 * 60));
                            const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
                            const seconds = Math.floor((diff % (1000 * 60)) / 1000);
                            
                            display.textContent = String(hours).padStart(2, '0') + ':' + 
                                                String(minutes).padStart(2, '0') + ':' + 
                                                String(seconds).padStart(2, '0');
                            status.textContent = 'Contest starts in:';
                            display.className = 'timer-display waiting';
                        }} else if (now >= startTime && now <= endTime) {{
                            const diff = endTime - now;
                            const hours = Math.floor(diff / (1000 * 60 * 60));
                            const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
                            const seconds = Math.floor((diff % (1000 * 60)) / 1000);
                            
                            display.textContent = String(hours).padStart(2, '0') + ':' + 
                                                String(minutes).padStart(2, '0') + ':' + 
                                                String(seconds).padStart(2, '0');
                            status.textContent = 'Time remaining:';
                            
                            if (hours === 0 && minutes < 30) {{
                                display.className = 'timer-display warning';
                            }} else {{
                                display.className = 'timer-display running';
                            }}
                        }} else {{
                            display.textContent = '00:00:00';
                            status.textContent = 'Contest finished';
                            display.className = 'timer-display finished';
                        }}
                    }}
                    
                    updateTimer();
                    setInterval(updateTimer, 1000);
                }})();
            </script>
            
            <style>
                .contest-timer {{
                    text-align: center;
                    padding: 20px;
                    background: white;
                    border-radius: 10px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    margin: 20px 0;
                }}
                
                .timer-display {{
                    font-size: 3em;
                    font-weight: bold;
                    font-family: 'Courier New', monospace;
                    margin-bottom: 10px;
                }}
                
                .timer-display.waiting {{ color: #f39c12; }}
                .timer-display.running {{ color: #27ae60; }}
                .timer-display.warning {{ color: #e74c3c; animation: pulse 1s infinite; }}
                .timer-display.finished {{ color: #7f8c8d; }}
                
                .timer-status {{
                    font-size: 1.2em;
                    color: #666;
                    text-transform: uppercase;
                    letter-spacing: 1px;
                }}
                
                @keyframes pulse {{
                    0% {{ opacity: 1; }}
                    50% {{ opacity: 0.7; }}
                    100% {{ opacity: 1; }}
                }}
            </style>
        "#, 
        contest_id,
        contest.status as u8,
        contest.start_time.to_rfc3339(),
        contest.end_time.to_rfc3339());
        
        Ok(html)
    }

    // Additional rendering methods can be implemented as needed...
    async fn render_team_registration(&self, _props: &serde_json::Value) -> PluginResult<String> {
        let html = r#"
            <div class="team-registration">
                <h2>Team Registration</h2>
                <form id="team-registration-form">
                    <div class="form-group">
                        <label for="team-name">Team Name *</label>
                        <input type="text" id="team-name" name="team_name" required>
                    </div>
                    
                    <div class="form-group">
                        <label for="organization">Organization</label>
                        <input type="text" id="organization" name="organization">
                    </div>
                    
                    <div class="form-group">
                        <label>Team Members</label>
                        <input type="text" name="member1" placeholder="Member 1" required>
                        <input type="text" name="member2" placeholder="Member 2">
                        <input type="text" name="member3" placeholder="Member 3">
                    </div>
                    
                    <div class="form-group">
                        <label for="coach">Coach</label>
                        <input type="text" id="coach" name="coach">
                    </div>
                    
                    <button type="submit">Register Team</button>
                </form>
            </div>
            
            <style>
                .team-registration {
                    max-width: 600px;
                    margin: 0 auto;
                    padding: 20px;
                }
                
                .form-group {
                    margin-bottom: 20px;
                }
                
                .form-group label {
                    display: block;
                    margin-bottom: 5px;
                    font-weight: bold;
                }
                
                .form-group input {
                    width: 100%;
                    padding: 10px;
                    border: 1px solid #ddd;
                    border-radius: 5px;
                    margin-bottom: 10px;
                }
                
                button {
                    background: #007bff;
                    color: white;
                    padding: 12px 24px;
                    border: none;
                    border-radius: 5px;
                    cursor: pointer;
                    font-size: 1.1em;
                }
                
                button:hover {
                    background: #0056b3;
                }
            </style>
        "#;
        
        Ok(html.to_string())
    }

    async fn render_clarification_system(&self, _props: &serde_json::Value) -> PluginResult<String> {
        let html = r#"
            <div class="clarification-system">
                <h2>Clarifications</h2>
                
                <div class="clarification-form">
                    <h3>Submit Clarification</h3>
                    <form id="clarification-form">
                        <div class="form-group">
                            <label for="problem">Problem (optional)</label>
                            <select id="problem" name="problem_letter">
                                <option value="">General</option>
                                <option value="A">Problem A</option>
                                <option value="B">Problem B</option>
                                <option value="C">Problem C</option>
                            </select>
                        </div>
                        
                        <div class="form-group">
                            <label for="question">Question *</label>
                            <textarea id="question" name="question" rows="4" required></textarea>
                        </div>
                        
                        <button type="submit">Submit Question</button>
                    </form>
                </div>
                
                <div class="clarifications-list">
                    <h3>Public Clarifications</h3>
                    <div id="clarifications-container">
                        Loading...
                    </div>
                </div>
            </div>
            
            <style>
                .clarification-system {
                    max-width: 800px;
                    margin: 0 auto;
                    padding: 20px;
                }
                
                .clarification-form {
                    background: white;
                    padding: 20px;
                    border-radius: 10px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    margin-bottom: 30px;
                }
                
                .form-group {
                    margin-bottom: 15px;
                }
                
                .form-group label {
                    display: block;
                    margin-bottom: 5px;
                    font-weight: bold;
                }
                
                .form-group input, .form-group select, .form-group textarea {
                    width: 100%;
                    padding: 10px;
                    border: 1px solid #ddd;
                    border-radius: 5px;
                }
                
                .clarifications-list {
                    background: white;
                    padding: 20px;
                    border-radius: 10px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                }
                
                button {
                    background: #28a745;
                    color: white;
                    padding: 10px 20px;
                    border: none;
                    border-radius: 5px;
                    cursor: pointer;
                }
                
                button:hover {
                    background: #218838;
                }
            </style>
        "#;
        
        Ok(html.to_string())
    }

    async fn render_problem_statistics(&self, props: &serde_json::Value) -> PluginResult<String> {
        let contest_id = props.get("contest_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| PluginError::InvalidInput("Missing contest_id".into()))?;
        
        let scoreboard = if let Some(cached) = self.scoreboard_cache.get(&contest_id) {
            cached.clone()
        } else {
            self.generate_scoreboard(contest_id).await?
        };
        
        let mut stats_html = String::new();
        for stat in &scoreboard.problem_stats {
            let first_solve = stat.first_solve_team.as_deref().unwrap_or("None");
            let first_solve_time = stat.first_solve_time
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_else(|| "N/A".to_string());
            
            stats_html.push_str(&format!(
                "<tr>
                    <td class=\"problem-letter\">{}</td>
                    <td class=\"problem-title\">{}</td>
                    <td class=\"attempts\">{}</td>
                    <td class=\"solves\">{}</td>
                    <td class=\"percentage\">{:.1}%</td>
                    <td class=\"first-solve\">{}</td>
                    <td class=\"first-time\">{}</td>
                </tr>",
                stat.problem_letter,
                stat.problem_title,
                stat.total_attempts,
                stat.total_solves,
                stat.solve_percentage,
                first_solve,
                first_solve_time
            ));
        }
        
        let html = format!(r#"
            <div class="problem-statistics">
                <h2>Problem Statistics</h2>
                
                <table class="stats-table">
                    <thead>
                        <tr>
                            <th>Problem</th>
                            <th>Title</th>
                            <th>Attempts</th>
                            <th>Solves</th>
                            <th>Success Rate</th>
                            <th>First Solve</th>
                            <th>Time</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
            
            <style>
                .problem-statistics {{
                    padding: 20px;
                }}
                
                .stats-table {{
                    width: 100%;
                    border-collapse: collapse;
                    background: white;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                }}
                
                .stats-table th, .stats-table td {{
                    padding: 12px;
                    text-align: left;
                    border-bottom: 1px solid #ddd;
                }}
                
                .stats-table th {{
                    background: #f8f9fa;
                    font-weight: 600;
                }}
                
                .problem-letter {{
                    font-weight: bold;
                    text-align: center;
                    width: 80px;
                }}
                
                .attempts, .solves, .percentage {{
                    text-align: center;
                    width: 100px;
                }}
                
                .first-solve {{
                    font-weight: 500;
                }}
                
                .first-time {{
                    font-family: monospace;
                    text-align: center;
                }}
            </style>
        "#, stats_html);
        
        Ok(html)
    }

    async fn render_balloon_queue(&self, props: &serde_json::Value) -> PluginResult<String> {
        let contest_id = props.get("contest_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let html = format!(r#"
            <div class="balloon-queue" data-contest-id="{}">
                <h2>🎈 Balloon Queue</h2>
                
                <div class="queue-stats">
                    <div class="stat-card">
                        <div class="stat-number" id="pending-count">0</div>
                        <div class="stat-label">Pending</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-number" id="delivered-count">0</div>
                        <div class="stat-label">Delivered</div>
                    </div>
                </div>
                
                <div class="balloon-list" id="balloon-list">
                    Loading balloons...
                </div>
            </div>
            
            <style>
                .balloon-queue {{
                    padding: 20px;
                }}
                
                .queue-stats {{
                    display: flex;
                    gap: 20px;
                    margin-bottom: 30px;
                }}
                
                .stat-card {{
                    background: white;
                    padding: 20px;
                    border-radius: 10px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    text-align: center;
                    min-width: 120px;
                }}
                
                .stat-number {{
                    font-size: 2.5em;
                    font-weight: bold;
                    color: #007bff;
                }}
                
                .stat-label {{
                    color: #666;
                    margin-top: 10px;
                    text-transform: uppercase;
                    font-size: 0.9em;
                }}
                
                .balloon-list {{
                    background: white;
                    border-radius: 10px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    overflow: hidden;
                }}
                
                .balloon-item {{
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 15px 20px;
                    border-bottom: 1px solid #eee;
                }}
                
                .balloon-item:last-child {{
                    border-bottom: none;
                }}
                
                .balloon-info {{
                    flex: 1;
                }}
                
                .team-name {{
                    font-weight: bold;
                    color: #333;
                }}
                
                .problem-info {{
                    color: #666;
                    margin-top: 5px;
                }}
                
                .balloon-color {{
                    width: 30px;
                    height: 30px;
                    border-radius: 50%;
                    margin-right: 15px;
                }}
                
                .deliver-btn {{
                    background: #28a745;
                    color: white;
                    border: none;
                    padding: 8px 16px;
                    border-radius: 5px;
                    cursor: pointer;
                }}
                
                .deliver-btn:hover {{
                    background: #218838;
                }}
                
                .delivered {{
                    opacity: 0.6;
                }}
                
                .delivered .deliver-btn {{
                    background: #6c757d;
                    cursor: not-allowed;
                }}
            </style>
        "#, contest_id);
        
        Ok(html)
    }

    async fn render_contest_admin(&self, _props: &serde_json::Value) -> PluginResult<String> {
        let html = r#"
            <div class="contest-admin">
                <h2>Contest Administration</h2>
                
                <div class="admin-actions">
                    <div class="action-group">
                        <h3>Contest Control</h3>
                        <button class="admin-btn freeze-btn" onclick="freezeContest()">
                            🥶 Freeze Scoreboard
                        </button>
                        <button class="admin-btn unfreeze-btn" onclick="unfreezeContest()">
                            🔥 Unfreeze Scoreboard  
                        </button>
                        <button class="admin-btn refresh-btn" onclick="refreshScoreboard()">
                            🔄 Refresh Scoreboard
                        </button>
                    </div>
                    
                    <div class="action-group">
                        <h3>Reports</h3>
                        <button class="admin-btn" onclick="exportResults()">
                            📊 Export Results
                        </button>
                        <button class="admin-btn" onclick="generateCertificates()">
                            🏆 Generate Certificates
                        </button>
                        <button class="admin-btn" onclick="balloonReport()">
                            🎈 Balloon Report
                        </button>
                    </div>
                </div>
                
                <div class="admin-stats">
                    <div class="stats-grid">
                        <div class="stat-item">
                            <div class="stat-value" id="total-submissions">0</div>
                            <div class="stat-label">Total Submissions</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-value" id="pending-clarifications">0</div>
                            <div class="stat-label">Pending Clarifications</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-value" id="active-teams">0</div>
                            <div class="stat-label">Active Teams</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-value" id="system-health">✅</div>
                            <div class="stat-label">System Health</div>
                        </div>
                    </div>
                </div>
            </div>
            
            <style>
                .contest-admin {
                    padding: 20px;
                }
                
                .admin-actions {
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                    gap: 30px;
                    margin-bottom: 40px;
                }
                
                .action-group {
                    background: white;
                    padding: 20px;
                    border-radius: 10px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                }
                
                .action-group h3 {
                    margin: 0 0 20px 0;
                    color: #333;
                }
                
                .admin-btn {
                    display: block;
                    width: 100%;
                    padding: 12px;
                    margin-bottom: 10px;
                    border: none;
                    border-radius: 5px;
                    cursor: pointer;
                    font-size: 1em;
                    background: #007bff;
                    color: white;
                }
                
                .admin-btn:hover {
                    background: #0056b3;
                }
                
                .freeze-btn { background: #17a2b8; }
                .unfreeze-btn { background: #dc3545; }
                .refresh-btn { background: #28a745; }
                
                .admin-stats {
                    background: white;
                    padding: 20px;
                    border-radius: 10px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                }
                
                .stats-grid {
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                    gap: 20px;
                }
                
                .stat-item {
                    text-align: center;
                    padding: 20px;
                    border-radius: 5px;
                    border-left: 4px solid #007bff;
                    background: #f8f9fa;
                }
                
                .stat-value {
                    font-size: 2em;
                    font-weight: bold;
                    color: #333;
                }
                
                .stat-label {
                    color: #666;
                    margin-top: 10px;
                    text-transform: uppercase;
                    font-size: 0.9em;
                }
            </style>
        "#;
        
        Ok(html.to_string())
    }
}

// Default implementations for configuration structs
impl Default for IcpcConfig {
    fn default() -> Self {
        Self {
            max_teams: None,
            problems_per_page: 10,
            enable_clarifications: true,
            enable_balloons: true,
            auto_freeze_duration_minutes: Some(60), // Auto-freeze 1 hour before end
            show_pending_submissions: false,
            penalty_per_wrong_submission: 20,
        }
    }
}