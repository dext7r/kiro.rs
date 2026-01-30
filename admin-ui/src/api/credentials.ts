import axios from 'axios'
import { storage } from '@/lib/storage'
import type {
  CredentialsStatusResponse,
  BalanceResponse,
  SuccessResponse,
  SetDisabledRequest,
  SetPriorityRequest,
  AddCredentialRequest,
  AddCredentialResponse,
  BatchImportRequest,
  BatchImportResponse,
  BatchDeleteRequest,
  BatchDeleteResponse,
  ExportFormat,
} from '@/types/api'

// 创建 axios 实例
const api = axios.create({
  baseURL: '/api/admin',
  headers: {
    'Content-Type': 'application/json',
  },
})

// 请求拦截器添加 API Key
api.interceptors.request.use((config) => {
  const apiKey = storage.getApiKey()
  if (apiKey) {
    config.headers['x-api-key'] = apiKey
  }
  return config
})

// 分页参数
export interface PaginationParams {
  page?: number
  pageSize?: number
}

// 获取所有凭据状态（分页）
export async function getCredentials(params?: PaginationParams): Promise<CredentialsStatusResponse> {
  const { data } = await api.get<CredentialsStatusResponse>('/credentials', { params })
  return data
}

// 设置凭据禁用状态
export async function setCredentialDisabled(
  id: number,
  disabled: boolean
): Promise<SuccessResponse> {
  const { data } = await api.post<SuccessResponse>(
    `/credentials/${id}/disabled`,
    { disabled } as SetDisabledRequest
  )
  return data
}

// 设置凭据优先级
export async function setCredentialPriority(
  id: number,
  priority: number
): Promise<SuccessResponse> {
  const { data } = await api.post<SuccessResponse>(
    `/credentials/${id}/priority`,
    { priority } as SetPriorityRequest
  )
  return data
}

// 重置失败计数
export async function resetCredentialFailure(
  id: number
): Promise<SuccessResponse> {
  const { data } = await api.post<SuccessResponse>(`/credentials/${id}/reset`)
  return data
}

// 获取凭据余额
export async function getCredentialBalance(id: number): Promise<BalanceResponse> {
  const { data } = await api.get<BalanceResponse>(`/credentials/${id}/balance`)
  return data
}

// 添加新凭据
export async function addCredential(
  req: AddCredentialRequest
): Promise<AddCredentialResponse> {
  const { data } = await api.post<AddCredentialResponse>('/credentials', req)
  return data
}

// 删除凭据
export async function deleteCredential(id: number): Promise<SuccessResponse> {
  const { data } = await api.delete<SuccessResponse>(`/credentials/${id}`)
  return data
}

// 批量导入凭据
export async function batchImportCredentials(
  req: BatchImportRequest
): Promise<BatchImportResponse> {
  const { data } = await api.post<BatchImportResponse>('/credentials/batch-import', req)
  return data
}

// 批量删除凭据
export async function batchDeleteCredentials(
  req: BatchDeleteRequest
): Promise<BatchDeleteResponse> {
  const { data } = await api.post<BatchDeleteResponse>('/credentials/batch-delete', req)
  return data
}

// 导出凭据
export async function exportCredentials(format: ExportFormat = 'json'): Promise<Blob> {
  const { data } = await api.get('/credentials/export', {
    params: { format },
    responseType: 'blob',
  })
  return data
}

// 下载导出文件
export function downloadExport(blob: Blob, format: ExportFormat) {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `credentials.${format}`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}
