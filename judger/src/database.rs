use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use shared::*;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Database { pool })
    }

    pub async fn get_submission(&self, id: Uuid) -> Result<Option<Submission>> {
        let submission = sqlx::query_as::<_, Submission>(
            "SELECT * FROM submissions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(submission)
    }

    pub async fn get_problem(&self, id: Uuid) -> Result<Option<Problem>> {
        let problem = sqlx::query_as::<_, Problem>(
            "SELECT * FROM problems WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(problem)
    }

    pub async fn get_language(&self, id: Uuid) -> Result<Option<Language>> {
        let language = sqlx::query_as::<_, Language>(
            "SELECT * FROM languages WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(language)
    }

    pub async fn get_question_type(&self, id: Uuid) -> Result<Option<QuestionTypeModel>> {
        let question_type = sqlx::query_as::<_, QuestionTypeModel>(
            "SELECT * FROM question_types WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(question_type)
    }

    pub async fn get_test_cases(&self, problem_id: Uuid) -> Result<Vec<TestCase>> {
        let test_cases = sqlx::query_as::<_, TestCase>(
            "SELECT * FROM test_cases WHERE problem_id = $1 ORDER BY order_index"
        )
        .bind(problem_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(test_cases)
    }

    pub async fn update_submission_status(&self, id: Uuid, status: &str) -> Result<()> {
        sqlx::query(
            "UPDATE submissions SET status = $1 WHERE id = $2"
        )
        .bind(status)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_submission_result(
        &self,
        id: Uuid,
        status: &str,
        verdict: Option<&str>,
        execution_time_ms: Option<i32>,
        execution_memory_kb: Option<i32>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE submissions 
            SET status = $1, verdict = $2, execution_time_ms = $3, execution_memory_kb = $4 
            WHERE id = $5
            "#
        )
        .bind(status)
        .bind(verdict)
        .bind(execution_time_ms)
        .bind(execution_memory_kb)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_submission_result(&self, result: &TestCaseResult, submission_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO submission_results 
            (id, submission_id, test_case_id, verdict, execution_time_ms, execution_memory_kb, stdout, stderr)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(submission_id)
        .bind(result.test_case_id)
        .bind(format!("{:?}", result.verdict))
        .bind(result.execution_time_ms)
        .bind(result.execution_memory_kb)
        .bind(&result.stdout)
        .bind(&result.stderr)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}