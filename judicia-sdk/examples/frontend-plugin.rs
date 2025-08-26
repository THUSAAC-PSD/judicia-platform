//! Example plugin that demonstrates frontend component integration

use judicia_sdk::prelude::*;
use judicia_sdk::frontend::{components::*, *};
use std::collections::HashMap;

/// Frontend plugin that provides custom UI components
#[judicia_plugin]
pub struct FrontendPlugin {
    name: "frontend-plugin",
    version: "1.0.0",
    author: "Judicia Team",
    description: "Plugin that demonstrates frontend component integration",
    capabilities: [
        Capability::RegisterComponents,
        Capability::ReadDatabase,
        Capability::RegisterRoutes
    ]
}

impl PluginMethods for FrontendPlugin {
    async fn on_initialize(&mut self, context: &PluginContext) -> PluginResult<()> {
        info!("Frontend Plugin initialized");
        
        // Register custom components
        self.register_components(context).await?;
        
        // Register API routes for component data
        self.register_api_routes(context).await?;
        
        Ok(())
    }

    async fn on_event(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        match event.event_type.as_str() {
            "frontend.component_requested" => {
                self.handle_component_request(event).await?;
            }
            _ => {}
        }
        
        Ok(())
    }

    async fn on_render(&self, component: &str, props: &serde_json::Value) -> PluginResult<String> {
        match component {
            "submission-dashboard" => {
                self.render_submission_dashboard(props).await
            }
            "live-scoreboard" => {
                self.render_live_scoreboard(props).await
            }
            "problem-editor" => {
                self.render_problem_editor(props).await
            }
            _ => Err(PluginError::NotImplemented(format!("Component not found: {}", component)))
        }
    }

    async fn on_http_request(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/api/components/submission-stats") => {
                self.get_submission_stats(request).await
            }
            ("GET", "/api/components/scoreboard-data") => {
                self.get_scoreboard_data(request).await
            }
            ("POST", "/api/components/save-problem") => {
                self.save_problem(request).await
            }
            _ => Ok(crate::http::error_response(404, "API endpoint not found"))
        }
    }
}

impl FrontendPlugin {
    async fn register_components(&self, context: &PluginContext) -> PluginResult<()> {
        info!("Registering frontend components for plugin: {}", context.plugin_id);
        
        // In a real implementation, this would register components with the platform
        let components = vec![
            "submission-dashboard",
            "live-scoreboard", 
            "problem-editor"
        ];
        
        for component in components {
            info!("Registered component: {}", component);
        }
        
        Ok(())
    }

    async fn register_api_routes(&self, context: &PluginContext) -> PluginResult<()> {
        info!("Registering API routes for plugin: {}", context.plugin_id);
        
        // In a real implementation, this would register routes with the platform
        let routes = vec![
            ("GET", "/api/components/submission-stats"),
            ("GET", "/api/components/scoreboard-data"),
            ("POST", "/api/components/save-problem"),
        ];
        
        for (method, path) in routes {
            info!("Registered route: {} {}", method, path);
        }
        
        Ok(())
    }

    async fn handle_component_request(&self, event: &PlatformEvent) -> PluginResult<()> {
        let component_name = event.payload.get("component")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        info!("Component requested: {}", component_name);
        
        // Could pre-load data or emit events for component preparation
        
        Ok(())
    }

    async fn render_submission_dashboard(&self, props: &serde_json::Value) -> PluginResult<String> {
        info!("Rendering submission dashboard");
        
        let user_id = props.get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        // Build dashboard HTML
        let html = HtmlBuilder::new()
            .div(Some("submission-dashboard"), "")
            .div(Some("dashboard-header"), 
                &format!("<h2>Submission Dashboard</h2><p>User: {}</p>", html_escape(user_id)))
            .div(Some("dashboard-stats"), &self.build_stats_section())
            .div(Some("recent-submissions"), &self.build_recent_submissions())
            .raw(&self.generate_dashboard_script())
            .build();
        
        Ok(html)
    }

    async fn render_live_scoreboard(&self, props: &serde_json::Value) -> PluginResult<String> {
        info!("Rendering live scoreboard");
        
        let contest_id = props.get("contest_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        
        // Create scoreboard table
        let headers = vec!["Rank", "Team", "Score", "Time", "A", "B", "C", "D", "E"];
        let rows = vec![
            vec!["1", "TeamAlpha", "500", "2:45:30", "✓", "✓", "✓", "✓", "-"],
            vec!["2", "TeamBeta", "400", "3:12:15", "✓", "✓", "✓", "-", "-"],
            vec!["3", "TeamGamma", "300", "2:30:45", "✓", "✓", "-", "-", "-"],
        ];
        
        let html = HtmlBuilder::new()
            .div(Some("live-scoreboard"), "")
            .div(Some("scoreboard-header"), 
                &format!("<h2>Live Scoreboard</h2><p>Contest: {}</p>", html_escape(contest_id)))
            .table(headers, rows)
            .raw(&self.generate_scoreboard_script())
            .build();
        
        Ok(html)
    }

    async fn render_problem_editor(&self, props: &serde_json::Value) -> PluginResult<String> {
        info!("Rendering problem editor");
        
        let problem_id = props.get("problem_id")
            .and_then(|v| v.as_str());
        
        let mode = if problem_id.is_some() { "edit" } else { "create" };
        
        // Build problem editor form
        let form = SimpleForm::new("/api/components/save-problem", "Save Problem")
            .text_field("title", "Problem Title", true)
            .text_field("time_limit", "Time Limit (ms)", true)
            .text_field("memory_limit", "Memory Limit (KB)", true)
            .add_field(FormField {
                name: "statement".to_string(),
                label: "Problem Statement".to_string(),
                field_type: "textarea".to_string(),
                required: true,
                placeholder: Some("Enter problem description...".to_string()),
            });
        
        let mut html = HtmlBuilder::new()
            .div(Some("problem-editor"), "")
            .div(Some("editor-header"), 
                &format!("<h2>{} Problem</h2>", if mode == "edit" { "Edit" } else { "Create" }))
            .build();
        
        // Add form HTML (simplified)
        html.push_str(&format!(
            "<form class='problem-form'>
                <div class='form-group'>
                    <label for='title'>Title</label>
                    <input type='text' id='title' name='title' required />
                </div>
                <div class='form-group'>
                    <label for='statement'>Statement</label>
                    <textarea id='statement' name='statement' rows='10' required></textarea>
                </div>
                <div class='form-row'>
                    <div class='form-group'>
                        <label for='time_limit'>Time Limit (ms)</label>
                        <input type='number' id='time_limit' name='time_limit' value='1000' required />
                    </div>
                    <div class='form-group'>
                        <label for='memory_limit'>Memory Limit (KB)</label>
                        <input type='number' id='memory_limit' name='memory_limit' value='262144' required />
                    </div>
                </div>
                <button type='submit' class='btn btn-primary'>Save Problem</button>
            </form>"
        ));
        
        html.push_str(&self.generate_editor_script());
        html.push_str("</div>");
        
        Ok(html)
    }

    fn build_stats_section(&self) -> String {
        format!(
            "<div class='stats-grid'>
                <div class='stat-card'>
                    <h3>Total Submissions</h3>
                    <span class='stat-value'>42</span>
                </div>
                <div class='stat-card'>
                    <h3>Accepted</h3>
                    <span class='stat-value success'>28</span>
                </div>
                <div class='stat-card'>
                    <h3>Success Rate</h3>
                    <span class='stat-value'>66.7%</span>
                </div>
                <div class='stat-card'>
                    <h3>Best Rank</h3>
                    <span class='stat-value'>5</span>
                </div>
            </div>"
        )
    }

    fn build_recent_submissions(&self) -> String {
        let headers = vec!["Time", "Problem", "Language", "Verdict", "Score"];
        let rows = vec![
            vec!["2 min ago", "A. Sum Two Numbers", "C++17", "AC", "100"],
            vec!["15 min ago", "B. Array Sort", "Python3", "WA", "0"],
            vec!["1 hour ago", "A. Sum Two Numbers", "Java", "TLE", "0"],
        ];
        
        HtmlBuilder::new()
            .raw("<h3>Recent Submissions</h3>")
            .table(headers, rows)
            .build()
    }

    fn generate_dashboard_script(&self) -> String {
        "<script>
        (function() {
            // Auto-refresh dashboard every 30 seconds
            setInterval(function() {
                // In real implementation, would fetch new data
                console.log('Refreshing dashboard data...');
            }, 30000);
            
            // Add click handlers for stats cards
            document.querySelectorAll('.stat-card').forEach(function(card) {
                card.addEventListener('click', function() {
                    console.log('Stat card clicked:', this.querySelector('h3').textContent);
                });
            });
        })();
        </script>".to_string()
    }

    fn generate_scoreboard_script(&self) -> String {
        "<script>
        (function() {
            // Auto-refresh scoreboard every 10 seconds
            setInterval(function() {
                // In real implementation, would fetch updated scoreboard data
                console.log('Refreshing scoreboard...');
                
                // Could update table rows here
            }, 10000);
            
            // Add hover effects for table rows
            document.querySelectorAll('tbody tr').forEach(function(row) {
                row.addEventListener('mouseenter', function() {
                    this.style.backgroundColor = '#f0f8ff';
                });
                row.addEventListener('mouseleave', function() {
                    this.style.backgroundColor = '';
                });
            });
        })();
        </script>".to_string()
    }

    fn generate_editor_script(&self) -> String {
        "<script>
        (function() {
            const form = document.querySelector('.problem-form');
            if (form) {
                form.addEventListener('submit', function(e) {
                    e.preventDefault();
                    
                    const formData = new FormData(this);
                    const data = {
                        title: formData.get('title'),
                        statement: formData.get('statement'),
                        time_limit: parseInt(formData.get('time_limit')),
                        memory_limit: parseInt(formData.get('memory_limit'))
                    };
                    
                    // In real implementation, would send to API
                    console.log('Saving problem:', data);
                    
                    // Show success message
                    alert('Problem saved successfully!');
                });
            }
            
            // Auto-save draft every 30 seconds
            setInterval(function() {
                const title = document.getElementById('title').value;
                const statement = document.getElementById('statement').value;
                
                if (title || statement) {
                    console.log('Auto-saving draft...');
                    // In real implementation, would save draft to storage
                }
            }, 30000);
        })();
        </script>".to_string()
    }

    async fn get_submission_stats(&self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let user_id = request.query_params.get("user_id")
            .ok_or_else(|| PluginError::InvalidInput("Missing user_id parameter".to_string()))?;
        
        info!("Getting submission stats for user: {}", user_id);
        
        // In real implementation, would query database
        let stats = serde_json::json!({
            "user_id": user_id,
            "total_submissions": 42,
            "accepted": 28,
            "wrong_answer": 8,
            "time_limit_exceeded": 4,
            "runtime_error": 2,
            "success_rate": 66.7,
            "best_rank": 5,
            "problems_solved": 25,
            "contest_participation": 12
        });
        
        crate::http::json_response(200, &stats)
    }

    async fn get_scoreboard_data(&self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let contest_id = request.query_params.get("contest_id")
            .unwrap_or(&"default".to_string());
        
        info!("Getting scoreboard data for contest: {}", contest_id);
        
        // Mock scoreboard data
        let scoreboard = serde_json::json!({
            "contest_id": contest_id,
            "last_updated": crate::time::now().to_rfc3339(),
            "participants": [
                {
                    "rank": 1,
                    "team_id": "team1",
                    "team_name": "TeamAlpha",
                    "score": 500,
                    "time": "2:45:30",
                    "problems": {
                        "A": {"verdict": "AC", "time": "0:12:34", "attempts": 1},
                        "B": {"verdict": "AC", "time": "0:45:12", "attempts": 2},
                        "C": {"verdict": "AC", "time": "1:23:45", "attempts": 1},
                        "D": {"verdict": "AC", "time": "2:01:23", "attempts": 3},
                        "E": {"verdict": "-", "time": null, "attempts": 0}
                    }
                },
                {
                    "rank": 2,
                    "team_id": "team2",
                    "team_name": "TeamBeta",
                    "score": 400,
                    "time": "3:12:15",
                    "problems": {
                        "A": {"verdict": "AC", "time": "0:08:45", "attempts": 1},
                        "B": {"verdict": "AC", "time": "1:15:30", "attempts": 1},
                        "C": {"verdict": "AC", "time": "2:30:15", "attempts": 4},
                        "D": {"verdict": "-", "time": null, "attempts": 2},
                        "E": {"verdict": "-", "time": null, "attempts": 0}
                    }
                }
            ]
        });
        
        crate::http::json_response(200, &scoreboard)
    }

    async fn save_problem(&self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let body = request.body.as_ref()
            .ok_or_else(|| PluginError::InvalidInput("Missing request body".to_string()))?;
        
        let problem_data: serde_json::Value = serde_json::from_str(body)
            .map_err(|e| PluginError::SerializationError(e.to_string()))?;
        
        // Validate required fields
        crate::validation::validate_required_fields(
            &problem_data,
            &["title", "statement", "time_limit", "memory_limit"]
        )?;
        
        info!("Saving problem: {}", problem_data.get("title").unwrap());
        
        // In real implementation, would save to database
        let problem_id = Uuid::new_v4();
        
        let response = serde_json::json!({
            "success": true,
            "problem_id": problem_id.to_string(),
            "message": "Problem saved successfully"
        });
        
        crate::http::json_response(200, &response)
    }
}