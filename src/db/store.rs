//! 凭据存储抽象层

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 凭据记录（数据库完整字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialRecord {
    pub id: i64,
    pub refresh_token: String,
    pub access_token: Option<String>,
    pub profile_arn: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auth_method: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub priority: i32,
    pub region: Option<String>,
    pub machine_id: Option<String>,
    pub failure_count: i32,
    pub disabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 分页查询结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total: i64, page: i32, page_size: i32) -> Self {
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as i32;
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

/// 创建凭据请求
#[derive(Debug, Clone)]
pub struct CreateCredential {
    pub refresh_token: String,
    pub auth_method: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub priority: i32,
    pub region: Option<String>,
    pub machine_id: Option<String>,
}

/// 更新凭据请求
#[derive(Debug, Clone, Default)]
pub struct UpdateCredential {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub profile_arn: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub priority: Option<i32>,
    pub failure_count: Option<i32>,
    pub disabled: Option<bool>,
    pub machine_id: Option<String>,
}

/// 批量导入结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchImportResult {
    pub imported: i32,
    pub failed: i32,
    pub errors: Vec<BatchImportError>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchImportError {
    pub index: usize,
    pub message: String,
}

/// 批量删除结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteResult {
    pub deleted: i32,
    pub failed: i32,
    pub errors: Vec<BatchDeleteError>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteError {
    pub id: i64,
    pub message: String,
}

/// 凭据存储抽象接口
#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// 分页查询凭据列表
    async fn list(&self, page: i32, page_size: i32) -> anyhow::Result<PaginatedResult<CredentialRecord>>;

    /// 获取所有凭据（不分页，用于内存缓存）
    async fn list_all(&self) -> anyhow::Result<Vec<CredentialRecord>>;

    /// 根据 ID 获取单个凭据
    async fn get(&self, id: i64) -> anyhow::Result<Option<CredentialRecord>>;

    /// 创建新凭据
    async fn create(&self, cred: CreateCredential) -> anyhow::Result<i64>;

    /// 更新凭据
    async fn update(&self, id: i64, update: UpdateCredential) -> anyhow::Result<()>;

    /// 删除凭据（软删除）
    async fn delete(&self, id: i64) -> anyhow::Result<()>;

    /// 批量创建凭据
    async fn batch_create(&self, creds: Vec<CreateCredential>) -> anyhow::Result<BatchImportResult>;

    /// 批量删除凭据
    async fn batch_delete(&self, ids: Vec<i64>) -> anyhow::Result<BatchDeleteResult>;

    /// 导出所有凭据（用于 JSON/CSV 导出）
    async fn export_all(&self) -> anyhow::Result<Vec<CredentialRecord>>;
}
