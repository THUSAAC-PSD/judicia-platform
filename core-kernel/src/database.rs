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
    
    pub fn pool(&self) -> &PgPool {
        &self.pool
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

    pub async fn update_user_password(&self, id: Uuid, hashed_password: &str) -> Result<()> {
        sqlx::query(
            "UPDATE users SET hashed_password = $1 WHERE id = $2",
        )
        .bind(hashed_password)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
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

    // Plugin system operations
    pub async fn register_plugin(&self, plugin: &Plugin) -> Result<Plugin> {
        let registered_plugin = sqlx::query_as::<_, Plugin>(
            r#"
            INSERT INTO plugins (id, name, version, author, description, plugin_type, 
                               wasm_path, config_schema, capabilities, status, 
                               installed_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(plugin.id)
        .bind(&plugin.name)
        .bind(&plugin.version)
        .bind(&plugin.author)
        .bind(&plugin.description)
        .bind(&plugin.plugin_type)
        .bind(&plugin.wasm_path)
        .bind(&plugin.config_schema)
        .bind(&plugin.capabilities)
        .bind(&plugin.status)
        .bind(plugin.installed_at)
        .bind(&plugin.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(registered_plugin)
    }

    pub async fn get_plugin(&self, plugin_id: Uuid) -> Result<Option<Plugin>> {
        let plugin = sqlx::query_as::<_, Plugin>(
            "SELECT * FROM plugins WHERE id = $1"
        )
        .bind(plugin_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(plugin)
    }

    pub async fn get_plugin_by_name(&self, name: &str) -> Result<Option<Plugin>> {
        let plugin = sqlx::query_as::<_, Plugin>(
            "SELECT * FROM plugins WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(plugin)
    }

    pub async fn list_plugins(&self, status_filter: Option<&str>) -> Result<Vec<Plugin>> {
        let plugins = if let Some(status) = status_filter {
            sqlx::query_as::<_, Plugin>(
                "SELECT * FROM plugins WHERE status = $1 ORDER BY name"
            )
            .bind(status)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Plugin>(
                "SELECT * FROM plugins ORDER BY name"
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(plugins)
    }

    pub async fn update_plugin_status(&self, plugin_id: Uuid, status: &str) -> Result<()> {
        sqlx::query(
            "UPDATE plugins SET status = $1, last_loaded_at = CASE WHEN $1 = 'active' THEN NOW() ELSE last_loaded_at END WHERE id = $2"
        )
        .bind(status)
        .bind(plugin_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Plugin permissions
    pub async fn grant_plugin_permission(&self, permission: &PluginPermission) -> Result<PluginPermission> {
        let granted_permission = sqlx::query_as::<_, PluginPermission>(
            r#"
            INSERT INTO plugin_permissions (id, plugin_id, capability, database_access_level,
                                          rate_limit_requests_per_second, rate_limit_db_queries_per_minute,
                                          rate_limit_events_per_minute, granted_at, granted_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (plugin_id, capability) DO UPDATE SET
                database_access_level = $4,
                rate_limit_requests_per_second = $5,
                rate_limit_db_queries_per_minute = $6,
                rate_limit_events_per_minute = $7,
                granted_at = $8,
                granted_by = $9
            RETURNING *
            "#,
        )
        .bind(permission.id)
        .bind(permission.plugin_id)
        .bind(&permission.capability)
        .bind(&permission.database_access_level)
        .bind(permission.rate_limit_requests_per_second)
        .bind(permission.rate_limit_db_queries_per_minute)
        .bind(permission.rate_limit_events_per_minute)
        .bind(permission.granted_at)
        .bind(permission.granted_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(granted_permission)
    }

    pub async fn get_plugin_permissions(&self, plugin_id: Uuid) -> Result<Vec<PluginPermission>> {
        let permissions = sqlx::query_as::<_, PluginPermission>(
            "SELECT * FROM plugin_permissions WHERE plugin_id = $1"
        )
        .bind(plugin_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(permissions)
    }

    // Worker node operations
    pub async fn register_worker_node(&self, worker: &WorkerNode) -> Result<WorkerNode> {
        let registered_worker = sqlx::query_as::<_, WorkerNode>(
            r#"
            INSERT INTO worker_nodes (id, node_id, host_address, port, capabilities,
                                    max_concurrent_jobs, current_load, status,
                                    last_heartbeat, registered_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (node_id) DO UPDATE SET
                host_address = $3,
                port = $4,
                capabilities = $5,
                max_concurrent_jobs = $6,
                status = $8,
                last_heartbeat = $9,
                metadata = $11
            RETURNING *
            "#,
        )
        .bind(worker.id)
        .bind(&worker.node_id)
        .bind(&worker.host_address)
        .bind(worker.port)
        .bind(&worker.capabilities)
        .bind(worker.max_concurrent_jobs)
        .bind(worker.current_load)
        .bind(&worker.status)
        .bind(worker.last_heartbeat)
        .bind(worker.registered_at)
        .bind(&worker.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(registered_worker)
    }

    pub async fn get_available_workers(&self) -> Result<Vec<WorkerNode>> {
        let workers = sqlx::query_as::<_, WorkerNode>(
            r#"
            SELECT * FROM worker_nodes 
            WHERE status = 'online' 
            AND current_load < max_concurrent_jobs
            AND last_heartbeat > NOW() - INTERVAL '1 minute'
            ORDER BY current_load ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(workers)
    }

    // Event operations
    pub async fn create_event(&self, event: &Event) -> Result<Event> {
        let created_event = sqlx::query_as::<_, Event>(
            r#"
            INSERT INTO events (id, event_type, source_plugin_id, source_user_id,
                              source_contest_id, source_submission_id, payload, timestamp, processed)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(event.id)
        .bind(&event.event_type)
        .bind(event.source_plugin_id)
        .bind(event.source_user_id)
        .bind(event.source_contest_id)
        .bind(event.source_submission_id)
        .bind(&event.payload)
        .bind(event.timestamp)
        .bind(event.processed)
        .fetch_one(&self.pool)
        .await?;

        Ok(created_event)
    }

    pub async fn get_unprocessed_events(&self, limit: i64) -> Result<Vec<Event>> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT * FROM events WHERE processed = FALSE ORDER BY timestamp ASC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    pub async fn mark_event_processed(&self, event_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE events SET processed = TRUE WHERE id = $1"
        )
        .bind(event_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    
    // Mock implementation for testing
    pub fn new_mock() -> Result<Self> {
        // Create a mock database that doesn't actually connect to anything
        // This is a bit hacky but works for testing plugin loading
        let pool = PgPool::connect_lazy("postgresql://mock/mock").expect("Mock database creation should never fail");
        Ok(Database { pool })
    }
}