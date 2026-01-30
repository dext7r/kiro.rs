//! Admin API HTTP 处理器

use axum::{
    Json,
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
};

use super::{
    middleware::AdminState,
    types::{
        AddCredentialRequest, BatchDeleteRequest, BatchImportRequest, ExportFormat,
        PaginationQuery, SetDisabledRequest, SetPriorityRequest, SuccessResponse,
    },
};

/// GET /api/admin/credentials
/// 获取所有凭据状态（分页）
pub async fn get_all_credentials(
    State(state): State<AdminState>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    let response = state.service.get_all_credentials(pagination.page, pagination.page_size);
    Json(response)
}

/// POST /api/admin/credentials/:id/disabled
/// 设置凭据禁用状态
pub async fn set_credential_disabled(
    State(state): State<AdminState>,
    Path(id): Path<u64>,
    Json(payload): Json<SetDisabledRequest>,
) -> impl IntoResponse {
    match state.service.set_disabled(id, payload.disabled) {
        Ok(_) => {
            let action = if payload.disabled { "禁用" } else { "启用" };
            Json(SuccessResponse::new(format!("凭据 #{} 已{}", id, action))).into_response()
        }
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// POST /api/admin/credentials/:id/priority
/// 设置凭据优先级
pub async fn set_credential_priority(
    State(state): State<AdminState>,
    Path(id): Path<u64>,
    Json(payload): Json<SetPriorityRequest>,
) -> impl IntoResponse {
    match state.service.set_priority(id, payload.priority) {
        Ok(_) => Json(SuccessResponse::new(format!(
            "凭据 #{} 优先级已设置为 {}",
            id, payload.priority
        )))
        .into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// POST /api/admin/credentials/:id/reset
/// 重置失败计数并重新启用
pub async fn reset_failure_count(
    State(state): State<AdminState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    match state.service.reset_and_enable(id) {
        Ok(_) => Json(SuccessResponse::new(format!(
            "凭据 #{} 失败计数已重置并重新启用",
            id
        )))
        .into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// GET /api/admin/credentials/:id/balance
/// 获取指定凭据的余额
pub async fn get_credential_balance(
    State(state): State<AdminState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    match state.service.get_balance(id).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// POST /api/admin/credentials
/// 添加新凭据
pub async fn add_credential(
    State(state): State<AdminState>,
    Json(payload): Json<AddCredentialRequest>,
) -> impl IntoResponse {
    match state.service.add_credential(payload).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// DELETE /api/admin/credentials/:id
/// 删除凭据
pub async fn delete_credential(
    State(state): State<AdminState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    match state.service.delete_credential(id) {
        Ok(_) => Json(SuccessResponse::new(format!("凭据 #{} 已删除", id))).into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// POST /api/admin/credentials/batch-import
/// 批量导入凭据
pub async fn batch_import(
    State(state): State<AdminState>,
    Json(payload): Json<BatchImportRequest>,
) -> impl IntoResponse {
    match state.service.batch_import(payload).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// POST /api/admin/credentials/batch-delete
/// 批量删除凭据
pub async fn batch_delete(
    State(state): State<AdminState>,
    Json(payload): Json<BatchDeleteRequest>,
) -> impl IntoResponse {
    let response = state.service.batch_delete(payload);
    Json(response)
}

/// 导出格式查询参数
#[derive(serde::Deserialize)]
pub struct ExportQuery {
    #[serde(default = "default_export_format")]
    pub format: ExportFormat,
}

fn default_export_format() -> ExportFormat {
    ExportFormat::Json
}

/// GET /api/admin/credentials/export
/// 导出所有凭据
pub async fn export_credentials(
    State(state): State<AdminState>,
    Query(query): Query<ExportQuery>,
) -> impl IntoResponse {
    let credentials = state.service.export_all();

    match query.format {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&credentials).unwrap_or_default();
            (
                [(header::CONTENT_TYPE, "application/json")],
                [(header::CONTENT_DISPOSITION, "attachment; filename=\"credentials.json\"")],
                json,
            )
                .into_response()
        }
        ExportFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(vec![]);
            // CSV header
            let _ = wtr.write_record([
                "id", "refresh_token", "access_token", "profile_arn", "expires_at",
                "auth_method", "client_id", "client_secret", "priority", "region",
                "machine_id", "failure_count", "disabled",
            ]);
            for cred in &credentials {
                let _ = wtr.write_record([
                    &cred.id.to_string(),
                    &cred.refresh_token,
                    cred.access_token.as_deref().unwrap_or(""),
                    cred.profile_arn.as_deref().unwrap_or(""),
                    cred.expires_at.as_deref().unwrap_or(""),
                    &cred.auth_method,
                    cred.client_id.as_deref().unwrap_or(""),
                    cred.client_secret.as_deref().unwrap_or(""),
                    &cred.priority.to_string(),
                    cred.region.as_deref().unwrap_or(""),
                    cred.machine_id.as_deref().unwrap_or(""),
                    &cred.failure_count.to_string(),
                    &cred.disabled.to_string(),
                ]);
            }
            let csv_data = String::from_utf8(wtr.into_inner().unwrap_or_default()).unwrap_or_default();
            (
                [(header::CONTENT_TYPE, "text/csv")],
                [(header::CONTENT_DISPOSITION, "attachment; filename=\"credentials.csv\"")],
                csv_data,
            )
                .into_response()
        }
    }
}
