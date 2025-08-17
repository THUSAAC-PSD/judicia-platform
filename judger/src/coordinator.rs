use anyhow::Result;
use futures_util::StreamExt;
use lapin::{
    options::*, types::FieldTable, Connection, ConnectionProperties, Consumer,
};
use std::sync::Arc;
use tokio::sync::Semaphore;
use shared::*;

use crate::{
    config::Config,
    database::Database,
    executor::Executor,
};

pub struct Coordinator {
    config: Arc<Config>,
    db: Database,
    executor: Executor,
    semaphore: Arc<Semaphore>,
}

impl Coordinator {
    pub async fn new(config: Arc<Config>, db: Database) -> Result<Self> {
        let executor = Executor::new(config.clone())?;
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_jobs));

        Ok(Coordinator {
            config,
            db,
            executor,
            semaphore,
        })
    }

    pub async fn start(&self) -> Result<()> {
        let connection = Connection::connect(&self.config.rabbitmq_url, ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;

        // Declare the queue
        channel
            .queue_declare(
                "judging_jobs",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        // Create consumer
        let mut consumer: Consumer = channel
            .basic_consume(
                "judging_jobs",
                "judger_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        tracing::info!("Waiting for judging jobs...");

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    let permit = self.semaphore.clone().acquire_owned().await?;
                    let db = self.db.clone();
                    let executor = self.executor.clone();
                    
                    tokio::spawn(async move {
                        let _permit = permit; // Hold permit until task completes
                        
                        let data = delivery.data.clone();
                        if let Err(e) = Self::process_job(data, db, executor).await {
                            tracing::error!("Failed to process job: {}", e);
                        }
                        
                        if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                            tracing::error!("Failed to ack message: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to consume message: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn process_job(data: Vec<u8>, db: Database, executor: Executor) -> Result<()> {
        let job: JudgingJob = serde_json::from_slice(&data)?;
        tracing::info!("Processing submission {}", job.submission_id);

        // Update status to compiling
        db.update_submission_status(job.submission_id, "Compiling").await?;

        // Get required data
        let _submission = db.get_submission(job.submission_id).await?
            .ok_or_else(|| anyhow::anyhow!("Submission not found"))?;
        
        let problem = db.get_problem(job.problem_id).await?
            .ok_or_else(|| anyhow::anyhow!("Problem not found"))?;
        
        let language = db.get_language(job.language_id).await?
            .ok_or_else(|| anyhow::anyhow!("Language not found"))?;
        
        let question_type = db.get_question_type(problem.question_type_id).await?
            .ok_or_else(|| anyhow::anyhow!("Question type not found"))?;
        
        let test_cases = db.get_test_cases(problem.id).await?;

        // Process based on question type
        let result = match question_type.name.as_str() {
            "ioi-standard" => {
                Self::judge_ioi_standard(&executor, &job, &problem, &language, &test_cases).await
            }
            "output-only" => {
                Self::judge_output_only(&job, &test_cases).await
            }
            "interactive" => {
                Self::judge_interactive(&executor, &job, &problem, &language, &test_cases).await
            }
            _ => Err(anyhow::anyhow!("Unknown question type: {}", question_type.name))
        };

        match result {
            Ok((verdict, execution_time, execution_memory, test_results)) => {
                // Update submission with final result
                db.update_submission_result(
                    job.submission_id,
                    "Finished",
                    Some(&format!("{:?}", verdict)),
                    execution_time,
                    execution_memory,
                ).await?;

                // Store individual test case results
                for result in test_results {
                    db.create_submission_result(&result, job.submission_id).await?;
                }
            }
            Err(e) => {
                tracing::error!("Judging failed: {}", e);
                db.update_submission_result(
                    job.submission_id,
                    "Error",
                    Some("SystemError"),
                    None,
                    None,
                ).await?;
            }
        }

        Ok(())
    }

    async fn judge_ioi_standard(
        executor: &Executor,
        job: &JudgingJob,
        problem: &Problem,
        language: &Language,
        test_cases: &[TestCase],
    ) -> Result<(Verdict, Option<i32>, Option<i32>, Vec<TestCaseResult>)> {
        // Compile the code
        let compile_result = executor.compile(&job.source_code, language).await?;
        if !compile_result.success {
            return Ok((Verdict::CompilationError, None, None, vec![]));
        }

        let mut results = Vec::new();
        let mut total_time = 0;
        let mut max_memory = 0;

        // Run against each test case
        for test_case in test_cases {
            let run_result = executor.run(
                &compile_result.executable_path,
                &test_case.input_data,
                problem.time_limit_ms,
                problem.memory_limit_kb,
            ).await?;

            let verdict = if run_result.exit_code != 0 {
                Verdict::RuntimeError
            } else if run_result.time_ms > problem.time_limit_ms {
                Verdict::TimeLimitExceeded
            } else if run_result.memory_kb > problem.memory_limit_kb {
                Verdict::MemoryLimitExceeded
            } else if run_result.stdout.trim() == test_case.output_data.trim() {
                Verdict::Accepted
            } else {
                Verdict::WrongAnswer
            };

            total_time += run_result.time_ms;
            max_memory = max_memory.max(run_result.memory_kb);

            let test_result = TestCaseResult {
                test_case_id: test_case.id,
                verdict: verdict.clone(),
                execution_time_ms: Some(run_result.time_ms),
                execution_memory_kb: Some(run_result.memory_kb),
                stdout: Some(run_result.stdout),
                stderr: Some(run_result.stderr),
            };

            results.push(test_result);

            // If any test case fails, return early
            if !matches!(verdict, Verdict::Accepted) {
                return Ok((verdict, Some(total_time), Some(max_memory), results));
            }
        }

        Ok((Verdict::Accepted, Some(total_time), Some(max_memory), results))
    }

    async fn judge_output_only(
        job: &JudgingJob,
        test_cases: &[TestCase],
    ) -> Result<(Verdict, Option<i32>, Option<i32>, Vec<TestCaseResult>)> {
        // For output-only problems, the source code is the answer
        let submitted_output = job.source_code.trim();
        
        let mut results = Vec::new();
        
        // Usually there's only one test case for output-only problems
        for test_case in test_cases {
            let verdict = if submitted_output == test_case.output_data.trim() {
                Verdict::Accepted
            } else {
                Verdict::WrongAnswer
            };

            let test_result = TestCaseResult {
                test_case_id: test_case.id,
                verdict: verdict.clone(),
                execution_time_ms: Some(0),
                execution_memory_kb: Some(0),
                stdout: Some(submitted_output.to_string()),
                stderr: None,
            };

            results.push(test_result);

            // Return after first test case (output-only usually has one)
            return Ok((verdict, Some(0), Some(0), results));
        }

        Ok((Verdict::WrongAnswer, Some(0), Some(0), results))
    }

    async fn judge_interactive(
        _executor: &Executor,
        _job: &JudgingJob,
        _problem: &Problem,
        _language: &Language,
        _test_cases: &[TestCase],
    ) -> Result<(Verdict, Option<i32>, Option<i32>, Vec<TestCaseResult>)> {
        // Interactive problems require more complex setup with interactor programs
        // This is a simplified placeholder - full implementation would require
        // running both the user's program and the interactor with proper IPC
        
        // For now, return system error as this needs more implementation
        Ok((Verdict::SystemError, None, None, vec![]))
    }
}