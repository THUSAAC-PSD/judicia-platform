use crate::Event;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Standard event types defined by the platform
/// Plugins can also define custom events

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionJudgedEvent {
    pub submission_id: Uuid,
    pub user_id: Uuid,
    pub problem_id: Uuid,
    pub verdict: String,
    pub execution_time_ms: Option<i32>,
    pub execution_memory_kb: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContestProblemFirstBloodEvent {
    pub contest_id: Uuid,
    pub problem_id: Uuid,
    pub user_id: Uuid,
    pub submission_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContestStartedEvent {
    pub contest_id: Uuid,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContestEndedEvent {
    pub contest_id: Uuid,
    pub end_time: chrono::DateTime<chrono::Utc>,
}

impl SubmissionJudgedEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "submission.judged".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

impl ContestProblemFirstBloodEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "contest.problem.first_blood".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

impl UserRegisteredEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "user.registered".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

impl ContestStartedEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "contest.started".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

impl ContestEndedEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "contest.ended".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

/// Additional event types for comprehensive platform coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionSubmittedEvent {
    pub submission_id: Uuid,
    pub user_id: Uuid,
    pub problem_id: Uuid,
    pub language_id: Uuid,
    pub contest_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemCreatedEvent {
    pub problem_id: Uuid,
    pub title: String,
    pub difficulty: Option<String>,
    pub creator_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLoadedEvent {
    pub plugin_id: Uuid,
    pub plugin_name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUnloadedEvent {
    pub plugin_id: Uuid,
    pub plugin_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncementCreatedEvent {
    pub announcement_id: Uuid,
    pub contest_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationRequestedEvent {
    pub clarification_id: Uuid,
    pub contest_id: Uuid,
    pub problem_id: Uuid,
    pub user_id: Uuid,
    pub question: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationAnsweredEvent {
    pub clarification_id: Uuid,
    pub contest_id: Uuid,
    pub problem_id: Uuid,
    pub answer: String,
    pub is_public: bool,
}

impl SubmissionSubmittedEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "submission.submitted".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

impl ProblemCreatedEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "problem.created".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

impl PluginLoadedEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "plugin.loaded".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

impl PluginUnloadedEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "plugin.unloaded".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

impl AnnouncementCreatedEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "announcement.created".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

impl ClarificationRequestedEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "clarification.requested".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

impl ClarificationAnsweredEvent {
    pub fn to_event(&self, source_plugin_id: Option<Uuid>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: "clarification.answered".to_string(),
            source_plugin_id,
            timestamp: chrono::Utc::now(),
            payload: serde_json::to_value(self).unwrap(),
        }
    }
}

/// Event type constants for easy reference
pub mod event_types {
    // Submission events
    pub const SUBMISSION_SUBMITTED: &str = "submission.submitted";
    pub const SUBMISSION_JUDGED: &str = "submission.judged";
    pub const SUBMISSION_QUEUED: &str = "submission.queued";
    
    // Contest events
    pub const CONTEST_CREATED: &str = "contest.created";
    pub const CONTEST_STARTED: &str = "contest.started";
    pub const CONTEST_ENDED: &str = "contest.ended";
    pub const CONTEST_PROBLEM_FIRST_BLOOD: &str = "contest.problem.first_blood";
    
    // User events
    pub const USER_REGISTERED: &str = "user.registered";
    pub const USER_LOGIN: &str = "user.login";
    pub const USER_LOGOUT: &str = "user.logout";
    
    // Problem events
    pub const PROBLEM_CREATED: &str = "problem.created";
    pub const PROBLEM_UPDATED: &str = "problem.updated";
    pub const PROBLEM_DELETED: &str = "problem.deleted";
    
    // Plugin system events
    pub const PLUGIN_LOADED: &str = "plugin.loaded";
    pub const PLUGIN_UNLOADED: &str = "plugin.unloaded";
    pub const PLUGIN_ERROR: &str = "plugin.error";
    
    // Communication events
    pub const ANNOUNCEMENT_CREATED: &str = "announcement.created";
    pub const CLARIFICATION_REQUESTED: &str = "clarification.requested";
    pub const CLARIFICATION_ANSWERED: &str = "clarification.answered";
    
    // System events
    pub const SYSTEM_MAINTENANCE_START: &str = "system.maintenance.start";
    pub const SYSTEM_MAINTENANCE_END: &str = "system.maintenance.end";
    pub const SYSTEM_BACKUP_COMPLETE: &str = "system.backup.complete";
}