//! Announcement System Plugin for Judicia Platform
//!
//! This plugin provides comprehensive announcement and communication capabilities including:
//! - Contest-wide announcements and updates
//! - System maintenance notifications
//! - Emergency alerts and bulletins
//! - Scheduled announcement publishing
//! - Multi-language announcement support
//! - Rich media announcements (text, images, links)
//! - Announcement categories and targeting
//! - Read receipts and engagement tracking

use judicia_sdk::prelude::*;
use judicia_sdk::{
    HttpRequest, HttpResponse, PluginError, PluginResult, Notification, NotificationUrgency, NotificationType,
    Plugin, PluginInfo, database_query, register_http_route, send_notification, emit_platform_event
};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Announcement System Plugin
pub struct AnnouncementSystemPlugin {
    // Plugin state
    announcements: HashMap<Uuid, Announcement>,
    templates: HashMap<String, AnnouncementTemplate>,
    user_read_status: HashMap<(Uuid, Uuid), DateTime<Utc>>, // (user_id, announcement_id) -> read_at
}

impl Plugin for AnnouncementSystemPlugin {
    fn new() -> Self {
        Self {
            announcements: HashMap::new(),
            templates: HashMap::new(),
            user_read_status: HashMap::new(),
        }
    }
    
    fn metadata(&self) -> PluginInfo {
        PluginInfo {
            name: "announcement-system".to_string(),
            version: "1.0.0".to_string(),
            author: "Judicia Platform Team".to_string(),
            description: "Comprehensive announcement and communication system for contests and platform-wide updates".to_string(),
            capabilities: vec![
                "ReadUsers".to_string(),
                "ReadContests".to_string(),
                "WriteContests".to_string(),
                "EmitEvent".to_string(),
                "SubscribeEvents".to_string(),
                "RegisterComponents".to_string(),
                "RegisterRoutes".to_string(),
                "SendNotifications".to_string(),
                "AccessDatabase".to_string(),
                "FileStorage".to_string(),
            ],
            dependencies: vec![],
            frontend_components: vec![
                "AnnouncementBanner".to_string(),
                "AnnouncementList".to_string(),
                "AnnouncementComposer".to_string(),
                "AnnouncementViewer".to_string(),
                "AnnouncementDashboard".to_string(),
                "EmergencyAlert".to_string(),
                "MaintenanceNotice".to_string(),
                "AnnouncementSettings".to_string(),
            ],
            api_routes: vec![
                "/api/announcements".to_string(),
                "/api/announcements/{id}".to_string(),
                "/api/announcements/create".to_string(),
                "/api/announcements/publish".to_string(),
                "/api/announcements/schedule".to_string(),
                "/api/announcements/templates".to_string(),
                "/api/announcements/mark-read".to_string(),
                "/api/announcements/statistics".to_string(),
                "/api/announcements/emergency".to_string(),
                "/api/announcements/maintenance".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Announcement {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub content_type: ContentType,
    pub category: AnnouncementCategory,
    pub priority: AnnouncementPriority,
    pub target_audience: TargetAudience,
    pub author_id: Uuid,
    pub author_name: String,
    pub contest_id: Option<Uuid>,
    pub attachments: Vec<AnnouncementAttachment>,
    pub tags: Vec<String>,
    pub language: String,
    pub translations: HashMap<String, AnnouncementTranslation>,
    pub status: AnnouncementStatus,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub read_count: i32,
    pub engagement_stats: EngagementStats,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnnouncementTemplate {
    pub id: Uuid,
    pub name: String,
    pub title_template: String,
    pub content_template: String,
    pub category: AnnouncementCategory,
    pub priority: AnnouncementPriority,
    pub variables: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnnouncementTranslation {
    pub language: String,
    pub title: String,
    pub content: String,
    pub translated_at: DateTime<Utc>,
    pub translator_id: Option<Uuid>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnnouncementAttachment {
    pub id: Uuid,
    pub filename: String,
    pub file_path: String,
    pub content_type: String,
    pub file_size: i64,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngagementStats {
    pub views: i32,
    pub reads: i32,
    pub clicks: i32,
    pub shares: i32,
    pub reactions: HashMap<String, i32>, // reaction_type -> count
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ContentType {
    PlainText,
    Markdown,
    Html,
    RichText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AnnouncementCategory {
    General,
    Contest,
    System,
    Maintenance,
    Emergency,
    Technical,
    Social,
    Achievement,
    Policy,
    Educational,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum AnnouncementPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
    Emergency = 5,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TargetAudience {
    Everyone,
    Participants,
    Admins,
    Judges,
    Spectators,
    Contest(Uuid),
    Role(String),
    Users(Vec<Uuid>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AnnouncementStatus {
    Draft,
    Scheduled,
    Published,
    Archived,
    Expired,
    Deleted,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateAnnouncementRequest {
    pub title: String,
    pub content: String,
    pub content_type: ContentType,
    pub category: AnnouncementCategory,
    pub priority: AnnouncementPriority,
    pub target_audience: TargetAudience,
    pub contest_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub language: String,
    pub schedule_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub publish_immediately: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnnouncementStats {
    pub total_announcements: i32,
    pub published_announcements: i32,
    pub draft_announcements: i32,
    pub scheduled_announcements: i32,
    pub total_views: i32,
    pub total_reads: i32,
    pub average_read_rate: f64,
    pub announcements_by_category: HashMap<String, i32>,
    pub top_viewed_announcements: Vec<AnnouncementSummary>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnnouncementSummary {
    pub id: Uuid,
    pub title: String,
    pub category: AnnouncementCategory,
    pub published_at: Option<DateTime<Utc>>,
    pub views: i32,
    pub reads: i32,
    pub read_rate: f64,
}

#[async_trait(?Send)]
impl PluginMethods for AnnouncementSystemPlugin {
    async fn on_initialize(&mut self, context: &PluginContext) -> PluginResult<()> {
        info!("Announcement System plugin initialized for context: {:?}", context.plugin_id);
        
        // Register announcement components
        self.register_components().await?;
        
        // Register HTTP routes
        self.register_routes().await?;
        
        // Subscribe to platform events
        self.setup_event_listeners().await?;
        
        // Load existing announcements
        self.load_announcements().await?;
        
        // Load announcement templates
        self.load_templates().await?;
        
        // Start scheduled announcement processor
        self.start_scheduler().await?;
        
        Ok(())
    }

    async fn on_event(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        match event.event_type.as_str() {
            "contest.created" => self.handle_contest_created(event).await?,
            "contest.started" => self.handle_contest_started(event).await?,
            "contest.finished" => self.handle_contest_finished(event).await?,
            "system.maintenance" => self.handle_system_maintenance(event).await?,
            "emergency.alert" => self.handle_emergency_alert(event).await?,
            "icpc.first_solve" => self.handle_first_solve_achievement(event).await?,
            "user.achievement" => self.handle_user_achievement(event).await?,
            _ => debug!("Unhandled event: {}", event.event_type),
        }
        Ok(())
    }

    async fn on_cleanup(&mut self) -> PluginResult<()> {
        info!("Announcement System plugin cleaning up");
        self.announcements.clear();
        self.templates.clear();
        self.user_read_status.clear();
        Ok(())
    }

    async fn on_http_request(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        match request.method.as_str() {
            "GET" => match request.path.as_str() {
                "/api/announcements" => self.handle_list_announcements(request).await,
                "/api/announcements/templates" => self.handle_list_templates(request).await,
                "/api/announcements/statistics" => self.handle_get_statistics(request).await,
                path if path.starts_with("/api/announcements/") => {
                    let parts: Vec<&str> = path.split('/').collect();
                    if parts.len() >= 4 {
                        let announcement_id = parts[3];
                        self.handle_get_announcement(request, announcement_id).await
                    } else {
                        Err(PluginError::InvalidInput("Invalid announcement path".into()))
                    }
                },
                _ => Err(PluginError::NotImplemented("Route not found".into())),
            },
            "POST" => match request.path.as_str() {
                "/api/announcements/create" => self.handle_create_announcement(request).await,
                "/api/announcements/publish" => self.handle_publish_announcement(request).await,
                "/api/announcements/schedule" => self.handle_schedule_announcement(request).await,
                "/api/announcements/mark-read" => self.handle_mark_read(request).await,
                "/api/announcements/emergency" => self.handle_create_emergency(request).await,
                "/api/announcements/maintenance" => self.handle_create_maintenance(request).await,
                _ => Err(PluginError::NotImplemented("Route not found".into())),
            },
            "PUT" => {
                if request.path.starts_with("/api/announcements/") {
                    let parts: Vec<&str> = request.path.split('/').collect();
                    if parts.len() >= 4 {
                        let announcement_id = parts[3];
                        self.handle_update_announcement(request, announcement_id).await
                    } else {
                        Err(PluginError::InvalidInput("Invalid announcement path".into()))
                    }
                } else {
                    Err(PluginError::NotImplemented("Route not found".into()))
                }
            },
            "DELETE" => {
                if request.path.starts_with("/api/announcements/") {
                    let parts: Vec<&str> = request.path.split('/').collect();
                    if parts.len() >= 4 {
                        let announcement_id = parts[3];
                        self.handle_delete_announcement(request, announcement_id).await
                    } else {
                        Err(PluginError::InvalidInput("Invalid announcement path".into()))
                    }
                } else {
                    Err(PluginError::NotImplemented("Route not found".into()))
                }
            },
            _ => Err(PluginError::NotImplemented("Method not supported".into())),
        }
    }

    async fn on_render(&self, component: &str, props: &serde_json::Value) -> PluginResult<String> {
        match component {
            "AnnouncementBanner" => self.render_announcement_banner(props).await,
            "AnnouncementList" => self.render_announcement_list(props).await,
            "AnnouncementComposer" => self.render_announcement_composer(props).await,
            "AnnouncementViewer" => self.render_announcement_viewer(props).await,
            "AnnouncementDashboard" => self.render_announcement_dashboard(props).await,
            "EmergencyAlert" => self.render_emergency_alert(props).await,
            "MaintenanceNotice" => self.render_maintenance_notice(props).await,
            "AnnouncementSettings" => self.render_announcement_settings(props).await,
            _ => Err(PluginError::NotImplemented(format!("Component '{}' not found", component))),
        }
    }
}

impl AnnouncementSystemPlugin {
    async fn register_components(&mut self) -> PluginResult<()> {
        info!("Registering announcement system components");
        Ok(())
    }

    async fn register_routes(&mut self) -> PluginResult<()> {
        info!("Registering announcement system routes");
        
        // Announcement CRUD routes
        register_http_route("GET", "/api/announcements", "handle_list_announcements").await?;
        register_http_route("GET", "/api/announcements/{id}", "handle_get_announcement").await?;
        register_http_route("POST", "/api/announcements/create", "handle_create_announcement").await?;
        register_http_route("PUT", "/api/announcements/{id}", "handle_update_announcement").await?;
        register_http_route("DELETE", "/api/announcements/{id}", "handle_delete_announcement").await?;
        
        // Publishing and scheduling routes
        register_http_route("POST", "/api/announcements/publish", "handle_publish_announcement").await?;
        register_http_route("POST", "/api/announcements/schedule", "handle_schedule_announcement").await?;
        
        // Template management routes
        register_http_route("GET", "/api/announcements/templates", "handle_list_templates").await?;
        
        // User interaction routes
        register_http_route("POST", "/api/announcements/mark-read", "handle_mark_read").await?;
        
        // Analytics routes
        register_http_route("GET", "/api/announcements/statistics", "handle_get_statistics").await?;
        
        // Special announcement types
        register_http_route("POST", "/api/announcements/emergency", "handle_create_emergency").await?;
        register_http_route("POST", "/api/announcements/maintenance", "handle_create_maintenance").await?;
        
        Ok(())
    }

    async fn setup_event_listeners(&mut self) -> PluginResult<()> {
        info!("Setting up announcement system event listeners");
        Ok(())
    }

    async fn load_announcements(&mut self) -> PluginResult<()> {
        info!("Loading announcements");
        
        let query = DatabaseQuery {
            query: r"
                SELECT id, title, content, content_type, category, priority, target_audience,
                       author_id, author_name, contest_id, attachments, tags, language,
                       status, created_at, published_at, scheduled_at, expires_at, updated_at,
                       read_count, engagement_stats
                FROM announcements
                WHERE status != 'deleted'
                ORDER BY created_at DESC
                LIMIT 100
            ".to_string(),
            parameters: vec![],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        
        if let Some(rows) = result.as_array() {
            for row in rows {
                if let Ok(announcement) = self.parse_announcement_from_row(row) {
                    self.announcements.insert(announcement.id, announcement);
                }
            }
        }
        
        info!("Loaded {} announcements", self.announcements.len());
        Ok(())
    }

    async fn load_templates(&mut self) -> PluginResult<()> {
        info!("Loading announcement templates");
        
        let query = DatabaseQuery {
            query: r"
                SELECT id, name, title_template, content_template, category, priority,
                       variables, created_at, updated_at
                FROM announcement_templates
                ORDER BY name
            ".to_string(),
            parameters: vec![],
            timeout_ms: Some(5000),
        };
        
        let result = database_query(&query).await?;
        
        if let Some(rows) = result.as_array() {
            for row in rows {
                if let Ok(template) = self.parse_template_from_row(row) {
                    self.templates.insert(template.name.clone(), template);
                }
            }
        }
        
        // Create default templates if none exist
        if self.templates.is_empty() {
            self.create_default_templates().await?;
        }
        
        info!("Loaded {} announcement templates", self.templates.len());
        Ok(())
    }

    async fn create_default_templates(&mut self) -> PluginResult<()> {
        info!("Creating default announcement templates");
        
        let default_templates = vec![
            // Contest templates
            ("contest_announcement", AnnouncementCategory::Contest, AnnouncementPriority::High,
             "Contest Update: {{contest_title}}", "{{message}}"),
            
            ("contest_clarification", AnnouncementCategory::Contest, AnnouncementPriority::Normal,
             "Clarification for {{contest_title}}", "**Question:** {{question}}\n\n**Answer:** {{answer}}"),
            
            ("contest_results", AnnouncementCategory::Contest, AnnouncementPriority::Normal,
             "Final Results: {{contest_title}}", "The contest has ended! View the final results and standings."),
             
            // System templates
            ("maintenance_notice", AnnouncementCategory::Maintenance, AnnouncementPriority::High,
             "Scheduled Maintenance: {{date}}", "The platform will undergo maintenance from {{start_time}} to {{end_time}}. {{details}}"),
             
            ("system_update", AnnouncementCategory::System, AnnouncementPriority::Normal,
             "System Update", "We've updated the platform with new features and improvements. {{changes}}"),
             
            ("emergency_alert", AnnouncementCategory::Emergency, AnnouncementPriority::Emergency,
             "🚨 Emergency Alert", "{{message}} Please take immediate action."),
             
            // Achievement templates
            ("achievement_unlock", AnnouncementCategory::Achievement, AnnouncementPriority::Normal,
             "🏆 Achievement Unlocked!", "Congratulations to {{user_name}} for earning the {{achievement_name}} achievement!"),
             
            ("first_solve_celebration", AnnouncementCategory::Achievement, AnnouncementPriority::Normal,
             "🎉 First Solve!", "{{team_name}} achieved the first solve for {{problem_title}}! Congratulations!"),
        ];
        
        for (name, category, priority, title_template, content_template) in default_templates {
            let template = AnnouncementTemplate {
                id: Uuid::new_v4(),
                name: name.to_string(),
                title_template: title_template.to_string(),
                content_template: content_template.to_string(),
                category,
                priority,
                variables: self.extract_template_variables(content_template),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            
            // Save to database
            self.save_template(&template).await?;
            self.templates.insert(name.to_string(), template);
        }
        
        Ok(())
    }

    fn extract_template_variables(&self, template: &str) -> Vec<String> {
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        re.captures_iter(template)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    async fn save_template(&self, template: &AnnouncementTemplate) -> PluginResult<()> {
        let query = DatabaseQuery {
            query: r"
                INSERT INTO announcement_templates 
                (id, name, title_template, content_template, category, priority, variables, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (name) DO UPDATE SET
                title_template = $3, content_template = $4, category = $5, 
                priority = $6, variables = $7, updated_at = $9
            ".to_string(),
            parameters: vec![
                serde_json::to_value(template.id.to_string())?,
                serde_json::to_value(&template.name)?,
                serde_json::to_value(&template.title_template)?,
                serde_json::to_value(&template.content_template)?,
                serde_json::to_value(&template.category)?,
                serde_json::to_value(&template.priority)?,
                serde_json::to_value(&template.variables)?,
                serde_json::to_value(template.created_at.to_rfc3339())?,
                serde_json::to_value(template.updated_at.to_rfc3339())?,
            ],
            timeout_ms: Some(5000),
        };
        
        database_query(&query).await?;
        Ok(())
    }

    async fn start_scheduler(&mut self) -> PluginResult<()> {
        info!("Starting announcement scheduler");
        
        // Process scheduled announcements
        self.process_scheduled_announcements().await?;
        
        Ok(())
    }

    async fn process_scheduled_announcements(&mut self) -> PluginResult<()> {
        let now = Utc::now();
        let scheduled_announcements: Vec<Uuid> = self.announcements
            .values()
            .filter(|a| a.status == AnnouncementStatus::Scheduled)
            .filter(|a| a.scheduled_at.map_or(false, |scheduled| scheduled <= now))
            .map(|a| a.id)
            .collect();
        
        for announcement_id in scheduled_announcements {
            self.publish_announcement_by_id(announcement_id).await?;
        }
        
        Ok(())
    }

    async fn publish_announcement_by_id(&mut self, announcement_id: Uuid) -> PluginResult<()> {
        // Clone the announcement to avoid borrowing issues
        let announcement_clone = if let Some(announcement) = self.announcements.get_mut(&announcement_id) {
            announcement.status = AnnouncementStatus::Published;
            announcement.published_at = Some(Utc::now());
            announcement.clone()
        } else {
            return Ok(());
        };
        
        // Save to database
        self.save_announcement(&announcement_clone).await?;
        
        // Emit publication event
        emit_platform_event(&PlatformEvent {
            id: Uuid::new_v4(),
            event_type: "announcement.published".to_string(),
            source_plugin_id: Some(Uuid::new_v4()),
            timestamp: Utc::now(),
            payload: serde_json::to_value(&announcement_clone)?,
            metadata: HashMap::new(),
        }).await?;
        
        // Send notifications to target audience
        self.send_announcement_notifications(&announcement_clone).await?;
        
        info!("Published announcement: {}", announcement_clone.title);
        
        Ok(())
    }

    async fn save_announcement(&self, announcement: &Announcement) -> PluginResult<()> {
        let query = DatabaseQuery {
            query: r"
                INSERT INTO announcements 
                (id, title, content, content_type, category, priority, target_audience,
                 author_id, author_name, contest_id, attachments, tags, language,
                 status, created_at, published_at, scheduled_at, expires_at, updated_at,
                 read_count, engagement_stats)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
                ON CONFLICT (id) DO UPDATE SET
                title = $2, content = $3, content_type = $4, category = $5, priority = $6,
                target_audience = $7, attachments = $11, tags = $12, status = $14,
                published_at = $16, scheduled_at = $17, expires_at = $18, updated_at = $19,
                read_count = $20, engagement_stats = $21
            ".to_string(),
            parameters: vec![
                serde_json::to_value(announcement.id.to_string())?,
                serde_json::to_value(&announcement.title)?,
                serde_json::to_value(&announcement.content)?,
                serde_json::to_value(&announcement.content_type)?,
                serde_json::to_value(&announcement.category)?,
                serde_json::to_value(&announcement.priority)?,
                serde_json::to_value(&announcement.target_audience)?,
                serde_json::to_value(announcement.author_id.to_string())?,
                serde_json::to_value(&announcement.author_name)?,
                serde_json::to_value(announcement.contest_id.map(|id| id.to_string()))?,
                serde_json::to_value(&announcement.attachments)?,
                serde_json::to_value(&announcement.tags)?,
                serde_json::to_value(&announcement.language)?,
                serde_json::to_value(&announcement.status)?,
                serde_json::to_value(announcement.created_at.to_rfc3339())?,
                serde_json::to_value(announcement.published_at.map(|t| t.to_rfc3339()))?,
                serde_json::to_value(announcement.scheduled_at.map(|t| t.to_rfc3339()))?,
                serde_json::to_value(announcement.expires_at.map(|t| t.to_rfc3339()))?,
                serde_json::to_value(announcement.updated_at.to_rfc3339())?,
                serde_json::to_value(announcement.read_count)?,
                serde_json::to_value(&announcement.engagement_stats)?,
            ],
            timeout_ms: Some(5000),
        };
        
        database_query(&query).await?;
        Ok(())
    }

    async fn send_announcement_notifications(&self, announcement: &Announcement) -> PluginResult<()> {
        // Get target users based on audience
        let target_users = self.get_target_users(&announcement.target_audience).await?;
        
        for user_id in target_users {
            let notification = Notification {
                recipient_id: user_id,
                title: format!("📢 {}", announcement.title),
                message: self.truncate_content(&announcement.content, 200),
                notification_type: self.priority_to_notification_type(announcement.priority),
                urgency: self.priority_to_urgency(announcement.priority),
                metadata: [
                    ("announcement_id".to_string(), serde_json::json!(announcement.id.to_string())),
                    ("category".to_string(), serde_json::json!(announcement.category)),
                    ("priority".to_string(), serde_json::json!(announcement.priority)),
                ].iter().cloned().collect(),
            };
            
            send_notification(&notification).await?;
        }
        
        Ok(())
    }

    async fn get_target_users(&self, target_audience: &TargetAudience) -> PluginResult<Vec<Uuid>> {
        let query = match target_audience {
            TargetAudience::Everyone => DatabaseQuery {
                query: "SELECT id FROM users".to_string(),
                parameters: vec![],
                timeout_ms: Some(5000),
            },
            TargetAudience::Participants => DatabaseQuery {
                query: "SELECT DISTINCT user_id as id FROM contest_participants".to_string(),
                parameters: vec![],
                timeout_ms: Some(5000),
            },
            TargetAudience::Admins => DatabaseQuery {
                query: "SELECT id FROM users WHERE role = 'admin'".to_string(),
                parameters: vec![],
                timeout_ms: Some(5000),
            },
            TargetAudience::Judges => DatabaseQuery {
                query: "SELECT id FROM users WHERE role = 'judge'".to_string(),
                parameters: vec![],
                timeout_ms: Some(5000),
            },
            TargetAudience::Spectators => DatabaseQuery {
                query: "SELECT id FROM users WHERE role = 'spectator'".to_string(),
                parameters: vec![],
                timeout_ms: Some(5000),
            },
            TargetAudience::Contest(contest_id) => DatabaseQuery {
                query: "SELECT user_id as id FROM contest_participants WHERE contest_id = $1".to_string(),
                parameters: vec![serde_json::to_value(contest_id.to_string())?],
                timeout_ms: Some(5000),
            },
            TargetAudience::Role(role) => DatabaseQuery {
                query: "SELECT id FROM users WHERE role = $1".to_string(),
                parameters: vec![serde_json::to_value(role)?],
                timeout_ms: Some(5000),
            },
            TargetAudience::Users(user_ids) => {
                return Ok(user_ids.clone());
            },
        };
        
        let result = database_query(&query).await?;
        let mut target_users = Vec::new();
        
        if let Some(rows) = result.as_array() {
            for row in rows {
                if let Some(id_str) = row.get("id").and_then(|v| v.as_str()) {
                    if let Ok(user_id) = Uuid::parse_str(id_str) {
                        target_users.push(user_id);
                    }
                }
            }
        }
        
        Ok(target_users)
    }

    fn priority_to_notification_type(&self, priority: AnnouncementPriority) -> NotificationType {
        match priority {
            AnnouncementPriority::Emergency => NotificationType::Error,
            AnnouncementPriority::Critical => NotificationType::Warning,
            AnnouncementPriority::High => NotificationType::Success,
            AnnouncementPriority::Normal => NotificationType::Info,
            AnnouncementPriority::Low => NotificationType::Info,
        }
    }

    fn priority_to_urgency(&self, priority: AnnouncementPriority) -> NotificationUrgency {
        match priority {
            AnnouncementPriority::Emergency => NotificationUrgency::Critical,
            AnnouncementPriority::Critical => NotificationUrgency::Critical,
            AnnouncementPriority::High => NotificationUrgency::High,
            AnnouncementPriority::Normal => NotificationUrgency::Normal,
            AnnouncementPriority::Low => NotificationUrgency::Low,
        }
    }

    fn truncate_content(&self, content: &str, max_length: usize) -> String {
        if content.len() <= max_length {
            content.to_string()
        } else {
            format!("{}...", &content[..max_length])
        }
    }

    // Event handlers
    
    async fn handle_contest_created(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        if let (Some(contest_id), Some(contest_title)) = (
            event.payload.get("contest_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
            event.payload.get("contest_title").and_then(|v| v.as_str())
        ) {
            let mut variables = HashMap::new();
            variables.insert("contest_title".to_string(), serde_json::json!(contest_title));
            
            self.create_templated_announcement(
                "contest_announcement",
                &variables,
                TargetAudience::Everyone,
                Some(contest_id),
                true, // publish immediately
            ).await?;
        }
        Ok(())
    }

    async fn handle_contest_started(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        if let (Some(contest_id), Some(contest_title)) = (
            event.payload.get("contest_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
            event.payload.get("contest_title").and_then(|v| v.as_str())
        ) {
            let announcement = Announcement {
                id: Uuid::new_v4(),
                title: format!("🚀 Contest Started: {}", contest_title),
                content: format!("The contest '{}' has officially started! Good luck to all participants!", contest_title),
                content_type: ContentType::Markdown,
                category: AnnouncementCategory::Contest,
                priority: AnnouncementPriority::High,
                target_audience: TargetAudience::Contest(contest_id),
                author_id: Uuid::nil(), // System generated
                author_name: "System".to_string(),
                contest_id: Some(contest_id),
                attachments: vec![],
                tags: vec!["contest".to_string(), "start".to_string()],
                language: "en".to_string(),
                translations: HashMap::new(),
                status: AnnouncementStatus::Published,
                created_at: Utc::now(),
                published_at: Some(Utc::now()),
                scheduled_at: None,
                expires_at: None,
                updated_at: Utc::now(),
                read_count: 0,
                engagement_stats: EngagementStats {
                    views: 0,
                    reads: 0,
                    clicks: 0,
                    shares: 0,
                    reactions: HashMap::new(),
                },
            };
            
            self.save_announcement(&announcement).await?;
            self.announcements.insert(announcement.id, announcement.clone());
            self.send_announcement_notifications(&announcement).await?;
        }
        Ok(())
    }

    async fn handle_contest_finished(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        if let (Some(contest_id), Some(contest_title)) = (
            event.payload.get("contest_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()),
            event.payload.get("contest_title").and_then(|v| v.as_str())
        ) {
            let mut variables = HashMap::new();
            variables.insert("contest_title".to_string(), serde_json::json!(contest_title));
            
            self.create_templated_announcement(
                "contest_results",
                &variables,
                TargetAudience::Contest(contest_id),
                Some(contest_id),
                true,
            ).await?;
        }
        Ok(())
    }

    async fn handle_system_maintenance(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        let details = event.payload.get("details").and_then(|v| v.as_str()).unwrap_or("System maintenance");
        let start_time = event.payload.get("start_time").and_then(|v| v.as_str()).unwrap_or("TBD");
        let end_time = event.payload.get("end_time").and_then(|v| v.as_str()).unwrap_or("TBD");
        
        let mut variables = HashMap::new();
        variables.insert("details".to_string(), serde_json::json!(details));
        variables.insert("start_time".to_string(), serde_json::json!(start_time));
        variables.insert("end_time".to_string(), serde_json::json!(end_time));
        variables.insert("date".to_string(), serde_json::json!(Utc::now().format("%Y-%m-%d").to_string()));
        
        self.create_templated_announcement(
            "maintenance_notice",
            &variables,
            TargetAudience::Everyone,
            None,
            true,
        ).await?;
        
        Ok(())
    }

    async fn handle_emergency_alert(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        let message = event.payload.get("message").and_then(|v| v.as_str()).unwrap_or("Emergency alert");
        
        let mut variables = HashMap::new();
        variables.insert("message".to_string(), serde_json::json!(message));
        
        self.create_templated_announcement(
            "emergency_alert",
            &variables,
            TargetAudience::Everyone,
            None,
            true,
        ).await?;
        
        Ok(())
    }

    async fn handle_first_solve_achievement(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        let team_name = event.payload.get("team_name").and_then(|v| v.as_str()).unwrap_or("Unknown Team");
        let problem_title = event.payload.get("problem_title").and_then(|v| v.as_str()).unwrap_or("Unknown Problem");
        
        let mut variables = HashMap::new();
        variables.insert("team_name".to_string(), serde_json::json!(team_name));
        variables.insert("problem_title".to_string(), serde_json::json!(problem_title));
        
        self.create_templated_announcement(
            "first_solve_celebration",
            &variables,
            TargetAudience::Everyone,
            None,
            true,
        ).await?;
        
        Ok(())
    }

    async fn handle_user_achievement(&mut self, event: &PlatformEvent) -> PluginResult<()> {
        let user_name = event.payload.get("user_name").and_then(|v| v.as_str()).unwrap_or("Unknown User");
        let achievement_name = event.payload.get("achievement_name").and_then(|v| v.as_str()).unwrap_or("Achievement");
        
        let mut variables = HashMap::new();
        variables.insert("user_name".to_string(), serde_json::json!(user_name));
        variables.insert("achievement_name".to_string(), serde_json::json!(achievement_name));
        
        self.create_templated_announcement(
            "achievement_unlock",
            &variables,
            TargetAudience::Everyone,
            None,
            true,
        ).await?;
        
        Ok(())
    }

    async fn create_templated_announcement(
        &mut self,
        template_name: &str,
        variables: &HashMap<String, Value>,
        target_audience: TargetAudience,
        contest_id: Option<Uuid>,
        publish_immediately: bool,
    ) -> PluginResult<Uuid> {
        if let Some(template) = self.templates.get(template_name) {
            let title = self.render_template(&template.title_template, variables);
            let content = self.render_template(&template.content_template, variables);
            
            let announcement = Announcement {
                id: Uuid::new_v4(),
                title,
                content,
                content_type: ContentType::Markdown,
                category: template.category,
                priority: template.priority,
                target_audience,
                author_id: Uuid::nil(), // System generated
                author_name: "System".to_string(),
                contest_id,
                attachments: vec![],
                tags: vec![template_name.to_string()],
                language: "en".to_string(),
                translations: HashMap::new(),
                status: if publish_immediately { AnnouncementStatus::Published } else { AnnouncementStatus::Draft },
                created_at: Utc::now(),
                published_at: if publish_immediately { Some(Utc::now()) } else { None },
                scheduled_at: None,
                expires_at: None,
                updated_at: Utc::now(),
                read_count: 0,
                engagement_stats: EngagementStats {
                    views: 0,
                    reads: 0,
                    clicks: 0,
                    shares: 0,
                    reactions: HashMap::new(),
                },
            };
            
            self.save_announcement(&announcement).await?;
            let announcement_id = announcement.id;
            self.announcements.insert(announcement_id, announcement.clone());
            
            if publish_immediately {
                self.send_announcement_notifications(&announcement).await?;
            }
            
            Ok(announcement_id)
        } else {
            Err(PluginError::InvalidInput(format!("Template '{}' not found", template_name)))
        }
    }

    fn render_template(&self, template: &str, variables: &HashMap<String, Value>) -> String {
        let mut result = template.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string().trim_matches('"').to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
        
        result
    }

    fn parse_announcement_from_row(&self, _row: &Value) -> PluginResult<Announcement> {
        // Placeholder implementation
        Err(PluginError::InvalidInput("Not implemented".to_string()))
    }

    fn parse_template_from_row(&self, _row: &Value) -> PluginResult<AnnouncementTemplate> {
        // Placeholder implementation
        Err(PluginError::InvalidInput("Not implemented".to_string()))
    }

    // HTTP handlers and component rendering would continue...
    // For brevity, implementing key methods

    async fn handle_list_announcements(&self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let category = request.query_params.get("category");
        let status = request.query_params.get("status");
        let contest_id = request.query_params.get("contest_id")
            .and_then(|s| Uuid::parse_str(s).ok());
        
        let mut filtered_announcements: Vec<&Announcement> = self.announcements.values()
            .filter(|a| {
                if let Some(cat) = category {
                    if format!("{:?}", a.category).to_lowercase() != cat.to_lowercase() {
                        return false;
                    }
                }
                if let Some(stat) = status {
                    if format!("{:?}", a.status).to_lowercase() != stat.to_lowercase() {
                        return false;
                    }
                }
                if let Some(cid) = contest_id {
                    if a.contest_id != Some(cid) {
                        return false;
                    }
                }
                true
            })
            .collect();
        
        // Sort by creation time, newest first
        filtered_announcements.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        let response = serde_json::json!({
            "announcements": filtered_announcements,
            "total": filtered_announcements.len()
        });
        
        Ok(HttpResponse {
            status_code: 200,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_create_announcement(&mut self, request: &HttpRequest) -> PluginResult<HttpResponse> {
        let body: Value = serde_json::from_str(request.body.as_ref().unwrap_or(&String::new()))
            .map_err(|e| PluginError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        
        let create_request: CreateAnnouncementRequest = serde_json::from_value(body)
            .map_err(|e| PluginError::InvalidInput(format!("Invalid request format: {}", e)))?;
        
        let author_id = request.user_id.ok_or_else(|| PluginError::InvalidInput("User ID required".to_string()))?;
        
        let announcement = Announcement {
            id: Uuid::new_v4(),
            title: create_request.title,
            content: create_request.content,
            content_type: create_request.content_type,
            category: create_request.category,
            priority: create_request.priority,
            target_audience: create_request.target_audience,
            author_id,
            author_name: "User".to_string(), // Should be fetched from user data
            contest_id: create_request.contest_id,
            attachments: vec![],
            tags: create_request.tags,
            language: create_request.language,
            translations: HashMap::new(),
            status: if create_request.publish_immediately { AnnouncementStatus::Published } else { AnnouncementStatus::Draft },
            created_at: Utc::now(),
            published_at: if create_request.publish_immediately { Some(Utc::now()) } else { None },
            scheduled_at: create_request.schedule_at,
            expires_at: create_request.expires_at,
            updated_at: Utc::now(),
            read_count: 0,
            engagement_stats: EngagementStats {
                views: 0,
                reads: 0,
                clicks: 0,
                shares: 0,
                reactions: HashMap::new(),
            },
        };
        
        self.save_announcement(&announcement).await?;
        let announcement_id = announcement.id;
        self.announcements.insert(announcement_id, announcement.clone());
        
        if create_request.publish_immediately {
            self.send_announcement_notifications(&announcement).await?;
        }
        
        let response = serde_json::json!({
            "id": announcement_id,
            "message": "Announcement created successfully",
            "status": announcement.status
        });
        
        Ok(HttpResponse {
            status_code: 201,
            headers: [("Content-Type".to_string(), "application/json".to_string())].iter().cloned().collect(),
            body: response.to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn render_announcement_banner(&self, props: &serde_json::Value) -> PluginResult<String> {
        let announcement_id = props.get("announcement_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        
        let announcement = if let Some(id) = announcement_id {
            self.announcements.get(&id)
        } else {
            // Get the latest high-priority announcement
            self.announcements.values()
                .filter(|a| a.status == AnnouncementStatus::Published)
                .filter(|a| matches!(a.priority, AnnouncementPriority::High | AnnouncementPriority::Critical | AnnouncementPriority::Emergency))
                .max_by_key(|a| a.published_at.unwrap_or_default())
        };
        
        if let Some(announcement) = announcement {
            let priority_class = match announcement.priority {
                AnnouncementPriority::Emergency => "emergency",
                AnnouncementPriority::Critical => "critical",
                AnnouncementPriority::High => "high",
                AnnouncementPriority::Normal => "normal",
                AnnouncementPriority::Low => "low",
            };
            
            let icon = match announcement.category {
                AnnouncementCategory::Emergency => "🚨",
                AnnouncementCategory::Maintenance => "🔧",
                AnnouncementCategory::Contest => "🏆",
                AnnouncementCategory::Achievement => "🎉",
                AnnouncementCategory::System => "⚙️",
                _ => "📢",
            };
            
            let html = format!(r#"
                <div class="announcement-banner {}" data-announcement-id="{}">
                    <div class="banner-content">
                        <div class="banner-icon">{}</div>
                        <div class="banner-text">
                            <div class="banner-title">{}</div>
                            <div class="banner-message">{}</div>
                        </div>
                        <div class="banner-actions">
                            <button class="banner-close" onclick="dismissBanner('{}')">✕</button>
                        </div>
                    </div>
                </div>
                
                <script>
                    function dismissBanner(announcementId) {{
                        const banner = document.querySelector(`[data-announcement-id="${{announcementId}}"]`);
                        if (banner) {{
                            banner.style.animation = 'slideUp 0.3s ease-out forwards';
                            setTimeout(() => banner.remove(), 300);
                        }}
                    }}
                </script>
                
                <style>
                    .announcement-banner {{
                        position: fixed;
                        top: 0;
                        left: 0;
                        right: 0;
                        z-index: 1000;
                        padding: 15px 20px;
                        box-shadow: 0 2px 10px rgba(0,0,0,0.2);
                        animation: slideDown 0.3s ease-out;
                    }}
                    
                    .announcement-banner.emergency {{
                        background: linear-gradient(135deg, #dc3545, #c82333);
                        color: white;
                    }}
                    
                    .announcement-banner.critical {{
                        background: linear-gradient(135deg, #fd7e14, #e55d00);
                        color: white;
                    }}
                    
                    .announcement-banner.high {{
                        background: linear-gradient(135deg, #ffc107, #e0a800);
                        color: #212529;
                    }}
                    
                    .announcement-banner.normal {{
                        background: linear-gradient(135deg, #007bff, #0056b3);
                        color: white;
                    }}
                    
                    .announcement-banner.low {{
                        background: linear-gradient(135deg, #6c757d, #545b62);
                        color: white;
                    }}
                    
                    .banner-content {{
                        display: flex;
                        align-items: center;
                        max-width: 1200px;
                        margin: 0 auto;
                    }}
                    
                    .banner-icon {{
                        font-size: 1.5em;
                        margin-right: 15px;
                    }}
                    
                    .banner-text {{
                        flex: 1;
                    }}
                    
                    .banner-title {{
                        font-weight: bold;
                        font-size: 1.1em;
                        margin-bottom: 5px;
                    }}
                    
                    .banner-message {{
                        font-size: 0.95em;
                        opacity: 0.9;
                    }}
                    
                    .banner-actions {{
                        margin-left: 15px;
                    }}
                    
                    .banner-close {{
                        background: none;
                        border: none;
                        color: inherit;
                        font-size: 1.2em;
                        cursor: pointer;
                        padding: 5px 8px;
                        border-radius: 50%;
                        transition: background-color 0.2s;
                    }}
                    
                    .banner-close:hover {{
                        background-color: rgba(255,255,255,0.2);
                    }}
                    
                    @keyframes slideDown {{
                        from {{
                            transform: translateY(-100%);
                            opacity: 0;
                        }}
                        to {{
                            transform: translateY(0);
                            opacity: 1;
                        }}
                    }}
                    
                    @keyframes slideUp {{
                        from {{
                            transform: translateY(0);
                            opacity: 1;
                        }}
                        to {{
                            transform: translateY(-100%);
                            opacity: 0;
                        }}
                    }}
                </style>
            "#, 
            priority_class,
            announcement.id,
            icon,
            announcement.title,
            self.truncate_content(&announcement.content, 150),
            announcement.id);
            
            Ok(html)
        } else {
            Ok(String::new()) // No announcement to show
        }
    }

    // Placeholder implementations for remaining handlers
    async fn handle_get_announcement(&self, _request: &HttpRequest, _announcement_id: &str) -> PluginResult<HttpResponse> {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "{}".to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_list_templates(&self, _request: &HttpRequest) -> PluginResult<HttpResponse> {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "{}".to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_get_statistics(&self, _request: &HttpRequest) -> PluginResult<HttpResponse> {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "{}".to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_publish_announcement(&mut self, _request: &HttpRequest) -> PluginResult<HttpResponse> {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "{}".to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_schedule_announcement(&mut self, _request: &HttpRequest) -> PluginResult<HttpResponse> {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "{}".to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_mark_read(&mut self, _request: &HttpRequest) -> PluginResult<HttpResponse> {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "{}".to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_create_emergency(&mut self, _request: &HttpRequest) -> PluginResult<HttpResponse> {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "{}".to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_create_maintenance(&mut self, _request: &HttpRequest) -> PluginResult<HttpResponse> {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "{}".to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_update_announcement(&mut self, _request: &HttpRequest, _announcement_id: &str) -> PluginResult<HttpResponse> {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "{}".to_string(),
            content_type: "application/json".to_string(),
        })
    }

    async fn handle_delete_announcement(&mut self, _request: &HttpRequest, _announcement_id: &str) -> PluginResult<HttpResponse> {
        Ok(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "{}".to_string(),
            content_type: "application/json".to_string(),
        })
    }

    // Additional rendering methods
    async fn render_announcement_list(&self, _props: &serde_json::Value) -> PluginResult<String> {
        Ok("Announcement list component".to_string())
    }

    async fn render_announcement_composer(&self, _props: &serde_json::Value) -> PluginResult<String> {
        Ok("Announcement composer component".to_string())
    }

    async fn render_announcement_viewer(&self, _props: &serde_json::Value) -> PluginResult<String> {
        Ok("Announcement viewer component".to_string())
    }

    async fn render_announcement_dashboard(&self, _props: &serde_json::Value) -> PluginResult<String> {
        Ok("Announcement dashboard component".to_string())
    }

    async fn render_emergency_alert(&self, _props: &serde_json::Value) -> PluginResult<String> {
        Ok("Emergency alert component".to_string())
    }

    async fn render_maintenance_notice(&self, _props: &serde_json::Value) -> PluginResult<String> {
        Ok("Maintenance notice component".to_string())
    }

    async fn render_announcement_settings(&self, _props: &serde_json::Value) -> PluginResult<String> {
        Ok("Announcement settings component".to_string())
    }
}