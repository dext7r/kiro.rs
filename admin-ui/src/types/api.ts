// 凭据状态响应（分页）
export interface CredentialsStatusResponse {
  total: number
  available: number
  currentId: number
  page: number
  pageSize: number
  totalPages: number
  credentials: CredentialStatusItem[]
}

// 单个凭据状态
export interface CredentialStatusItem {
  id: number
  priority: number
  disabled: boolean
  failureCount: number
  isCurrent: boolean
  expiresAt: string | null
  authMethod: string | null
  hasProfileArn: boolean
}

// 余额响应
export interface BalanceResponse {
  id: number
  subscriptionTitle: string | null
  currentUsage: number
  usageLimit: number
  remaining: number
  usagePercentage: number
  nextResetAt: number | null
}

// 成功响应
export interface SuccessResponse {
  success: boolean
  message: string
}

// 错误响应
export interface AdminErrorResponse {
  error: {
    type: string
    message: string
  }
}

// 请求类型
export interface SetDisabledRequest {
  disabled: boolean
}

export interface SetPriorityRequest {
  priority: number
}

// 添加凭据请求
export interface AddCredentialRequest {
  refreshToken: string
  authMethod?: 'social' | 'idc'
  clientId?: string
  clientSecret?: string
  priority?: number
  region?: string
}

// 添加凭据响应
export interface AddCredentialResponse {
  success: boolean
  message: string
  credentialId: number
}

// 批量导入请求
export interface BatchImportRequest {
  credentials: AddCredentialRequest[]
}

// 批量导入响应
export interface BatchImportResponse {
  imported: number
  failed: number
  errors: BatchImportError[]
}

export interface BatchImportError {
  index: number
  message: string
}

// 批量删除请求
export interface BatchDeleteRequest {
  ids: number[]
}

// 批量删除响应
export interface BatchDeleteResponse {
  deleted: number
  failed: number
  errors: BatchDeleteError[]
}

export interface BatchDeleteError {
  id: number
  message: string
}

// 导出格式
export type ExportFormat = 'json' | 'csv'

// 凭据导出记录
export interface CredentialExportItem {
  id: number
  refreshToken: string
  accessToken: string | null
  profileArn: string | null
  expiresAt: string | null
  authMethod: string
  clientId: string | null
  clientSecret: string | null
  priority: number
  region: string | null
  machineId: string | null
  failureCount: number
  disabled: boolean
}
