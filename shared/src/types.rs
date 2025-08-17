use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JudgeStatus {
    Queued,
    Compiling,
    Running,
    Finished,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Verdict {
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
    CompilationError,
    PresentationError,
    SystemError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionType {
    IoiStandard,
    OutputOnly,
    Interactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgingJob {
    pub submission_id: Uuid,
    pub user_id: Uuid,
    pub problem_id: Uuid,
    pub language_id: Uuid,
    pub source_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub test_case_id: Uuid,
    pub verdict: Verdict,
    pub execution_time_ms: Option<i32>,
    pub execution_memory_kb: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum WebSocketMessage {
    #[serde(rename = "status_update")]
    StatusUpdate { status: JudgeStatus },
    #[serde(rename = "test_case_finished")]
    TestCaseFinished {
        test_case: i32,
        verdict: Verdict,
    },
    #[serde(rename = "final_result")]
    FinalResult {
        verdict: Verdict,
        failed_case: Option<i32>,
        execution_time_ms: Option<i32>,
        execution_memory_kb: Option<i32>,
    },
}