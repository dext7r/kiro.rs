//! Admin API 业务逻辑服务

use std::sync::Arc;

use crate::kiro::model::credentials::KiroCredentials;
use crate::kiro::token_manager::MultiTokenManager;

use super::error::AdminServiceError;
use super::types::{
    AddCredentialRequest, AddCredentialResponse, BalanceResponse, BatchDeleteError,
    BatchDeleteRequest, BatchDeleteResponse, BatchImportError, BatchImportRequest,
    BatchImportResponse, CredentialExportItem, CredentialStatusItem, CredentialsStatusResponse,
};

/// Admin 服务
///
/// 封装所有 Admin API 的业务逻辑
pub struct AdminService {
    token_manager: Arc<MultiTokenManager>,
}

impl AdminService {
    pub fn new(token_manager: Arc<MultiTokenManager>) -> Self {
        Self { token_manager }
    }

    /// 获取所有凭据状态（分页）
    pub fn get_all_credentials(&self, page: i32, page_size: i32) -> CredentialsStatusResponse {
        let snapshot = self.token_manager.snapshot();

        let mut credentials: Vec<CredentialStatusItem> = snapshot
            .entries
            .into_iter()
            .map(|entry| CredentialStatusItem {
                id: entry.id,
                priority: entry.priority,
                disabled: entry.disabled,
                failure_count: entry.failure_count,
                is_current: entry.id == snapshot.current_id,
                expires_at: entry.expires_at,
                auth_method: entry.auth_method,
                has_profile_arn: entry.has_profile_arn,
            })
            .collect();

        // 按优先级排序（数字越小优先级越高）
        credentials.sort_by_key(|c| c.priority);

        let total = credentials.len() as i64;
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as i32;

        // 分页处理
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(credentials.len());
        let paged_credentials = if start < credentials.len() {
            credentials[start..end].to_vec()
        } else {
            vec![]
        };

        CredentialsStatusResponse {
            total,
            available: snapshot.available,
            current_id: snapshot.current_id,
            page,
            page_size,
            total_pages,
            credentials: paged_credentials,
        }
    }

    /// 设置凭据禁用状态
    pub fn set_disabled(&self, id: u64, disabled: bool) -> Result<(), AdminServiceError> {
        // 先获取当前凭据 ID，用于判断是否需要切换
        let snapshot = self.token_manager.snapshot();
        let current_id = snapshot.current_id;

        self.token_manager
            .set_disabled(id, disabled)
            .map_err(|e| self.classify_error(e, id))?;

        // 只有禁用的是当前凭据时才尝试切换到下一个
        if disabled && id == current_id {
            let _ = self.token_manager.switch_to_next();
        }
        Ok(())
    }

    /// 设置凭据优先级
    pub fn set_priority(&self, id: u64, priority: u32) -> Result<(), AdminServiceError> {
        self.token_manager
            .set_priority(id, priority)
            .map_err(|e| self.classify_error(e, id))
    }

    /// 重置失败计数并重新启用
    pub fn reset_and_enable(&self, id: u64) -> Result<(), AdminServiceError> {
        self.token_manager
            .reset_and_enable(id)
            .map_err(|e| self.classify_error(e, id))
    }

    /// 获取凭据余额
    pub async fn get_balance(&self, id: u64) -> Result<BalanceResponse, AdminServiceError> {
        let usage = self
            .token_manager
            .get_usage_limits_for(id)
            .await
            .map_err(|e| self.classify_balance_error(e, id))?;

        let current_usage = usage.current_usage();
        let usage_limit = usage.usage_limit();
        let remaining = (usage_limit - current_usage).max(0.0);
        let usage_percentage = if usage_limit > 0.0 {
            (current_usage / usage_limit * 100.0).min(100.0)
        } else {
            0.0
        };

        Ok(BalanceResponse {
            id,
            subscription_title: usage.subscription_title().map(|s| s.to_string()),
            current_usage,
            usage_limit,
            remaining,
            usage_percentage,
            next_reset_at: usage.next_date_reset,
        })
    }

    /// 添加新凭据
    pub async fn add_credential(
        &self,
        req: AddCredentialRequest,
    ) -> Result<AddCredentialResponse, AdminServiceError> {
        // 构建凭据对象
        let new_cred = KiroCredentials {
            id: None,
            access_token: None,
            refresh_token: Some(req.refresh_token),
            profile_arn: None,
            expires_at: None,
            auth_method: Some(req.auth_method),
            client_id: req.client_id,
            client_secret: req.client_secret,
            priority: req.priority,
            region: req.region,
            machine_id: req.machine_id,
        };

        // 调用 token_manager 添加凭据
        let credential_id = self
            .token_manager
            .add_credential(new_cred)
            .await
            .map_err(|e| self.classify_add_error(e))?;

        Ok(AddCredentialResponse {
            success: true,
            message: format!("凭据添加成功，ID: {}", credential_id),
            credential_id,
        })
    }

    /// 删除凭据
    pub fn delete_credential(&self, id: u64) -> Result<(), AdminServiceError> {
        self.token_manager
            .delete_credential(id)
            .map_err(|e| self.classify_delete_error(e, id))
    }

    /// 批量导入凭据
    pub async fn batch_import(
        &self,
        req: BatchImportRequest,
    ) -> Result<BatchImportResponse, AdminServiceError> {
        let mut imported = 0i32;
        let mut failed = 0i32;
        let mut errors = Vec::new();

        for (index, cred_req) in req.credentials.into_iter().enumerate() {
            let new_cred = KiroCredentials {
                id: None,
                access_token: None,
                refresh_token: Some(cred_req.refresh_token),
                profile_arn: None,
                expires_at: None,
                auth_method: Some(cred_req.auth_method),
                client_id: cred_req.client_id,
                client_secret: cred_req.client_secret,
                priority: cred_req.priority,
                region: cred_req.region,
                machine_id: cred_req.machine_id,
            };

            match self.token_manager.add_credential(new_cred).await {
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

        Ok(BatchImportResponse {
            imported,
            failed,
            errors,
        })
    }

    /// 批量删除凭据
    pub fn batch_delete(&self, req: BatchDeleteRequest) -> BatchDeleteResponse {
        let mut deleted = 0i32;
        let mut failed = 0i32;
        let mut errors = Vec::new();

        for id in req.ids {
            match self.token_manager.delete_credential(id) {
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

        BatchDeleteResponse {
            deleted,
            failed,
            errors,
        }
    }

    /// 导出所有凭据
    pub fn export_all(&self) -> Vec<CredentialExportItem> {
        let snapshot = self.token_manager.snapshot_full();
        snapshot
            .into_iter()
            .map(|entry| CredentialExportItem {
                id: entry.id,
                refresh_token: entry.credentials.refresh_token.unwrap_or_default(),
                access_token: entry.credentials.access_token,
                profile_arn: entry.credentials.profile_arn,
                expires_at: entry.credentials.expires_at,
                auth_method: entry.credentials.auth_method.unwrap_or_else(|| "social".to_string()),
                client_id: entry.credentials.client_id,
                client_secret: entry.credentials.client_secret,
                priority: entry.credentials.priority,
                region: entry.credentials.region,
                machine_id: entry.credentials.machine_id,
                failure_count: entry.failure_count,
                disabled: entry.disabled,
            })
            .collect()
    }

    /// 分类简单操作错误（set_disabled, set_priority, reset_and_enable）
    fn classify_error(&self, e: anyhow::Error, id: u64) -> AdminServiceError {
        let msg = e.to_string();
        if msg.contains("不存在") {
            AdminServiceError::NotFound { id }
        } else {
            AdminServiceError::InternalError(msg)
        }
    }

    /// 分类余额查询错误（可能涉及上游 API 调用）
    fn classify_balance_error(&self, e: anyhow::Error, id: u64) -> AdminServiceError {
        let msg = e.to_string();

        // 1. 凭据不存在
        if msg.contains("不存在") {
            return AdminServiceError::NotFound { id };
        }

        // 2. 上游服务错误特征：HTTP 响应错误或网络错误
        let is_upstream_error =
            // HTTP 响应错误（来自 refresh_*_token 的错误消息）
            msg.contains("凭证已过期或无效") ||
            msg.contains("权限不足") ||
            msg.contains("已被限流") ||
            msg.contains("服务器错误") ||
            msg.contains("Token 刷新失败") ||
            msg.contains("暂时不可用") ||
            // 网络错误（reqwest 错误）
            msg.contains("error trying to connect") ||
            msg.contains("connection") ||
            msg.contains("timeout") ||
            msg.contains("timed out");

        if is_upstream_error {
            AdminServiceError::UpstreamError(msg)
        } else {
            // 3. 默认归类为内部错误（本地验证失败、配置错误等）
            // 包括：缺少 refreshToken、refreshToken 已被截断、无法生成 machineId 等
            AdminServiceError::InternalError(msg)
        }
    }

    /// 分类添加凭据错误
    fn classify_add_error(&self, e: anyhow::Error) -> AdminServiceError {
        let msg = e.to_string();

        // 凭据验证失败（refreshToken 无效、格式错误等）
        let is_invalid_credential = msg.contains("缺少 refreshToken")
            || msg.contains("refreshToken 为空")
            || msg.contains("refreshToken 已被截断")
            || msg.contains("凭证已过期或无效")
            || msg.contains("权限不足")
            || msg.contains("已被限流");

        if is_invalid_credential {
            AdminServiceError::InvalidCredential(msg)
        } else if msg.contains("error trying to connect")
            || msg.contains("connection")
            || msg.contains("timeout")
        {
            AdminServiceError::UpstreamError(msg)
        } else {
            AdminServiceError::InternalError(msg)
        }
    }

    /// 分类删除凭据错误
    fn classify_delete_error(&self, e: anyhow::Error, id: u64) -> AdminServiceError {
        let msg = e.to_string();
        if msg.contains("不存在") {
            AdminServiceError::NotFound { id }
        } else if msg.contains("只能删除已禁用的凭据") {
            AdminServiceError::InvalidCredential(msg)
        } else {
            AdminServiceError::InternalError(msg)
        }
    }
}
