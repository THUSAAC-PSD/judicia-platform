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

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    // User operations
    pub async fn create_user(&self, username: &str, email: &str, hashed_password: &str) -> Result<User> {
        self.create_user_with_roles(username, email, hashed_password, vec!["contestant".to_string()]).await
    }

    pub async fn create_user_with_roles(&self, username: &str, email: &str, hashed_password: &str, roles: Vec<String>) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, hashed_password, roles, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(username)
        .bind(email)
        .bind(hashed_password)
        .bind(roles)
        .bind(chrono::Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    // Problem operations
    pub async fn list_problems(&self, contest_id: Option<Uuid>) -> Result<Vec<Problem>> {
        let problems = if let Some(contest_id) = contest_id {
            sqlx::query_as::<_, Problem>(
                "SELECT * FROM problems WHERE contest_id = $1 ORDER BY created_at DESC"
            )
            .bind(contest_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Problem>(
                "SELECT * FROM problems ORDER BY created_at DESC"
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(problems)
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

    pub async fn create_problem(&self, req: &CreateProblemRequest, author_id: Uuid) -> Result<Problem> {
        let problem = sqlx::query_as::<_, Problem>(
            r#"
            INSERT INTO problems (id, title, author_id, created_at, statement, difficulty, 
                                time_limit_ms, memory_limit_kb, question_type_id, metadata, 
                                points, contest_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&req.title)
        .bind(author_id)
        .bind(chrono::Utc::now())
        .bind(&req.statement)
        .bind(&req.difficulty)
        .bind(req.time_limit_ms)
        .bind(req.memory_limit_kb)
        .bind(req.question_type_id)
        .bind(&req.metadata)
        .bind(req.points)
        .bind(req.contest_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(problem)
    }

    // Language operations
    pub async fn list_languages(&self) -> Result<Vec<Language>> {
        let languages = sqlx::query_as::<_, Language>(
            "SELECT * FROM languages ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(languages)
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

    // Submission operations
    pub async fn create_submission(&self, req: &SubmissionRequest, user_id: Uuid) -> Result<Submission> {
        let submission = sqlx::query_as::<_, Submission>(
            r#"
            INSERT INTO submissions (id, user_id, problem_id, language_id, source_code, 
                                   submitted_at, status, verdict, execution_time_ms, 
                                   execution_memory_kb, contest_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(req.problem_id)
        .bind(req.language_id)
        .bind(&req.source_code)
        .bind(chrono::Utc::now())
        .bind("Queued")
        .bind::<Option<String>>(None)
        .bind::<Option<i32>>(None)
        .bind::<Option<i32>>(None)
        .bind::<Option<Uuid>>(None) // contest_id will be set based on problem
        .fetch_one(&self.pool)
        .await?;

        Ok(submission)
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

    pub async fn list_problem_submissions(&self, problem_id: Uuid, user_id: Uuid) -> Result<Vec<Submission>> {
        let submissions = sqlx::query_as::<_, Submission>(
            "SELECT * FROM submissions WHERE problem_id = $1 AND user_id = $2 ORDER BY submitted_at DESC"
        )
        .bind(problem_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(submissions)
    }

    // Contest operations
    pub async fn list_contests(&self) -> Result<Vec<Contest>> {
        let contests = sqlx::query_as::<_, Contest>(
            "SELECT * FROM contests ORDER BY start_time DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(contests)
    }

    pub async fn get_contest(&self, id: Uuid) -> Result<Option<Contest>> {
        let contest = sqlx::query_as::<_, Contest>(
            "SELECT * FROM contests WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(contest)
    }

    pub async fn create_contest(&self, req: &CreateContestRequest, created_by: Uuid) -> Result<Contest> {
        let end_time = req.start_time + chrono::Duration::seconds(req.duration as i64);
        
        let contest = sqlx::query_as::<_, Contest>(
            r#"
            INSERT INTO contests (id, title, description, start_time, end_time, duration, 
                                created_by, participant_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&req.title)
        .bind(&req.description)
        .bind(req.start_time)
        .bind(end_time)
        .bind(req.duration)
        .bind(created_by)
        .bind(0)
        .fetch_one(&self.pool)
        .await?;

        Ok(contest)
    }

    // Test case operations
    pub async fn get_test_cases(&self, problem_id: Uuid) -> Result<Vec<TestCase>> {
        let test_cases = sqlx::query_as::<_, TestCase>(
            "SELECT * FROM test_cases WHERE problem_id = $1 ORDER BY order_index"
        )
        .bind(problem_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(test_cases)
    }

    // Question type operations
    pub async fn get_question_type(&self, id: Uuid) -> Result<Option<QuestionTypeModel>> {
        let question_type = sqlx::query_as::<_, QuestionTypeModel>(
            "SELECT * FROM question_types WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(question_type)
    }

    // Contest admin operations
    pub async fn assign_contest_admin(&self, contest_id: Uuid, user_id: Uuid) -> Result<ContestAdmin> {
        let contest_admin = sqlx::query_as::<_, ContestAdmin>(
            r#"
            INSERT INTO contest_admins (id, contest_id, user_id, assigned_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(contest_id)
        .bind(user_id)
        .bind(chrono::Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(contest_admin)
    }

    pub async fn remove_contest_admin(&self, contest_id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM contest_admins WHERE contest_id = $1 AND user_id = $2"
        )
        .bind(contest_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_contest_admins(&self, contest_id: Uuid) -> Result<Vec<ContestAdminWithUser>> {
        let contest_admins = sqlx::query_as::<_, ContestAdminWithUser>(
            r#"
            SELECT 
                ca.id, 
                ca.contest_id, 
                ca.user_id, 
                ca.assigned_at,
                u.username,
                u.email
            FROM contest_admins ca
            JOIN users u ON ca.user_id = u.id
            WHERE ca.contest_id = $1
            ORDER BY ca.assigned_at
            "#,
        )
        .bind(contest_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(contest_admins)
    }

    pub async fn is_contest_admin(&self, contest_id: Uuid, user_id: Uuid) -> Result<bool> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM contest_admins WHERE contest_id = $1 AND user_id = $2"
        )
        .bind(contest_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 > 0)
    }

    pub async fn get_user_administered_contests(&self, user_id: Uuid) -> Result<Vec<Contest>> {
        let contests = sqlx::query_as::<_, Contest>(
            r#"
            SELECT c.* FROM contests c
            JOIN contest_admins ca ON c.id = ca.contest_id
            WHERE ca.user_id = $1
            ORDER BY c.start_time DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(contests)
    }
}