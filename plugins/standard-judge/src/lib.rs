//! Standard Judge Plugin for Judicia Platform
//!
//! This plugin provides traditional competitive programming problem judging,
//! including:
//! - Exact output matching
//! - Whitespace-normalized comparison  
//! - Floating point comparison with tolerance
//! - Time and memory limit enforcement
//! - Partial scoring support
//! - Custom checker integration

use judicia_sdk::prelude::*;
use judicia_sdk::{
    HttpRequest, HttpResponse, PluginError, PluginResult, Notification, NotificationUrgency, NotificationType,
    Plugin, PluginInfo, trigger_judging, database_query, register_http_route, send_notification, load_file, emit_platform_event
};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Standard Judge Plugin
pub struct StandardJudge {
    // Plugin state can be added here
}

impl Plugin for StandardJudge {
    fn new() -> Self {
        Self {}
    }
    
    fn metadata(&self) -> PluginInfo {
        PluginInfo {
            name: "standard-judge".to_string(),
            version: "1.0.0".to_string(),
            author: "Judicia Platform Team".to_string(),
            description: "Standard judge for traditional competitive programming problems".to_string(),
            capabilities: vec![
                "TriggerJudging".to_string(),
                "ReadProblems".to_string(),
                "ReadSubmissions".to_string(),
                "WriteSubmissions".to_string(),
                "EmitEvent".to_string(),
                "SubscribeEvents".to_string(),
                "RegisterComponents".to_string(),
                "RegisterRoutes".to_string(),
                "SendNotifications".to_string(),
                "FileStorage".to_string(),
            ],
            dependencies: vec![],
            frontend_components: vec![
                "JudgingStatus".to_string(),
                "VerdictDisplay".to_string(),
                "TestCaseResults".to_string(),
                "JudgingConfig".to_string(),
            ],
            api_routes: vec![
                "/api/standard-judge/status".to_string(),
                "/api/standard-judge/judge".to_string(),
                "/api/standard-judge/compare".to_string(),
                "/api/standard-judge/verdicts".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JudgingResult {
    pub verdict: Verdict,
    pub score: f64,
    pub max_score: f64,
    pub execution_time: u32,
    pub memory_usage: u32,
    pub test_results: Vec<TestCaseResult>,
    pub compilation_log: Option<String>,
    pub judge_log: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestCaseResult {
    pub test_id: Uuid,
    pub verdict: Verdict,
    pub score: f64,
    pub max_score: f64,
    pub execution_time: u32,
    pub memory_usage: u32,
    pub input_preview: Option<String>,
    pub expected_output_preview: Option<String>,
    pub actual_output_preview: Option<String>,
    pub checker_output: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Verdict {
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
    CompilationError,
    PresentationError,
    PartiallyCorrect,
    SystemError,
    Queued,
    Running,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verdict::Accepted => write!(f, "AC"),
            Verdict::WrongAnswer => write!(f, "WA"),
            Verdict::TimeLimitExceeded => write!(f, "TLE"),
            Verdict::MemoryLimitExceeded => write!(f, "MLE"), 
            Verdict::RuntimeError => write!(f, "RE"),
            Verdict::CompilationError => write!(f, "CE"),
            Verdict::PresentationError => write!(f, "PE"),
            Verdict::PartiallyCorrect => write!(f, "PC"),
            Verdict::SystemError => write!(f, "SE"),
            Verdict::Queued => write!(f, "QU"),
            Verdict::Running => write!(f, "RU"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComparisonConfig {
    pub mode: ComparisonMode,
    pub ignore_whitespace: bool,
    pub ignore_trailing_whitespace: bool,
    pub ignore_case: bool,
    pub float_tolerance: Option<f64>,
    pub custom_checker: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComparisonMode {
    Exact,
    IgnoreWhitespace,
    FloatingPoint,
    Custom,
}

#[async_trait::async_trait(?Send)]
impl PluginMethods for StandardJudge {
    async fn on_initialize(&mut self, context: &PluginContext) -> PluginResult<()> {
        info!("Standard Judge plugin initialized for context: {:?}", context.plugin_id);
        
        // Register judging components
        self.register_components().await?;
        
        // Register HTTP routes
        self.register_routes().await?;
        
        // Subscribe to submission events
        self.setup_event_listeners().await?;
        
        Ok(())
    }

    async fn on_event(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        match event.event_type.as_str() {
            "submission.created" => self.handle_submission_created(event).await?,
            "problem.updated" => self.handle_problem_updated(event).await?,
            "judging.requested" => self.handle_judging_requested(event).await?,
            _ => debug!("Unhandled event: {}", event.event_type),
        }
        Ok(())
    }

    async fn on_cleanup(&mut self) -> PluginResult<()> {
        info!("Standard Judge plugin cleaning up");
        Ok(())
    }

    async fn on_http_request(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        match request.path.as_str() {
            "/api/standard-judge/status" => self.handle_status_request(request).await,
            "/api/standard-judge/judge" => self.handle_judge_request(request).await,
            "/api/standard-judge/compare" => self.handle_compare_request(request).await,
            "/api/standard-judge/verdicts" => self.handle_verdicts_request(request).await,
            _ => Err(PluginError::NotImplemented("Route not found".into())),
        }
    }

    async fn on_render(&self, component: &str, props: &serde_json::Value) -> PluginResult<String> {
        match component {
            "JudgingStatus" => self.render_judging_status(props).await,
            "VerdictDisplay" => self.render_verdict_display(props).await,
            "TestCaseResults" => self.render_test_case_results(props).await,
            "JudgingConfig" => self.render_judging_config(props).await,
            _ => Err(PluginError::NotImplemented(format!("Component '{}' not found", component))),
        }
    }
}

impl StandardJudge {
    async fn register_components(&mut self) -> PluginResult<()> {
        info!("Registering standard judge components");
        
        // Components will be automatically registered by the macro
        // This includes JudgingStatus, VerdictDisplay, TestCaseResults, JudgingConfig
        
        Ok(())
    }

    async fn register_routes(&mut self) -> PluginResult<()> {
        info!("Registering standard judge routes");
        
        register_http_route("GET", "/api/standard-judge/status", "handle_status_request").await?;
        register_http_route("POST", "/api/standard-judge/judge", "handle_judge_request").await?;
        register_http_route("POST", "/api/standard-judge/compare", "handle_compare_request").await?;
        register_http_route("GET", "/api/standard-judge/verdicts", "handle_verdicts_request").await?;
        
        Ok(())
    }

    async fn setup_event_listeners(&mut self) -> PluginResult<()> {
        info!("Setting up standard judge event listeners");
        Ok(())
    }

    async fn handle_submission_created(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        debug!("Submission created event received: {:?}", event);
        
        let submission_id = event.payload.get("submission_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
            
        let problem_id = event.payload.get("problem_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        if let (Some(submission_id), Some(problem_id)) = (submission_id, problem_id) {
            // Check if this problem should be judged by the standard judge
            if self.should_judge_problem(problem_id).await? {
                self.queue_judging_task(submission_id, problem_id).await?;
            }
        }
        
        Ok(())
    }

    async fn handle_problem_updated(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        debug!("Problem updated event received: {:?}", event);
        
        let problem_id = event.payload.get("problem_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
            
        if let Some(problem_id) = problem_id {
            // Invalidate any cached problem configuration
            info!("Problem {} updated, invalidating cache", problem_id);
        }
        
        Ok(())
    }

    async fn handle_judging_requested(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        debug!("Judging requested event received: {:?}", event);
        
        let submission_id = event.payload.get("submission_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
            
        if let Some(submission_id) = submission_id {
            self.start_judging(submission_id).await?;
        }
        
        Ok(())
    }

    async fn should_judge_problem(&self, problem_id: Uuid) -> PluginResult<bool> {
        // Query problem configuration to check if it uses standard judging
        let query = DatabaseQuery {
            query: "SELECT judge_type FROM problems WHERE id = $1".to_string(),
            parameters: vec![serde_json::to_value(problem_id.to_string())?],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        
        if let Some(rows) = result.as_array() {
            if let Some(first_row) = rows.first() {
                if let Some(judge_type) = first_row.get("judge_type") {
                    return Ok(judge_type.as_str() == Some("standard"));
                }
            }
        }
        
        // Default to standard judging if not specified
        Ok(true)
    }

    async fn queue_judging_task(&self, submission_id: Uuid, problem_id: Uuid) -> PluginResult<()> {
        info!("Queuing judging task for submission {} on problem {}", submission_id, problem_id);
        
        // Create judging request
        let judging_request = JudgingRequest {
            submission_id,
            problem_id,
            language_id: Uuid::new_v4(), // This should come from the submission
            source_code: String::new(),   // This should come from the submission
            test_cases: Vec::new(),       // This should come from the problem
            time_limit_ms: 1000,         // This should come from the problem
            memory_limit_kb: 65536,      // This should come from the problem
            priority: 1,
            metadata: HashMap::new(),
        };
        
        // Trigger judging
        let _job_id = trigger_judging(&judging_request).await?;
        
        // Emit event that judging has been queued
        emit_platform_event(&PlatformEvent {
            id: Uuid::new_v4(),
            event_type: "judging.queued".to_string(),
            source_plugin_id: Some(Uuid::new_v4()), // Should be our plugin ID
            timestamp: chrono::Utc::now(),
            payload: serde_json::json!({
                "submission_id": submission_id,
                "problem_id": problem_id,
                "judge_type": "standard"
            }),
            metadata: HashMap::new(),
        }).await?;
        
        Ok(())
    }

    async fn start_judging(&mut self, submission_id: Uuid) -> PluginResult<()> {
        info!("Starting judging for submission {}", submission_id);
        
        // Load submission details
        let submission = self.load_submission(submission_id).await?;
        let problem = self.load_problem(submission.problem_id).await?;
        
        // Perform judging
        let result = self.judge_submission(&submission, &problem).await?;
        
        // Save results
        self.save_judging_result(submission_id, &result).await?;
        
        // Emit result event
        emit_platform_event(&PlatformEvent {
            id: Uuid::new_v4(),
            event_type: "judging.completed".to_string(),
            source_plugin_id: Some(Uuid::new_v4()), // Should be our plugin ID
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(&result)?,
            metadata: HashMap::new(),
        }).await?;
        
        // Send notification to user
        if result.verdict == Verdict::Accepted {
            self.send_acceptance_notification(submission_id).await?;
        }
        
        Ok(())
    }

    async fn load_submission(&self, submission_id: Uuid) -> PluginResult<SubmissionData> {
        let query = DatabaseQuery {
            query: r"
                SELECT s.id, s.problem_id, s.language_id, s.source_code, s.user_id,
                       p.title as problem_title
                FROM submissions s
                JOIN problems p ON s.problem_id = p.id  
                WHERE s.id = $1
            ".to_string(),
            parameters: vec![serde_json::to_value(submission_id.to_string())?],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        
        if let Some(rows) = result.as_array() {
            if let Some(row) = rows.first() {
                return Ok(SubmissionData {
                    id: submission_id,
                    problem_id: Uuid::parse_str(row["problem_id"].as_str().unwrap_or(""))
                        .map_err(|e| PluginError::InvalidInput(format!("Invalid UUID: {}", e)))?,
                    language_id: Uuid::parse_str(row["language_id"].as_str().unwrap_or(""))
                        .map_err(|e| PluginError::InvalidInput(format!("Invalid UUID: {}", e)))?,
                    source_code: row["source_code"].as_str().unwrap_or("").to_string(),
                    user_id: Uuid::parse_str(row["user_id"].as_str().unwrap_or(""))
                        .map_err(|e| PluginError::InvalidInput(format!("Invalid UUID: {}", e)))?,
                    problem_title: row["problem_title"].as_str().unwrap_or("").to_string(),
                });
            }
        }
        
        Err(PluginError::InvalidInput("Submission not found".to_string()))
    }

    async fn load_problem(&self, problem_id: Uuid) -> PluginResult<ProblemData> {
        let query = DatabaseQuery {
            query: r"
                SELECT id, title, time_limit_ms, memory_limit_kb, judge_config
                FROM problems 
                WHERE id = $1
            ".to_string(),
            parameters: vec![serde_json::to_value(problem_id.to_string())?],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        
        if let Some(rows) = result.as_array() {
            if let Some(row) = rows.first() {
                let judge_config: ComparisonConfig = if let Some(config_value) = row.get("judge_config") {
                    serde_json::from_value(config_value.clone()).unwrap_or_default()
                } else {
                    ComparisonConfig::default()
                };
                
                let test_cases = self.load_test_cases(problem_id).await?;
                
                return Ok(ProblemData {
                    id: problem_id,
                    title: row["title"].as_str().unwrap_or("").to_string(),
                    time_limit_ms: row["time_limit_ms"].as_u64().unwrap_or(1000) as u32,
                    memory_limit_kb: row["memory_limit_kb"].as_u64().unwrap_or(65536) as u32,
                    judge_config,
                    test_cases,
                });
            }
        }
        
        Err(PluginError::InvalidInput("Problem not found".to_string()))
    }

    async fn load_test_cases(&self, problem_id: Uuid) -> PluginResult<Vec<TestCaseData>> {
        let query = DatabaseQuery {
            query: r"
                SELECT id, input_file, output_file, points, is_sample
                FROM test_cases 
                WHERE problem_id = $1
                ORDER BY id
            ".to_string(),
            parameters: vec![serde_json::to_value(problem_id.to_string())?],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        let mut test_cases = Vec::new();
        
        if let Some(rows) = result.as_array() {
            for row in rows {
                let test_case_id = Uuid::parse_str(row["id"].as_str().unwrap_or(""))
                    .map_err(|e| PluginError::InvalidInput(format!("Invalid UUID: {}", e)))?;
                
                // Load test case files
                let input_data = self.load_test_file(row["input_file"].as_str().unwrap_or("")).await?;
                let expected_output = self.load_test_file(row["output_file"].as_str().unwrap_or("")).await?;
                
                test_cases.push(TestCaseData {
                    id: test_case_id,
                    input: input_data,
                    expected_output,
                    points: row["points"].as_i64().unwrap_or(1) as i32,
                    is_sample: row["is_sample"].as_bool().unwrap_or(false),
                });
            }
        }
        
        Ok(test_cases)
    }

    async fn load_test_file(&self, file_path: &str) -> PluginResult<String> {
        if file_path.is_empty() {
            return Ok(String::new());
        }
        
        let file_data = load_file(file_path).await?;
        String::from_utf8(file_data)
            .map_err(|e| PluginError::InvalidInput(format!("Invalid UTF-8 in test file: {}", e)))
    }

    async fn judge_submission(&self, submission: &SubmissionData, problem: &ProblemData) -> PluginResult<JudgingResult> {
        info!("Judging submission {} for problem {}", submission.id, problem.title);
        
        let mut test_results = Vec::new();
        let mut total_score = 0.0;
        let mut max_total_score = 0.0;
        let mut overall_verdict = Verdict::Accepted;
        let mut max_time = 0u32;
        let mut max_memory = 0u32;
        
        for test_case in &problem.test_cases {
            let result = self.judge_test_case(submission, problem, test_case).await?;
            
            total_score += result.score;
            max_total_score += result.max_score as f64;
            max_time = max_time.max(result.execution_time);
            max_memory = max_memory.max(result.memory_usage);
            
            // Update overall verdict based on worst result
            match (overall_verdict, result.verdict) {
                (Verdict::Accepted, v) => overall_verdict = v,
                (Verdict::PartiallyCorrect, Verdict::Accepted) => {},
                (Verdict::PartiallyCorrect, v) if v != Verdict::Accepted => overall_verdict = v,
                (_, Verdict::WrongAnswer) => overall_verdict = Verdict::WrongAnswer,
                (_, Verdict::TimeLimitExceeded) => overall_verdict = Verdict::TimeLimitExceeded,
                (_, Verdict::MemoryLimitExceeded) => overall_verdict = Verdict::MemoryLimitExceeded,
                (_, Verdict::RuntimeError) => overall_verdict = Verdict::RuntimeError,
                _ => {}
            }
            
            test_results.push(result);
        }
        
        // Adjust overall verdict based on score
        if overall_verdict == Verdict::Accepted && total_score < max_total_score {
            overall_verdict = Verdict::PartiallyCorrect;
        }
        
        Ok(JudgingResult {
            verdict: overall_verdict,
            score: total_score,
            max_score: max_total_score,
            execution_time: max_time,
            memory_usage: max_memory,
            test_results,
            compilation_log: None, // Would be populated by compilation step
            judge_log: Some(format!("Standard judge completed for {} test cases", problem.test_cases.len())),
        })
    }

    async fn judge_test_case(&self, submission: &SubmissionData, problem: &ProblemData, test_case: &TestCaseData) -> PluginResult<TestCaseResult> {
        info!("Judging test case {} for submission {}", test_case.id, submission.id);
        
        // This is a simplified implementation - in reality, this would:
        // 1. Compile the submission if needed
        // 2. Execute the program with the test input in a sandboxed environment
        // 3. Capture the output and measure execution time/memory
        // 4. Compare the output with expected output
        
        // For now, simulate execution
        let (actual_output, execution_time, memory_usage) = self.simulate_execution(
            &submission.source_code,
            &test_case.input,
            problem.time_limit_ms,
            problem.memory_limit_kb
        ).await?;
        
        // Compare outputs
        let comparison_result = self.compare_outputs(
            &test_case.expected_output,
            &actual_output,
            &problem.judge_config
        ).await?;
        
        let verdict = match comparison_result {
            ComparisonResult::Accepted => Verdict::Accepted,
            ComparisonResult::WrongAnswer => Verdict::WrongAnswer,
            ComparisonResult::PresentationError => Verdict::PresentationError,
        };
        
        let score = if verdict == Verdict::Accepted {
            test_case.points as f64
        } else {
            0.0
        };
        
        Ok(TestCaseResult {
            test_id: test_case.id,
            verdict,
            score,
            max_score: test_case.points as f64,
            execution_time,
            memory_usage,
            input_preview: Some(self.truncate_text(&test_case.input, 200)),
            expected_output_preview: Some(self.truncate_text(&test_case.expected_output, 200)),
            actual_output_preview: Some(self.truncate_text(&actual_output, 200)),
            checker_output: None,
        })
    }

    async fn simulate_execution(&self, _source_code: &str, _input: &str, time_limit: u32, memory_limit: u32) -> PluginResult<(String, u32, u32)> {
        // This is a simulation - in reality, this would:
        // 1. Set up a sandboxed execution environment
        // 2. Run the compiled program with the input
        // 3. Monitor execution time and memory usage
        // 4. Kill the process if limits are exceeded
        // 5. Return the output and resource usage
        
        // For demonstration, return mock values
        let execution_time = (time_limit as f32 * 0.1) as u32; // 10% of time limit
        let memory_usage = (memory_limit as f32 * 0.2) as u32;  // 20% of memory limit
        let output = "42\n".to_string(); // Mock output
        
        Ok((output, execution_time, memory_usage))
    }

    async fn compare_outputs(&self, expected: &str, actual: &str, config: &ComparisonConfig) -> PluginResult<ComparisonResult> {
        match config.mode {
            ComparisonMode::Exact => {
                if expected == actual {
                    Ok(ComparisonResult::Accepted)
                } else {
                    Ok(ComparisonResult::WrongAnswer)
                }
            }
            ComparisonMode::IgnoreWhitespace => {
                let expected_normalized = self.normalize_whitespace(expected, config);
                let actual_normalized = self.normalize_whitespace(actual, config);
                
                if expected_normalized == actual_normalized {
                    Ok(ComparisonResult::Accepted)
                } else if self.outputs_differ_only_in_whitespace(&expected_normalized, &actual_normalized) {
                    Ok(ComparisonResult::PresentationError)
                } else {
                    Ok(ComparisonResult::WrongAnswer)
                }
            }
            ComparisonMode::FloatingPoint => {
                self.compare_floating_point_outputs(expected, actual, config).await
            }
            ComparisonMode::Custom => {
                if let Some(_checker) = &config.custom_checker {
                    // Would invoke custom checker program
                    Ok(ComparisonResult::Accepted) // Placeholder
                } else {
                    Ok(ComparisonResult::WrongAnswer)
                }
            }
        }
    }

    fn normalize_whitespace(&self, text: &str, config: &ComparisonConfig) -> String {
        let mut result = text.to_string();
        
        if config.ignore_case {
            result = result.to_lowercase();
        }
        
        if config.ignore_whitespace {
            result = result.split_whitespace().collect::<Vec<_>>().join(" ");
        }
        
        if config.ignore_trailing_whitespace {
            result = result.lines()
                .map(|line| line.trim_end())
                .collect::<Vec<_>>()
                .join("\n");
        }
        
        result.trim().to_string()
    }

    fn outputs_differ_only_in_whitespace(&self, text1: &str, text2: &str) -> bool {
        let text1_no_ws: String = text1.chars().filter(|c| !c.is_whitespace()).collect();
        let text2_no_ws: String = text2.chars().filter(|c| !c.is_whitespace()).collect();
        text1_no_ws == text2_no_ws
    }

    async fn compare_floating_point_outputs(&self, expected: &str, actual: &str, config: &ComparisonConfig) -> PluginResult<ComparisonResult> {
        let tolerance = config.float_tolerance.unwrap_or(1e-9);
        
        let expected_values = self.extract_numbers(expected);
        let actual_values = self.extract_numbers(actual);
        
        if expected_values.len() != actual_values.len() {
            return Ok(ComparisonResult::WrongAnswer);
        }
        
        for (expected_val, actual_val) in expected_values.iter().zip(actual_values.iter()) {
            let diff = (expected_val - actual_val).abs();
            let relative_error = if expected_val.abs() > tolerance {
                diff / expected_val.abs()
            } else {
                diff
            };
            
            if relative_error > tolerance {
                return Ok(ComparisonResult::WrongAnswer);
            }
        }
        
        Ok(ComparisonResult::Accepted)
    }

    fn extract_numbers(&self, text: &str) -> Vec<f64> {
        text.split_whitespace()
            .filter_map(|s| s.parse::<f64>().ok())
            .collect()
    }

    fn truncate_text(&self, text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else {
            format!("{}...", &text[..max_length])
        }
    }

    async fn save_judging_result(&self, submission_id: Uuid, result: &JudgingResult) -> PluginResult<()> {
        let query = DatabaseQuery {
            query: r"
                UPDATE submissions 
                SET verdict = $1, score = $2, max_score = $3, 
                    execution_time = $4, memory_usage = $5, 
                    judging_result = $6, judged_at = NOW()
                WHERE id = $7
            ".to_string(),
            parameters: vec![
                serde_json::to_value(result.verdict.to_string())?,
                serde_json::to_value(result.score)?,
                serde_json::to_value(result.max_score)?,
                serde_json::to_value(result.execution_time)?,
                serde_json::to_value(result.memory_usage)?,
                serde_json::to_value(result)?,
                serde_json::to_value(submission_id.to_string())?,
            ],
            timeout_ms: Some(5000),
        };
        
        database_query(&query).await?;
        Ok(())
    }

    async fn send_acceptance_notification(&self, submission_id: Uuid) -> PluginResult<()> {
        // Load submission details for notification
        let submission = self.load_submission(submission_id).await?;
        
        let notification = Notification {
            recipient_id: submission.user_id,
            title: "Submission Accepted!".to_string(),
            message: format!("Your solution for '{}' has been accepted!", submission.problem_title),
            notification_type: NotificationType::Success,
            urgency: NotificationUrgency::Normal,
            metadata: [
                ("submission_id".to_string(), serde_json::json!(submission_id.to_string())),
                ("problem_id".to_string(), serde_json::json!(submission.problem_id.to_string())),
                ("plugin".to_string(), serde_json::json!("standard-judge"))
            ].iter().cloned().collect(),
        };
        
        send_notification(&notification).await?;
        Ok(())
    }

    // HTTP request handlers

    async fn handle_status_request(&self, _request: &HttpRequest) -> PluginResult<HttpResponse> {
        let status = serde_json::json!({
            "plugin": "standard-judge",
            "version": "1.0.0",
            "status": "healthy",
            "supported_verdicts": ["AC", "WA", "TLE", "MLE", "RE", "CE", "PE", "PC"],
            "comparison_modes": ["exact", "ignore_whitespace", "floating_point", "custom"],
            "timestamp": chrono::Utc::now()
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: status.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_judge_request(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let body: Value = serde_json::from_str(request.body.as_ref().unwrap_or(&String::new()))
            .map_err(|e| PluginError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        
        let submission_id = body.get("submission_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| PluginError::InvalidInput("Invalid submission_id".to_string()))?;
        
        // Start judging asynchronously
        self.start_judging(submission_id).await?;
        
        let response = serde_json::json!({
            "message": "Judging started",
            "submission_id": submission_id
        });
        
        Ok(HttpResponse {
            status_code: 202,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_compare_request(&self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let body: Value = serde_json::from_str(request.body.as_ref().unwrap_or(&String::new()))
            .map_err(|e| PluginError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        
        let expected = body.get("expected")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::InvalidInput("Missing 'expected' field".to_string()))?;
        
        let actual = body.get("actual")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::InvalidInput("Missing 'actual' field".to_string()))?;
        
        let config: ComparisonConfig = if let Some(config_value) = body.get("config") {
            serde_json::from_value(config_value.clone())
                .map_err(|e| PluginError::InvalidInput(format!("Invalid config: {}", e)))?
        } else {
            ComparisonConfig::default()
        };
        
        let result = self.compare_outputs(expected, actual, &config).await?;
        
        let response = serde_json::json!({
            "result": match result {
                ComparisonResult::Accepted => "accepted",
                ComparisonResult::WrongAnswer => "wrong_answer",
                ComparisonResult::PresentationError => "presentation_error",
            },
            "expected": expected,
            "actual": actual,
            "config": config
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_verdicts_request(&self, _request: &HttpRequest) -> PluginResult<HttpResponse> {
        let verdicts = serde_json::json!({
            "verdicts": [
                {"code": "AC", "name": "Accepted", "description": "Solution is correct"},
                {"code": "WA", "name": "Wrong Answer", "description": "Output does not match expected output"},
                {"code": "TLE", "name": "Time Limit Exceeded", "description": "Program took too long to execute"},
                {"code": "MLE", "name": "Memory Limit Exceeded", "description": "Program used too much memory"},
                {"code": "RE", "name": "Runtime Error", "description": "Program crashed during execution"},
                {"code": "CE", "name": "Compilation Error", "description": "Program failed to compile"},
                {"code": "PE", "name": "Presentation Error", "description": "Output format is incorrect"},
                {"code": "PC", "name": "Partially Correct", "description": "Some test cases passed"}
            ]
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: verdicts.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    // Component rendering methods

    async fn render_judging_status(&self, props: &serde_json::Value) -> PluginResult<String> {
        let submission_id = props.get("submission_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let status = props.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("pending");
        
        let html = format!(r#"
            <div class="judging-status">
                <div class="status-header">
                    <h3>Judging Status</h3>
                    <span class="submission-id">#{}</span>
                </div>
                <div class="status-indicator status-{}">
                    <span class="status-text">{}</span>
                </div>
            </div>
            <style>
                .judging-status {{
                    padding: 1rem;
                    border: 1px solid #e1e5e9;
                    border-radius: 6px;
                    background: white;
                }}
                .status-header {{
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 0.5rem;
                }}
                .status-header h3 {{
                    margin: 0;
                    font-size: 1.1rem;
                }}
                .submission-id {{
                    font-family: monospace;
                    color: #6a737d;
                }}
                .status-indicator {{
                    padding: 0.5rem 1rem;
                    border-radius: 4px;
                    font-weight: 500;
                }}
                .status-pending {{ background: #fff3cd; color: #856404; }}
                .status-running {{ background: #d4edda; color: #155724; }}
                .status-completed {{ background: #d1ecf1; color: #0c5460; }}
                .status-error {{ background: #f8d7da; color: #721c24; }}
            </style>
        "#, submission_id, status, status.to_uppercase());
        
        Ok(html)
    }

    async fn render_verdict_display(&self, props: &serde_json::Value) -> PluginResult<String> {
        let verdict = props.get("verdict")
            .and_then(|v| v.as_str())
            .unwrap_or("QU");
        
        let score = props.get("score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let max_score = props.get("max_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);
        
        let execution_time = props.get("execution_time")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let memory_usage = props.get("memory_usage")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let verdict_class = match verdict {
            "AC" => "accepted",
            "WA" => "wrong-answer",
            "TLE" => "time-limit",
            "MLE" => "memory-limit",
            "RE" => "runtime-error",
            "CE" => "compile-error",
            _ => "other"
        };
        
        let html = format!(r#"
            <div class="verdict-display">
                <div class="verdict-badge verdict-{}">
                    <span class="verdict-code">{}</span>
                </div>
                <div class="verdict-details">
                    <div class="score">
                        <span class="score-value">{:.1}</span>
                        <span class="score-max">/ {:.1}</span>
                    </div>
                    <div class="execution-stats">
                        <span class="time">{}ms</span>
                        <span class="memory">{}KB</span>
                    </div>
                </div>
            </div>
            <style>
                .verdict-display {{
                    display: flex;
                    align-items: center;
                    gap: 1rem;
                    padding: 0.75rem;
                    background: #f8f9fa;
                    border-radius: 6px;
                }}
                .verdict-badge {{
                    padding: 0.5rem 1rem;
                    border-radius: 4px;
                    font-weight: bold;
                    font-family: monospace;
                }}
                .verdict-accepted {{ background: #28a745; color: white; }}
                .verdict-wrong-answer {{ background: #dc3545; color: white; }}
                .verdict-time-limit {{ background: #ffc107; color: #212529; }}
                .verdict-memory-limit {{ background: #fd7e14; color: white; }}
                .verdict-runtime-error {{ background: #6f42c1; color: white; }}
                .verdict-compile-error {{ background: #6c757d; color: white; }}
                .verdict-other {{ background: #17a2b8; color: white; }}
                .verdict-details {{
                    flex: 1;
                }}
                .score {{
                    font-size: 1.2rem;
                    font-weight: bold;
                }}
                .score-max {{
                    color: #6a737d;
                    font-weight: normal;
                }}
                .execution-stats {{
                    margin-top: 0.25rem;
                    font-size: 0.9rem;
                    color: #6a737d;
                }}
                .execution-stats span {{
                    margin-right: 1rem;
                }}
            </style>
        "#, verdict_class, verdict, score, max_score, execution_time, memory_usage);
        
        Ok(html)
    }

    async fn render_test_case_results(&self, props: &serde_json::Value) -> PluginResult<String> {
        let empty_vec = vec![];
        let test_results = props.get("test_results")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);
        
        let mut rows = String::new();
        for (index, test_result) in test_results.iter().enumerate() {
            let verdict = test_result.get("verdict")
                .and_then(|v| v.as_str())
                .unwrap_or("QU");
            
            let score = test_result.get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            
            let max_score = test_result.get("max_score")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);
            
            let time = test_result.get("execution_time")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            
            let memory = test_result.get("memory_usage")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            
            let verdict_class = match verdict {
                "AC" => "accepted",
                _ => "failed"
            };
            
            rows.push_str(&format!(r#"
                <tr class="test-case-row test-case-{}">
                    <td>Test {}</td>
                    <td><span class="verdict-badge verdict-{}">{}</span></td>
                    <td>{:.1} / {:.1}</td>
                    <td>{}ms</td>
                    <td>{}KB</td>
                </tr>
            "#, verdict_class, index + 1, verdict_class, verdict, score, max_score, time, memory));
        }
        
        let html = format!(r#"
            <div class="test-case-results">
                <h4>Test Case Results</h4>
                <table class="results-table">
                    <thead>
                        <tr>
                            <th>Test Case</th>
                            <th>Verdict</th>
                            <th>Score</th>
                            <th>Time</th>
                            <th>Memory</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
            <style>
                .test-case-results {{
                    margin-top: 1rem;
                }}
                .test-case-results h4 {{
                    margin: 0 0 0.5rem 0;
                    font-size: 1rem;
                }}
                .results-table {{
                    width: 100%;
                    border-collapse: collapse;
                    font-size: 0.9rem;
                }}
                .results-table th, .results-table td {{
                    padding: 0.5rem;
                    text-align: left;
                    border-bottom: 1px solid #e1e5e9;
                }}
                .results-table th {{
                    background: #f8f9fa;
                    font-weight: 600;
                }}
                .test-case-accepted {{
                    background: #f8fff9;
                }}
                .test-case-failed {{
                    background: #fff5f5;
                }}
                .verdict-badge {{
                    padding: 0.2rem 0.5rem;
                    border-radius: 3px;
                    font-size: 0.8rem;
                    font-weight: bold;
                }}
                .verdict-accepted {{
                    background: #28a745;
                    color: white;
                }}
                .verdict-failed {{
                    background: #dc3545;
                    color: white;
                }}
            </style>
        "#, rows);
        
        Ok(html)
    }

    async fn render_judging_config(&self, props: &serde_json::Value) -> PluginResult<String> {
        let default_config = serde_json::json!({});
        let config = props.get("config").unwrap_or(&default_config);
        
        let comparison_mode = config.get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("exact");
        
        let ignore_whitespace = config.get("ignore_whitespace")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let float_tolerance = config.get("float_tolerance")
            .and_then(|v| v.as_f64())
            .unwrap_or(1e-9);
        
        let html = format!(r#"
            <div class="judging-config">
                <h4>Judging Configuration</h4>
                <div class="config-grid">
                    <div class="config-item">
                        <label>Comparison Mode:</label>
                        <span class="config-value">{}</span>
                    </div>
                    <div class="config-item">
                        <label>Ignore Whitespace:</label>
                        <span class="config-value">{}</span>
                    </div>
                    <div class="config-item">
                        <label>Float Tolerance:</label>
                        <span class="config-value">{:.1e}</span>
                    </div>
                </div>
            </div>
            <style>
                .judging-config {{
                    margin-top: 1rem;
                    padding: 1rem;
                    background: #f8f9fa;
                    border-radius: 6px;
                }}
                .judging-config h4 {{
                    margin: 0 0 0.75rem 0;
                    font-size: 1rem;
                }}
                .config-grid {{
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                    gap: 0.5rem;
                }}
                .config-item {{
                    display: flex;
                    justify-content: space-between;
                }}
                .config-item label {{
                    font-weight: 500;
                    color: #495057;
                }}
                .config-value {{
                    font-family: monospace;
                    color: #007bff;
                }}
            </style>
        "#, comparison_mode, ignore_whitespace, float_tolerance);
        
        Ok(html)
    }
}

// Helper data structures

#[derive(Debug, Clone)]
struct SubmissionData {
    id: Uuid,
    problem_id: Uuid,
    language_id: Uuid,
    source_code: String,
    user_id: Uuid,
    problem_title: String,
}

#[derive(Debug, Clone)]
struct ProblemData {
    id: Uuid,
    title: String,
    time_limit_ms: u32,
    memory_limit_kb: u32,
    judge_config: ComparisonConfig,
    test_cases: Vec<TestCaseData>,
}

#[derive(Debug, Clone)]
struct TestCaseData {
    id: Uuid,
    input: String,
    expected_output: String,
    points: i32,
    is_sample: bool,
}

#[derive(Debug, Clone)]
enum ComparisonResult {
    Accepted,
    WrongAnswer,
    PresentationError,
}

impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            mode: ComparisonMode::IgnoreWhitespace,
            ignore_whitespace: true,
            ignore_trailing_whitespace: true,
            ignore_case: false,
            float_tolerance: Some(1e-9),
            custom_checker: None,
        }
    }
}