use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties,
    Connection, ConnectionProperties,
};

/// Evaluation job queue for distributing submission evaluation tasks
/// Integrates with RabbitMQ for reliable message delivery

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationJob {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub problem_id: Uuid,
    pub language_id: Uuid,
    pub source_code: String,
    pub priority: i32,
    pub timeout_ms: i32,
    pub memory_limit_kb: i32,
    pub test_case_count: i32,
    pub created_at: DateTime<Utc>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub job_id: Uuid,
    pub submission_id: Uuid,
    pub verdict: String, // "AC", "WA", "TLE", "MLE", "RE", "CE", etc.
    pub execution_time_ms: Option<i32>,
    pub execution_memory_kb: Option<i32>,
    pub score: Option<i32>,
    pub test_results: Vec<TestResult>,
    pub compile_output: Option<String>,
    pub completed_at: DateTime<Utc>,
    pub worker_node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_case_id: Option<Uuid>,
    pub test_number: i32,
    pub verdict: String,
    pub execution_time_ms: Option<i32>,
    pub execution_memory_kb: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHeartbeat {
    pub worker_id: String,
    pub node_id: String,
    pub status: String, // "online", "busy", "maintenance"
    pub current_load: i32,
    pub max_capacity: i32,
    pub capabilities: Vec<String>,
    pub last_heartbeat: DateTime<Utc>,
    pub system_info: serde_json::Value,
}

#[async_trait]
pub trait JobQueue {
    /// Submit a new evaluation job to the queue
    async fn submit_job(&self, job: EvaluationJob) -> Result<()>;
    
    /// Claim the next job from the queue (worker pulls)
    async fn claim_job(&self, worker_id: &str, capabilities: &[String]) -> Result<Option<EvaluationJob>>;
    
    /// Mark a job as completed with results
    async fn complete_job(&self, result: EvaluationResult) -> Result<()>;
    
    /// Mark a job as failed and potentially retry
    async fn fail_job(&self, job_id: Uuid, error: &str, retry: bool) -> Result<()>;
    
    /// Send worker heartbeat
    async fn worker_heartbeat(&self, heartbeat: WorkerHeartbeat) -> Result<()>;
    
    /// Get queue statistics
    async fn get_queue_stats(&self) -> Result<QueueStats>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub pending_jobs: i64,
    pub running_jobs: i64,
    pub completed_jobs: i64,
    pub failed_jobs: i64,
    pub active_workers: i64,
    pub average_wait_time_ms: Option<i64>,
    pub average_execution_time_ms: Option<i64>,
}

pub struct RabbitMQJobQueue {
    connection: Arc<Connection>,
    job_queue_name: String,
    result_queue_name: String,
    heartbeat_queue_name: String,
}

impl RabbitMQJobQueue {
    pub async fn new(rabbitmq_url: &str) -> Result<Self> {
        let connection = Arc::new(
            Connection::connect(rabbitmq_url, ConnectionProperties::default()).await?
        );
        
        let job_queue_name = "judicia_evaluation_jobs".to_string();
        let result_queue_name = "judicia_evaluation_results".to_string();
        let heartbeat_queue_name = "judicia_worker_heartbeats".to_string();
        
        // Set up queues and exchanges
        Self::setup_infrastructure(&connection, &job_queue_name, &result_queue_name, &heartbeat_queue_name).await?;
        
        Ok(Self {
            connection,
            job_queue_name,
            result_queue_name,
            heartbeat_queue_name,
        })
    }
    
    async fn setup_infrastructure(
        connection: &Connection,
        job_queue: &str,
        result_queue: &str,
        heartbeat_queue: &str,
    ) -> Result<()> {
        let channel = connection.create_channel().await?;
        
        // Create exchange for job distribution
        channel
            .exchange_declare(
                "judicia_jobs",
                lapin::ExchangeKind::Direct,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;
            
        // Create job queue with priority support
        let mut job_queue_args = FieldTable::default();
        job_queue_args.insert("x-max-priority".into(), 10.into());
        
        channel
            .queue_declare(
                job_queue,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                job_queue_args,
            )
            .await?;
            
        // Bind job queue to exchange
        channel
            .queue_bind(
                job_queue,
                "judicia_jobs",
                "evaluation",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
            
        // Create result queue
        channel
            .queue_declare(
                result_queue,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;
            
        // Create heartbeat queue (short TTL)
        let mut heartbeat_args = FieldTable::default();
        heartbeat_args.insert("x-message-ttl".into(), 300000.into()); // 5 minute TTL
        
        channel
            .queue_declare(
                heartbeat_queue,
                QueueDeclareOptions {
                    durable: false,
                    ..Default::default()
                },
                heartbeat_args,
            )
            .await?;
        
        tracing::info!("✅ RabbitMQ job queue infrastructure set up");
        Ok(())
    }
}

#[async_trait]
impl JobQueue for RabbitMQJobQueue {
    async fn submit_job(&self, job: EvaluationJob) -> Result<()> {
        let channel = self.connection.create_channel().await?;
        let payload = serde_json::to_vec(&job)?;
        
        let properties = BasicProperties::default()
            .with_priority(job.priority as u8)
            .with_message_id(job.id.to_string().into())
            .with_timestamp(job.created_at.timestamp() as u64)
            .with_content_type("application/json".into());
        
        let confirm = channel
            .basic_publish(
                "judicia_jobs",
                "evaluation",
                BasicPublishOptions::default(),
                &payload,
                properties,
            )
            .await?
            .await?;
            
        match confirm {
            Confirmation::Ack(_) => {
                tracing::info!("✅ Evaluation job submitted: {} (submission: {})", job.id, job.submission_id);
                Ok(())
            }
            Confirmation::Nack(_) => Err(anyhow::anyhow!("Failed to submit evaluation job")),
            Confirmation::NotRequested => Ok(()),
        }
    }
    
    async fn claim_job(&self, worker_id: &str, _capabilities: &[String]) -> Result<Option<EvaluationJob>> {
        let channel = self.connection.create_channel().await?;
        
        // Set QoS to ensure fair distribution
        channel.basic_qos(1, BasicQosOptions::default()).await?;
        
        // Try to get a message from the queue
        let delivery = channel
            .basic_get(
                &self.job_queue_name,
                BasicGetOptions { no_ack: false }
            )
            .await?;
            
        if let Some(delivery) = delivery {
            let job: EvaluationJob = serde_json::from_slice(&delivery.data)?;
            
            // Check if worker has required capabilities
            // For now, we'll assume all workers can handle all jobs
            // In full implementation, match job requirements with worker capabilities
            
            tracing::info!("🎯 Job claimed by worker {}: {} (submission: {})", 
                          worker_id, job.id, job.submission_id);
            
            // Acknowledge the message
            delivery.ack(BasicAckOptions::default()).await?;
            
            Ok(Some(job))
        } else {
            Ok(None)
        }
    }
    
    async fn complete_job(&self, result: EvaluationResult) -> Result<()> {
        let channel = self.connection.create_channel().await?;
        let payload = serde_json::to_vec(&result)?;
        
        let properties = BasicProperties::default()
            .with_message_id(result.job_id.to_string().into())
            .with_timestamp(result.completed_at.timestamp() as u64)
            .with_content_type("application/json".into());
        
        channel
            .basic_publish(
                "",
                &self.result_queue_name,
                BasicPublishOptions::default(),
                &payload,
                properties,
            )
            .await?;
            
        tracing::info!("✅ Evaluation result submitted: {} -> {}", 
                      result.submission_id, result.verdict);
        Ok(())
    }
    
    async fn fail_job(&self, job_id: Uuid, error: &str, retry: bool) -> Result<()> {
        tracing::error!("❌ Job failed: {} - {}", job_id, error);
        
        if retry {
            tracing::info!("🔄 Job will be retried: {}", job_id);
            // In full implementation, requeue the job with incremented retry count
        }
        
        Ok(())
    }
    
    async fn worker_heartbeat(&self, heartbeat: WorkerHeartbeat) -> Result<()> {
        let channel = self.connection.create_channel().await?;
        let payload = serde_json::to_vec(&heartbeat)?;
        
        let properties = BasicProperties::default()
            .with_message_id(heartbeat.worker_id.clone().into())
            .with_timestamp(heartbeat.last_heartbeat.timestamp() as u64)
            .with_content_type("application/json".into());
        
        channel
            .basic_publish(
                "",
                &self.heartbeat_queue_name,
                BasicPublishOptions::default(),
                &payload,
                properties,
            )
            .await?;
            
        tracing::debug!("💓 Worker heartbeat: {} (load: {}/{})", 
                       heartbeat.worker_id, heartbeat.current_load, heartbeat.max_capacity);
        Ok(())
    }
    
    async fn get_queue_stats(&self) -> Result<QueueStats> {
        let channel = self.connection.create_channel().await?;
        
        // Get queue info for job queue
        let job_queue_info = channel
            .queue_declare(
                &self.job_queue_name,
                QueueDeclareOptions {
                    passive: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;
            
        Ok(QueueStats {
            pending_jobs: job_queue_info.message_count() as i64,
            running_jobs: 0, // Would need to track this separately
            completed_jobs: 0, // Would need to track this separately
            failed_jobs: 0, // Would need to track this separately
            active_workers: 0, // Would need to track this from heartbeats
            average_wait_time_ms: None,
            average_execution_time_ms: None,
        })
    }
}

/// Mock implementation for testing
pub struct MockJobQueue;

impl MockJobQueue {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl JobQueue for MockJobQueue {
    async fn submit_job(&self, job: EvaluationJob) -> Result<()> {
        tracing::info!("Mock: Submitted job {} (submission: {})", job.id, job.submission_id);
        Ok(())
    }
    
    async fn claim_job(&self, worker_id: &str, _capabilities: &[String]) -> Result<Option<EvaluationJob>> {
        tracing::debug!("Mock: Worker {} requested job - none available", worker_id);
        Ok(None)
    }
    
    async fn complete_job(&self, result: EvaluationResult) -> Result<()> {
        tracing::info!("Mock: Completed job {} -> {}", result.submission_id, result.verdict);
        Ok(())
    }
    
    async fn fail_job(&self, job_id: Uuid, error: &str, retry: bool) -> Result<()> {
        tracing::warn!("Mock: Failed job {} - {} (retry: {})", job_id, error, retry);
        Ok(())
    }
    
    async fn worker_heartbeat(&self, heartbeat: WorkerHeartbeat) -> Result<()> {
        tracing::debug!("Mock: Heartbeat from worker {}", heartbeat.worker_id);
        Ok(())
    }
    
    async fn get_queue_stats(&self) -> Result<QueueStats> {
        Ok(QueueStats {
            pending_jobs: 0,
            running_jobs: 0,
            completed_jobs: 0,
            failed_jobs: 0,
            active_workers: 0,
            average_wait_time_ms: None,
            average_execution_time_ms: None,
        })
    }
}