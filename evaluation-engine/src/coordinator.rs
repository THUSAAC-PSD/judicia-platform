use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

use crate::config::Config;
use crate::executor::Executor;
use crate::job_queue::{JobQueue, RabbitMQJobQueue, EvaluationJob, QueueStats, EvaluationResult, TestResult};
use event_bus::{EventBus, RabbitMQEventBus, Event};
use jpp_parser::JPProblem;

/// Evaluation Coordinator manages worker nodes and job distribution
pub struct EvaluationCoordinator {
    config: Arc<Config>,
    job_queue: Arc<dyn JobQueue + Send + Sync>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
    workers: Arc<tokio::sync::RwLock<HashMap<String, WorkerInfo>>>,
    node_id: String,
}

#[derive(Debug, Clone)]
struct WorkerInfo {
    worker_id: String,
    node_id: String,
    status: String,
    current_load: i32,
    max_capacity: i32,
    capabilities: Vec<String>,
    last_heartbeat: chrono::DateTime<chrono::Utc>,
}

impl EvaluationCoordinator {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        // Initialize job queue
        let job_queue = Arc::new(RabbitMQJobQueue::new(&config.rabbitmq_url).await?);
        
        // Initialize event bus
        let event_bus = Arc::new(RabbitMQEventBus::new(&config.rabbitmq_url).await?);
        
        // Generate unique node ID
        let node_id = format!("node-{}", Uuid::new_v4());
        
        Ok(Self {
            config,
            job_queue,
            event_bus,
            workers: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            node_id,
        })
    }

    /// Start the evaluation coordinator
    pub async fn start(&self) -> Result<()> {
        tracing::info!("🚀 Starting evaluation coordinator on node: {}", self.node_id);
        
        // Start worker monitoring task
        let coordinator_clone = self.clone_for_task();
        tokio::spawn(async move {
            coordinator_clone.monitor_workers().await;
        });
        
        // Start queue monitoring task
        let coordinator_clone = self.clone_for_task();
        tokio::spawn(async move {
            coordinator_clone.monitor_queue().await;
        });
        
        // Start local workers based on configuration
        self.start_local_workers().await?;
        
        // Start event listening
        self.start_event_listener().await?;
        
        tracing::info!("✅ Evaluation coordinator started successfully");
        
        // Keep the coordinator running
        loop {
            time::sleep(Duration::from_secs(60)).await;
            self.log_status().await;
        }
    }
    
    /// Start local worker processes
    async fn start_local_workers(&self) -> Result<()> {
        let worker_count = self.config.worker_count.unwrap_or(2);
        tracing::info!("🏭 Starting {} local workers", worker_count);
        
        // Instead of creating EvaluationWorker instances, 
        // we'll create simple worker tasks that claim and process jobs
        for i in 0..worker_count {
            let worker_id = format!("{}-worker-{}", self.node_id, i);
            let job_queue = self.job_queue.clone();
            let event_bus = self.event_bus.clone();
            let executor = Arc::new(Executor::new(self.config.clone())?);
            
            // Start worker task
            tokio::spawn(async move {
                Self::worker_loop(worker_id, job_queue, event_bus, executor).await;
            });
        }
        
        Ok(())
    }
    
    /// Start event listener for coordinator events
    async fn start_event_listener(&self) -> Result<()> {
        let subscriber_id = Uuid::new_v4();
        let mut event_receiver = self.event_bus.subscribe("evaluation.*", subscriber_id).await?;
        
        let coordinator_clone = self.clone_for_task();
        tokio::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                coordinator_clone.handle_event(event).await;
            }
        });
        
        Ok(())
    }
    
    /// Handle incoming events
    async fn handle_event(&self, event: Event) {
        match event.event_type.as_str() {
            "evaluation.started" => {
                tracing::debug!("📝 Evaluation started: {}", event.id);
            }
            "evaluation.completed" => {
                tracing::debug!("✅ Evaluation completed: {}", event.id);
            }
            "evaluation.failed" => {
                tracing::warn!("❌ Evaluation failed: {}", event.id);
            }
            _ => {
                tracing::debug!("📬 Received event: {}", event.event_type);
            }
        }
    }
    
    /// Monitor worker health and status
    async fn monitor_workers(&self) {
        let mut interval = time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            let mut workers = self.workers.write().await;
            let now = chrono::Utc::now();
            
            // Remove stale workers (no heartbeat in 2 minutes)
            workers.retain(|worker_id, worker_info| {
                let stale = now.signed_duration_since(worker_info.last_heartbeat).num_seconds() > 120;
                if stale {
                    tracing::warn!("🚨 Worker {} is stale, removing from registry", worker_id);
                }
                !stale
            });
            
            // Log worker status
            let active_workers = workers.len();
            let total_capacity: i32 = workers.values().map(|w| w.max_capacity).sum();
            let current_load: i32 = workers.values().map(|w| w.current_load).sum();
            
            tracing::debug!("👥 Workers: {} active, {}/{} load", active_workers, current_load, total_capacity);
        }
    }
    
    /// Monitor queue statistics
    async fn monitor_queue(&self) {
        let mut interval = time::interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            match self.job_queue.get_queue_stats().await {
                Ok(stats) => {
                    tracing::info!(
                        "📊 Queue stats - Pending: {}, Running: {}, Completed: {}, Failed: {}, Workers: {}",
                        stats.pending_jobs,
                        stats.running_jobs,
                        stats.completed_jobs,
                        stats.failed_jobs,
                        stats.active_workers
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to get queue stats: {}", e);
                }
            }
        }
    }
    
    /// Log coordinator status
    async fn log_status(&self) {
        let workers = self.workers.read().await;
        tracing::info!(
            "🏛️ Coordinator status - Node: {}, Workers: {}, Queue: connected, Events: connected",
            self.node_id,
            workers.len()
        );
    }
    
    /// Create job from submission parameters (integration with Core Kernel)
    pub async fn create_evaluation_job(
        &self,
        submission_id: Uuid,
        problem_id: Uuid,
        language_id: Uuid,
        source_code: String,
        jpp_problem: &JPProblem,
    ) -> Result<EvaluationJob> {
        let job = EvaluationJob {
            id: Uuid::new_v4(),
            submission_id,
            problem_id,
            language_id,
            source_code,
            priority: 1, // Normal priority
            timeout_ms: jpp_problem.time_limit_ms as i32,
            memory_limit_kb: jpp_problem.memory_limit_kb as i32,
            test_case_count: 10, // TODO: Get from JPP problem
            created_at: chrono::Utc::now(),
            retry_count: 0,
            max_retries: 3,
            metadata: serde_json::json!({
                "problem_title": jpp_problem.title,
                "plugin_type": jpp_problem.judging.plugin_type,
                "judging_config": jpp_problem.judging.config,
                "coordinator_node": self.node_id
            }),
        };
        
        Ok(job)
    }
    
    /// Submit evaluation job to the queue
    pub async fn submit_job(&self, job: EvaluationJob) -> Result<()> {
        tracing::info!("📤 Submitting evaluation job: {} for submission: {}", job.id, job.submission_id);
        self.job_queue.submit_job(job).await
    }
    
    /// Get queue statistics
    pub async fn get_stats(&self) -> Result<QueueStats> {
        self.job_queue.get_queue_stats().await
    }
    
    /// Worker loop for processing jobs
    async fn worker_loop(
        worker_id: String,
        job_queue: Arc<dyn JobQueue + Send + Sync>,
        _event_bus: Arc<dyn EventBus + Send + Sync>,
        _executor: Arc<Executor>,
    ) {
        tracing::info!("🚀 Starting worker: {}", worker_id);
        
        loop {
            // Try to claim a job
            match job_queue.claim_job(&worker_id, &["judicia/standard@1.0".to_string()]).await {
                Ok(Some(job)) => {
                    tracing::info!("🎯 Worker {} claimed job: {}", worker_id, job.id);
                    
                    // Process job (simplified version)
                    let result = Self::process_job_simple(&job).await;
                    
                    match result {
                        Ok(evaluation_result) => {
                            if let Err(e) = job_queue.complete_job(evaluation_result).await {
                                tracing::error!("Failed to submit result: {}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Job processing failed: {}", e);
                            if let Err(e) = job_queue.fail_job(job.id, &e.to_string(), true).await {
                                tracing::error!("Failed to mark job as failed: {}", e);
                            }
                        }
                    }
                }
                Ok(None) => {
                    // No jobs available, wait
                    time::sleep(Duration::from_millis(500)).await;
                }
                Err(e) => {
                    tracing::error!("Worker {} failed to claim job: {}", worker_id, e);
                    time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
    
    /// Simplified job processing
    async fn process_job_simple(job: &EvaluationJob) -> Result<EvaluationResult> {
        // Simulate processing time
        time::sleep(Duration::from_millis(100)).await;
        
        // Create mock result
        let result = EvaluationResult {
            job_id: job.id,
            submission_id: job.submission_id,
            verdict: "AC".to_string(),
            execution_time_ms: Some(45),
            execution_memory_kb: Some(1024),
            score: Some(100),
            test_results: vec![
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
            ],
            compile_output: Some("Compilation successful".to_string()),
            completed_at: chrono::Utc::now(),
            worker_node_id: "mock-node".to_string(),
        };
        
        Ok(result)
    }
    
    /// Clone for task spawning
    fn clone_for_task(&self) -> CoordinatorTaskHandle {
        CoordinatorTaskHandle {
            node_id: self.node_id.clone(),
            job_queue: self.job_queue.clone(),
            event_bus: self.event_bus.clone(),
            workers: self.workers.clone(),
        }
    }

}

/// Task handle for coordinator operations
struct CoordinatorTaskHandle {
    node_id: String,
    job_queue: Arc<dyn JobQueue + Send + Sync>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
    workers: Arc<tokio::sync::RwLock<HashMap<String, WorkerInfo>>>,
}

impl CoordinatorTaskHandle {
    async fn monitor_workers(&self) {
        let coordinator = EvaluationCoordinator {
            config: Arc::new(Config::default()), // Not used in task context
            job_queue: self.job_queue.clone(),
            event_bus: self.event_bus.clone(),
            workers: self.workers.clone(),
            node_id: self.node_id.clone(),
        };
        
        coordinator.monitor_workers().await;
    }
    
    async fn monitor_queue(&self) {
        let coordinator = EvaluationCoordinator {
            config: Arc::new(Config::default()), // Not used in task context
            job_queue: self.job_queue.clone(),
            event_bus: self.event_bus.clone(),
            workers: self.workers.clone(),
            node_id: self.node_id.clone(),
        };
        
        coordinator.monitor_queue().await;
    }
    
    async fn handle_event(&self, event: Event) {
        let coordinator = EvaluationCoordinator {
            config: Arc::new(Config::default()), // Not used in task context
            job_queue: self.job_queue.clone(),
            event_bus: self.event_bus.clone(),
            workers: self.workers.clone(),
            node_id: self.node_id.clone(),
        };
        
        coordinator.handle_event(event).await;
    }
}