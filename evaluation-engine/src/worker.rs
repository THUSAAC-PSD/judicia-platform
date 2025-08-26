use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

use crate::job_queue::{JobQueue, EvaluationJob, EvaluationResult, TestResult, WorkerHeartbeat};
use crate::executor::Executor;
use event_bus::{EventBus, Event};
// use jpp_parser::{JPPParser, JPProblem, JPPIntegration}; // TODO: Use when implementing actual evaluation

/// Distributed evaluation worker that processes evaluation jobs
pub struct EvaluationWorker {
    worker_id: String,
    node_id: String,
    job_queue: Arc<dyn JobQueue + Send + Sync>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
    executor: Arc<Executor>,
    max_capacity: i32,
    current_load: Arc<std::sync::atomic::AtomicI32>,
    capabilities: Vec<String>,
}

impl EvaluationWorker {
    pub fn new(
        worker_id: String,
        node_id: String,
        job_queue: Arc<dyn JobQueue + Send + Sync>,
        event_bus: Arc<dyn EventBus + Send + Sync>,
        executor: Arc<Executor>,
        max_capacity: i32,
    ) -> Self {
        Self {
            worker_id,
            node_id,
            job_queue,
            event_bus,
            executor,
            max_capacity,
            current_load: Arc::new(std::sync::atomic::AtomicI32::new(0)),
            capabilities: vec![
                "judicia/standard@1.0".to_string(),
                "judicia/special-judge@1.0".to_string(),
                "judicia/interactive@1.0".to_string(),
            ],
        }
    }
    
    /// Start the worker processing loop
    pub async fn start(&self) -> Result<()> {
        tracing::info!("🚀 Starting evaluation worker: {} on node: {}", self.worker_id, self.node_id);
        
        // Start heartbeat task
        let worker_id = self.worker_id.clone();
        let node_id = self.node_id.clone();
        let job_queue = self.job_queue.clone();
        let current_load = self.current_load.clone();
        let max_capacity = self.max_capacity;
        let capabilities = self.capabilities.clone();
        
        tokio::spawn(async move {
            Self::heartbeat_loop(worker_id, node_id, job_queue, current_load, max_capacity, capabilities).await;
        });
        
        // Main job processing loop
        self.job_processing_loop().await
    }
    
    /// Main job processing loop
    async fn job_processing_loop(&self) -> Result<()> {
        loop {
            // Check if we can accept more jobs
            let current_load = self.current_load.load(std::sync::atomic::Ordering::Acquire);
            if current_load >= self.max_capacity {
                tracing::debug!("Worker at capacity ({}/{}), waiting...", current_load, self.max_capacity);
                time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            
            // Try to claim a job
            match self.job_queue.claim_job(&self.worker_id, &self.capabilities).await {
                Ok(Some(job)) => {
                    // Increment load counter
                    self.current_load.fetch_add(1, std::sync::atomic::Ordering::Release);
                    
                    tracing::info!("🎯 Claimed job: {} for submission: {}", job.id, job.submission_id);
                    
                    // Process job in separate task
                    let worker_clone = self.clone_for_task();
                    tokio::spawn(async move {
                        worker_clone.process_job(job).await;
                    });
                }
                Ok(None) => {
                    // No jobs available, wait a bit
                    time::sleep(Duration::from_millis(500)).await;
                }
                Err(e) => {
                    tracing::error!("Failed to claim job: {}", e);
                    time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
    
    /// Process a single evaluation job
    async fn process_job(&self, job: EvaluationJob) {
        let job_id = job.id;
        let submission_id = job.submission_id;
        
        tracing::info!("📝 Processing evaluation job: {} (submission: {})", job_id, submission_id);
        
        // Emit job started event
        if let Err(e) = self.emit_job_event("evaluation.started", &job, None).await {
            tracing::warn!("Failed to emit job started event: {}", e);
        }
        
        let result = self.execute_evaluation(&job).await;
        
        match result {
            Ok(evaluation_result) => {
                // Submit result to queue
                if let Err(e) = self.job_queue.complete_job(evaluation_result.clone()).await {
                    tracing::error!("Failed to submit evaluation result: {}", e);
                }
                
                // Emit job completed event
                if let Err(e) = self.emit_job_event("evaluation.completed", &job, Some(&evaluation_result)).await {
                    tracing::warn!("Failed to emit job completed event: {}", e);
                }
                
                tracing::info!("✅ Completed evaluation job: {} -> {}", job_id, evaluation_result.verdict);
            }
            Err(e) => {
                tracing::error!("❌ Evaluation job failed: {} - {}", job_id, e);
                
                // Mark job as failed
                if let Err(fail_err) = self.job_queue.fail_job(job_id, &e.to_string(), true).await {
                    tracing::error!("Failed to mark job as failed: {}", fail_err);
                }
                
                // Emit job failed event
                if let Err(e) = self.emit_job_event("evaluation.failed", &job, None).await {
                    tracing::warn!("Failed to emit job failed event: {}", e);
                }
            }
        }
        
        // Decrement load counter
        self.current_load.fetch_sub(1, std::sync::atomic::Ordering::Release);
    }
    
    /// Execute the actual evaluation based on job parameters
    async fn execute_evaluation(&self, job: &EvaluationJob) -> Result<EvaluationResult> {
        // For now, create a simplified evaluation result
        // In full implementation, this would:
        // 1. Parse the JPP problem format from job metadata
        // 2. Set up the appropriate sandbox environment
        // 3. Compile and execute the code based on problem type
        // 4. Run test cases and collect results
        
        // Simulate processing time
        time::sleep(Duration::from_millis(100)).await;
        
        // Create mock test results
        let test_results = vec![
            TestResult {
                test_case_id: Some(Uuid::new_v4()),
                test_number: 1,
                verdict: "AC".to_string(),
                execution_time_ms: Some(45),
                execution_memory_kb: Some(1024),
                stdout: Some("Hello World".to_string()),
                stderr: None,
                exit_code: Some(0),
            }
        ];
        
        let result = EvaluationResult {
            job_id: job.id,
            submission_id: job.submission_id,
            verdict: "AC".to_string(),
            execution_time_ms: Some(45),
            execution_memory_kb: Some(1024),
            score: Some(100),
            test_results,
            compile_output: Some("Compilation successful".to_string()),
            completed_at: chrono::Utc::now(),
            worker_node_id: self.node_id.clone(),
        };
        
        Ok(result)
    }
    
    /// Emit job-related events
    async fn emit_job_event(
        &self, 
        event_type: &str, 
        job: &EvaluationJob, 
        result: Option<&EvaluationResult>
    ) -> Result<()> {
        let mut payload = serde_json::json!({
            "job_id": job.id,
            "submission_id": job.submission_id,
            "problem_id": job.problem_id,
            "worker_id": self.worker_id,
            "node_id": self.node_id
        });
        
        if let Some(result) = result {
            payload["verdict"] = serde_json::json!(result.verdict);
            payload["execution_time_ms"] = serde_json::json!(result.execution_time_ms);
            payload["execution_memory_kb"] = serde_json::json!(result.execution_memory_kb);
            payload["score"] = serde_json::json!(result.score);
        }
        
        let event = Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            source_plugin_id: None,
            timestamp: chrono::Utc::now(),
            payload,
        };
        
        self.event_bus.publish(event).await
    }
    
    /// Heartbeat loop to report worker status
    async fn heartbeat_loop(
        worker_id: String,
        node_id: String,
        job_queue: Arc<dyn JobQueue + Send + Sync>,
        current_load: Arc<std::sync::atomic::AtomicI32>,
        max_capacity: i32,
        capabilities: Vec<String>,
    ) {
        let mut interval = time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            let load = current_load.load(std::sync::atomic::Ordering::Acquire);
            let status = if load >= max_capacity {
                "busy"
            } else if load > 0 {
                "online"
            } else {
                "online"
            };
            
            let heartbeat = WorkerHeartbeat {
                worker_id: worker_id.clone(),
                node_id: node_id.clone(),
                status: status.to_string(),
                current_load: load,
                max_capacity,
                capabilities: capabilities.clone(),
                last_heartbeat: chrono::Utc::now(),
                system_info: serde_json::json!({
                    "hostname": hostname::get().unwrap_or_default().to_string_lossy(),
                    "pid": std::process::id(),
                }),
            };
            
            if let Err(e) = job_queue.worker_heartbeat(heartbeat).await {
                tracing::error!("Failed to send heartbeat: {}", e);
            }
        }
    }
    
    /// Create a clone suitable for task spawning
    fn clone_for_task(&self) -> WorkerTaskHandle {
        WorkerTaskHandle {
            worker_id: self.worker_id.clone(),
            node_id: self.node_id.clone(),
            job_queue: self.job_queue.clone(),
            event_bus: self.event_bus.clone(),
            executor: self.executor.clone(),
            current_load: self.current_load.clone(),
        }
    }
}

/// Handle for worker tasks
struct WorkerTaskHandle {
    worker_id: String,
    node_id: String,
    job_queue: Arc<dyn JobQueue + Send + Sync>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
    executor: Arc<Executor>,
    current_load: Arc<std::sync::atomic::AtomicI32>,
}

impl WorkerTaskHandle {
    async fn process_job(&self, job: EvaluationJob) {
        let worker = EvaluationWorker {
            worker_id: self.worker_id.clone(),
            node_id: self.node_id.clone(),
            job_queue: self.job_queue.clone(),
            event_bus: self.event_bus.clone(),
            executor: self.executor.clone(),
            max_capacity: 1, // Not used in task context
            current_load: self.current_load.clone(),
            capabilities: vec![], // Not used in task context
        };
        
        worker.process_job(job).await;
    }
}

/// Utility to get system hostname
mod hostname {
    use std::ffi::OsString;
    
    pub fn get() -> Option<OsString> {
        #[cfg(windows)]
        {
            use std::env;
            env::var_os("COMPUTERNAME")
        }
        
        #[cfg(unix)]
        {
            use std::ffi::CStr;
            unsafe {
                let mut buf = [0u8; 256];
                if libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) == 0 {
                    let hostname = CStr::from_ptr(buf.as_ptr() as *const libc::c_char);
                    Some(hostname.to_string_lossy().into_owned().into())
                } else {
                    None
                }
            }
        }
        
        #[cfg(not(any(windows, unix)))]
        {
            None
        }
    }
}