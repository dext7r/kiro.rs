import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  getCredentials,
  setCredentialDisabled,
  setCredentialPriority,
  resetCredentialFailure,
  getCredentialBalance,
  addCredential,
  deleteCredential,
  batchImportCredentials,
  batchDeleteCredentials,
  exportCredentials,
  downloadExport,
  type PaginationParams,
} from '@/api/credentials'
import type { AddCredentialRequest, BatchImportRequest, BatchDeleteRequest, ExportFormat } from '@/types/api'

// 查询凭据列表（分页）
export function useCredentials(params?: PaginationParams) {
  return useQuery({
    queryKey: ['credentials', params?.page, params?.pageSize],
    queryFn: () => getCredentials(params),
    refetchInterval: 30000,
  })
}

// 查询凭据余额
export function useCredentialBalance(id: number | null) {
  return useQuery({
    queryKey: ['credential-balance', id],
    queryFn: () => getCredentialBalance(id!),
    enabled: id !== null,
    retry: false, // 余额查询失败时不重试（避免重复请求被封禁的账号）
  })
}

// 设置禁用状态
export function useSetDisabled() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ id, disabled }: { id: number; disabled: boolean }) =>
      setCredentialDisabled(id, disabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] })
    },
  })
}

// 设置优先级
export function useSetPriority() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ id, priority }: { id: number; priority: number }) =>
      setCredentialPriority(id, priority),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] })
    },
  })
}

// 重置失败计数
export function useResetFailure() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => resetCredentialFailure(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] })
    },
  })
}

// 添加新凭据
export function useAddCredential() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (req: AddCredentialRequest) => addCredential(req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] })
    },
  })
}

// 删除凭据
export function useDeleteCredential() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => deleteCredential(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] })
    },
  })
}

// 批量导入凭据
export function useBatchImport() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (req: BatchImportRequest) => batchImportCredentials(req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] })
    },
  })
}

// 批量删除凭据
export function useBatchDelete() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (req: BatchDeleteRequest) => batchDeleteCredentials(req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['credentials'] })
    },
  })
}

// 导出凭据
export function useExportCredentials() {
  return useMutation({
    mutationFn: async (format: ExportFormat) => {
      const blob = await exportCredentials(format)
      downloadExport(blob, format)
    },
  })
}
