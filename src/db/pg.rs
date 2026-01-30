//! PostgreSQL 凭据存储实现

use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;

use super::store::{
    BatchDeleteError, BatchDeleteResult, BatchImportError, BatchImportResult, CreateCredential,
    CredentialRecord, CredentialStore, PaginatedResult, UpdateCredential,
};

/// PostgreSQL 凭据存储
pub struct PgCredentialStore {
    pool: PgPool,
}

impl PgCredentialStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 创建数据库连接池
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    /// 运行数据库迁移（创建表结构）
    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS credentials (
                id              BIGSERIAL PRIMARY KEY,
                refresh_token   TEXT NOT NULL,
                access_token    TEXT,
                profile_arn     TEXT,
                expires_at      TIMESTAMPTZ,
                auth_method     VARCHAR(20) NOT NULL DEFAULT 'social',
                client_id       TEXT,
                client_secret   TEXT,
                priority        INTEGER NOT NULL DEFAULT 0,
                region          VARCHAR(50),
                machine_id      VARCHAR(128),
                failure_count   INTEGER NOT NULL DEFAULT 0,
                disabled        BOOLEAN NOT NULL DEFAULT FALSE,
                created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                deleted_at      TIMESTAMPTZ
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建索引（IF NOT EXISTS）
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_credentials_priority ON credentials(priority) WHERE deleted_at IS NULL",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_credentials_disabled ON credentials(disabled) WHERE deleted_at IS NULL",
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("数据库迁移完成");
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl CredentialStore for PgCredentialStore {
    async fn list(
        &self,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<PaginatedResult<CredentialRecord>> {
        let offset = (page - 1).max(0) * page_size;

        let total: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM credentials WHERE deleted_at IS NULL")
                .fetch_one(&self.pool)
                .await?;

        let rows = sqlx::query_as::<_, CredentialRow>(
            r#"
            SELECT id, refresh_token, access_token, profile_arn, expires_at,
                   auth_method, client_id, client_secret, priority, region,
                   machine_id, failure_count, disabled, created_at, updated_at
            FROM credentials
            WHERE deleted_at IS NULL
            ORDER BY priority ASC, id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let items: Vec<CredentialRecord> = rows.into_iter().map(|r| r.into()).collect();
        Ok(PaginatedResult::new(items, total.0, page, page_size))
    }

    async fn list_all(&self) -> anyhow::Result<Vec<CredentialRecord>> {
        let rows = sqlx::query_as::<_, CredentialRow>(
            r#"
            SELECT id, refresh_token, access_token, profile_arn, expires_at,
                   auth_method, client_id, client_secret, priority, region,
                   machine_id, failure_count, disabled, created_at, updated_at
            FROM credentials
            WHERE deleted_at IS NULL
            ORDER BY priority ASC, id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn get(&self, id: i64) -> anyhow::Result<Option<CredentialRecord>> {
        let row = sqlx::query_as::<_, CredentialRow>(
            r#"
            SELECT id, refresh_token, access_token, profile_arn, expires_at,
                   auth_method, client_id, client_secret, priority, region,
                   machine_id, failure_count, disabled, created_at, updated_at
            FROM credentials
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn create(&self, cred: CreateCredential) -> anyhow::Result<i64> {
        let now = Utc::now();
        let row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO credentials (refresh_token, auth_method, client_id, client_secret,
                                     priority, region, machine_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
        .bind(&cred.refresh_token)
        .bind(&cred.auth_method)
        .bind(&cred.client_id)
        .bind(&cred.client_secret)
        .bind(cred.priority)
        .bind(&cred.region)
        .bind(&cred.machine_id)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    async fn update(&self, id: i64, update: UpdateCredential) -> anyhow::Result<()> {
        let now = Utc::now();

        // 动态构建 SET 子句
        let mut sets = vec!["updated_at = $1".to_string()];
        let mut param_idx = 2u32;

        macro_rules! push_set {
            ($field:ident, $col:expr) => {
                if update.$field.is_some() {
                    sets.push(format!("{} = ${}", $col, param_idx));
                    param_idx += 1;
                }
            };
        }

        push_set!(access_token, "access_token");
        push_set!(refresh_token, "refresh_token");
        push_set!(profile_arn, "profile_arn");
        push_set!(expires_at, "expires_at");
        push_set!(priority, "priority");
        push_set!(failure_count, "failure_count");
        push_set!(disabled, "disabled");
        push_set!(machine_id, "machine_id");

        let sql = format!(
            "UPDATE credentials SET {} WHERE id = ${} AND deleted_at IS NULL",
            sets.join(", "),
            param_idx
        );

        let mut query = sqlx::query(&sql).bind(now);

        if let Some(v) = &update.access_token {
            query = query.bind(v);
        }
        if let Some(v) = &update.refresh_token {
            query = query.bind(v);
        }
        if let Some(v) = &update.profile_arn {
            query = query.bind(v);
        }
        if let Some(v) = &update.expires_at {
            query = query.bind(v);
        }
        if let Some(v) = &update.priority {
            query = query.bind(v);
        }
        if let Some(v) = &update.failure_count {
            query = query.bind(v);
        }
        if let Some(v) = &update.disabled {
            query = query.bind(v);
        }
        if let Some(v) = &update.machine_id {
            query = query.bind(v);
        }

        query = query.bind(id);
        let result = query.execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            anyhow::bail!("凭据不存在: {}", id);
        }

        Ok(())
    }

    async fn delete(&self, id: i64) -> anyhow::Result<()> {
        let result = sqlx::query(
            "UPDATE credentials SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            anyhow::bail!("凭据不存在: {}", id);
        }

        Ok(())
    }

    async fn batch_create(
        &self,
        creds: Vec<CreateCredential>,
    ) -> anyhow::Result<BatchImportResult> {
        let mut imported = 0i32;
        let mut failed = 0i32;
        let mut errors = Vec::new();

        for (index, cred) in creds.into_iter().enumerate() {
            match self.create(cred).await {
                Ok(_) => imported += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(BatchImportError {
                        index,
                        message: e.to_string(),
                    });
                }
            }
        }

        Ok(BatchImportResult {
            imported,
            failed,
            errors,
        })
    }

    async fn batch_delete(&self, ids: Vec<i64>) -> anyhow::Result<BatchDeleteResult> {
        let mut deleted = 0i32;
        let mut failed = 0i32;
        let mut errors = Vec::new();

        for id in ids {
            match self.delete(id).await {
                Ok(_) => deleted += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(BatchDeleteError {
                        id,
                        message: e.to_string(),
                    });
                }
            }
        }

        Ok(BatchDeleteResult {
            deleted,
            failed,
            errors,
        })
    }

    async fn export_all(&self) -> anyhow::Result<Vec<CredentialRecord>> {
        self.list_all().await
    }
}

/// 数据库行映射
#[derive(sqlx::FromRow)]
struct CredentialRow {
    id: i64,
    refresh_token: String,
    access_token: Option<String>,
    profile_arn: Option<String>,
    expires_at: Option<chrono::DateTime<Utc>>,
    auth_method: String,
    client_id: Option<String>,
    client_secret: Option<String>,
    priority: i32,
    region: Option<String>,
    machine_id: Option<String>,
    failure_count: i32,
    disabled: bool,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<CredentialRow> for CredentialRecord {
    fn from(row: CredentialRow) -> Self {
        Self {
            id: row.id,
            refresh_token: row.refresh_token,
            access_token: row.access_token,
            profile_arn: row.profile_arn,
            expires_at: row.expires_at,
            auth_method: row.auth_method,
            client_id: row.client_id,
            client_secret: row.client_secret,
            priority: row.priority,
            region: row.region,
            machine_id: row.machine_id,
            failure_count: row.failure_count,
            disabled: row.disabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
