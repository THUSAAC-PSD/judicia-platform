//! Example plugin that demonstrates advanced judging capabilities

use judicia_sdk::prelude::*;
use std::collections::HashMap;

/// Advanced judging plugin that handles special judging logic
#[judicia_plugin]
pub struct AdvancedJudgingPlugin {
    name: "advanced-judging",
    version: "2.1.0",
    author: "Judicia Team", 
    description: "Advanced judging plugin with custom evaluation logic",
    capabilities: [
        Capability::TriggerJudging,
        Capability::ReadDatabase,
        Capability::WriteDatabase,
        Capability::EmitEvent,
        Capability::FileStorage
    ]
}

impl PluginMethods for AdvancedJudgingPlugin {
    async fn on_initialize(&mut self, context: &PluginContext) -> PluginResult<()> {
        info!("Advanced Judging Plugin initialized");
        
        // Initialize custom judging configurations
        self.load_judging_configs(context).await?;
        
        Ok(())
    }

    async fn on_event(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        match event.event_type.as_str() {
            "submission.needs_judging" => {
                self.handle_judging_request(event).await?;
            }
            
            "problem.updated" => {
                self.handle_problem_update(event).await?;
            }
            
            _ => {}
        }
        
        Ok(())
    }

    async fn on_http_request(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        match (request.method.as_str(), request.path.as_str()) {
            ("POST", "/judge/special") => {
                self.handle_special_judge_request(request).await
            }
            
            ("GET", "/judge/stats") => {
                self.get_judging_statistics(request).await
            }
            
            ("POST", "/judge/rejudge") => {
                self.handle_rejudge_request(request).await
            }
            
            _ => Ok(crate::http::error_response(404, "Endpoint not found"))
        }
    }
}

impl AdvancedJudgingPlugin {
    async fn load_judging_configs(&mut self, context: &PluginContext) -> PluginResult<()> {
        // Load custom judging configurations from database or file storage
        info!("Loading judging configurations for plugin: {}", context.plugin_id);
        
        // In a real implementation, this would load from the platform's storage
        // For now, we'll just log that it's happening
        debug!("Judging configurations loaded successfully");
        
        Ok(())
    }

    async fn handle_judging_request(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        info!("Processing judging request from event: {}", event.id);
        
        // Extract submission details from event payload
        let submission_id = event.payload.get("submission_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| PluginError::InvalidInput("Missing submission_id".to_string()))?;
        
        let problem_id = event.payload.get("problem_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| PluginError::InvalidInput("Missing problem_id".to_string()))?;
        
        let language_id = event.payload.get("language_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| PluginError::InvalidInput("Missing language_id".to_string()))?;
        
        let source_code = event.payload.get("source_code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::InvalidInput("Missing source_code".to_string()))?;

        // Create a comprehensive judging request
        let judging_request = JudgingRequestBuilder::new(submission_id, problem_id, language_id)
            .source_code(source_code)
            .time_limit(5000) // 5 seconds
            .memory_limit(256 * 1024) // 256MB
            .priority(1)
            .metadata("judging_plugin", serde_json::Value::String("advanced-judging".to_string()))
            .metadata("custom_options", serde_json::json!({
                "use_special_judge": true,
                "enable_detailed_feedback": true,
                "timeout_handling": "strict"
            }))
            .build();

        // Add test cases (in real implementation, these would come from database)
        let test_cases = self.load_test_cases(problem_id).await?;
        
        info!("Submitting judging request for submission: {} with {} test cases", 
              submission_id, test_cases.len());

        // In a real implementation, this would trigger the actual judging
        // For now, we'll emit an event indicating the judging has been initiated
        let judging_started_event = serde_json::json!({
            "submission_id": submission_id.to_string(),
            "problem_id": problem_id.to_string(),
            "test_case_count": test_cases.len(),
            "judging_mode": "advanced",
            "estimated_time_seconds": 30
        });
        
        info!("Judging initiated: {}", judging_started_event);
        
        Ok(())
    }

    async fn handle_problem_update(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        info!("Problem updated: {}", event.payload);
        
        // Check if we need to invalidate any cached judging data
        if let Some(problem_id) = event.payload.get("problem_id") {
            info!("Invalidating cached data for problem: {}", problem_id);
            
            // In real implementation, would clear relevant caches
            self.clear_problem_cache(problem_id).await?;
        }
        
        Ok(())
    }

    async fn handle_special_judge_request(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        info!("Handling special judge request");
        
        // Parse request body
        let body = request.body.as_ref()
            .ok_or_else(|| PluginError::InvalidInput("Missing request body".to_string()))?;
        
        let request_data: serde_json::Value = serde_json::from_str(body)
            .map_err(|e| PluginError::SerializationError(e.to_string()))?;
        
        // Validate required fields
        crate::validation::validate_required_fields(
            &request_data, 
            &["submission_id", "expected_output", "actual_output"]
        )?;
        
        // Perform special judging logic
        let result = self.perform_special_judging(&request_data).await?;
        
        crate::http::json_response(200, &result)
    }

    async fn get_judging_statistics(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        info!("Getting judging statistics");
        
        // Check authorization
        let _token = crate::http::extract_bearer_token(&request.headers)
            .ok_or_else(|| PluginError::SecurityViolation("Missing authorization token".to_string()))?;
        
        // In real implementation, would query database for statistics
        let stats = serde_json::json!({
            "total_submissions_judged": 12_450,
            "average_judging_time_ms": 1_250,
            "success_rate": 0.985,
            "by_language": {
                "cpp": { "count": 8_200, "avg_time_ms": 1_100 },
                "python": { "count": 3_100, "avg_time_ms": 1_800 },
                "java": { "count": 1_150, "avg_time_ms": 1_500 }
            },
            "last_updated": crate::time::now().to_rfc3339()
        });
        
        crate::http::json_response(200, &stats)
    }

    async fn handle_rejudge_request(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        info!("Handling rejudge request");
        
        let body = request.body.as_ref()
            .ok_or_else(|| PluginError::InvalidInput("Missing request body".to_string()))?;
        
        let rejudge_data: serde_json::Value = serde_json::from_str(body)
            .map_err(|e| PluginError::SerializationError(e.to_string()))?;
        
        let submission_ids = rejudge_data.get("submission_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| PluginError::InvalidInput("Missing or invalid submission_ids".to_string()))?;
        
        let mut results = Vec::new();
        
        for id_value in submission_ids {
            if let Some(id_str) = id_value.as_str() {
                if let Ok(submission_id) = Uuid::parse_str(id_str) {
                    // Queue submission for rejudging
                    let result = self.queue_rejudging(submission_id).await?;
                    results.push(serde_json::json!({
                        "submission_id": id_str,
                        "status": "queued",
                        "queue_position": result.queue_position,
                        "estimated_time_minutes": result.estimated_minutes
                    }));
                }
            }
        }
        
        let response = serde_json::json!({
            "rejudge_requests": results,
            "total_queued": results.len()
        });
        
        crate::http::json_response(200, &response)
    }

    async fn load_test_cases(&self, problem_id: Uuid) -> PluginResult<Vec<TestCase>> {
        info!("Loading test cases for problem: {}", problem_id);
        
        // In real implementation, would query database
        // For now, return mock test cases
        let test_cases = vec![
            TestCase {
                id: Uuid::new_v4(),
                input: "3 4\n".to_string(),
                expected_output: "7\n".to_string(),
                points: 10,
                is_sample: true,
            },
            TestCase {
                id: Uuid::new_v4(),
                input: "10 20\n".to_string(),
                expected_output: "30\n".to_string(),
                points: 90,
                is_sample: false,
            },
        ];
        
        Ok(test_cases)
    }

    async fn clear_problem_cache(&self, problem_id: &serde_json::Value) -> PluginResult<()> {
        info!("Clearing cache for problem: {}", problem_id);
        
        // In real implementation, would clear cached test cases, 
        // judge configurations, etc.
        
        Ok(())
    }

    async fn perform_special_judging(&self, data: &serde_json::Value) -> PluginResult<serde_json::Value> {
        let expected = data.get("expected_output")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::InvalidInput("Missing expected_output".to_string()))?;
        
        let actual = data.get("actual_output")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::InvalidInput("Missing actual_output".to_string()))?;
        
        // Implement custom judging logic here
        // For example, floating point comparison, ignoring whitespace, etc.
        let is_correct = self.compare_outputs(expected, actual);
        let score = if is_correct { 100 } else { 0 };
        
        Ok(serde_json::json!({
            "verdict": if is_correct { "AC" } else { "WA" },
            "score": score,
            "message": if is_correct { 
                "Output is correct" 
            } else { 
                "Output does not match expected result" 
            },
            "details": {
                "comparison_method": "special_judge",
                "expected_length": expected.len(),
                "actual_length": actual.len()
            }
        }))
    }

    fn compare_outputs(&self, expected: &str, actual: &str) -> bool {
        // Implement sophisticated output comparison
        // For this example, just do trimmed string comparison
        expected.trim() == actual.trim()
    }

    async fn queue_rejudging(&self, submission_id: Uuid) -> PluginResult<RejudgeResult> {
        info!("Queueing submission for rejudging: {}", submission_id);
        
        // In real implementation, would add to judging queue
        Ok(RejudgeResult {
            queue_position: 5,
            estimated_minutes: 2,
        })
    }
}

struct RejudgeResult {
    queue_position: u32,
    estimated_minutes: u32,
}