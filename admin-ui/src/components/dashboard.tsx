import { useState, useRef } from 'react'
import { RefreshCw, LogOut, Moon, Sun, Server, Plus, Download, Upload, Trash2, ChevronLeft, ChevronRight } from 'lucide-react'
import { useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { storage } from '@/lib/storage'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { CredentialCard } from '@/components/credential-card'
import { BalanceDialog } from '@/components/balance-dialog'
import { AddCredentialDialog } from '@/components/add-credential-dialog'
import { useCredentials, useBatchImport, useBatchDelete, useExportCredentials } from '@/hooks/use-credentials'
import type { AddCredentialRequest } from '@/types/api'

interface DashboardProps {
  onLogout: () => void
}

export function Dashboard({ onLogout }: DashboardProps) {
  const [selectedCredentialId, setSelectedCredentialId] = useState<number | null>(null)
  const [balanceDialogOpen, setBalanceDialogOpen] = useState(false)
  const [addDialogOpen, setAddDialogOpen] = useState(false)
  const [page, setPage] = useState(1)
  const [pageSize] = useState(20)
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set())
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [darkMode, setDarkMode] = useState(() => {
    if (typeof window !== 'undefined') {
      return document.documentElement.classList.contains('dark')
    }
    return false
  })

  const queryClient = useQueryClient()
  const { data, isLoading, error, refetch } = useCredentials({ page, pageSize })
  const batchImport = useBatchImport()
  const batchDelete = useBatchDelete()
  const exportCredentials = useExportCredentials()

  const toggleDarkMode = () => {
    setDarkMode(!darkMode)
    document.documentElement.classList.toggle('dark')
  }

  const handleViewBalance = (id: number) => {
    setSelectedCredentialId(id)
    setBalanceDialogOpen(true)
  }

  const handleRefresh = () => {
    refetch()
    toast.success('已刷新凭据列表')
  }

  const handleLogout = () => {
    storage.removeApiKey()
    queryClient.clear()
    onLogout()
  }

  const handleSelectCredential = (id: number, selected: boolean) => {
    setSelectedIds(prev => {
      const next = new Set(prev)
      if (selected) {
        next.add(id)
      } else {
        next.delete(id)
      }
      return next
    })
  }

  const handleSelectAll = () => {
    if (data?.credentials) {
      if (selectedIds.size === data.credentials.length) {
        setSelectedIds(new Set())
      } else {
        setSelectedIds(new Set(data.credentials.map(c => c.id)))
      }
    }
  }

  const handleBatchDelete = async () => {
    if (selectedIds.size === 0) return
    if (!confirm(`确定要删除选中的 ${selectedIds.size} 个凭据吗？`)) return

    try {
      const result = await batchDelete.mutateAsync({ ids: Array.from(selectedIds) })
      toast.success(`成功删除 ${result.deleted} 个凭据`)
      if (result.failed > 0) {
        toast.error(`${result.failed} 个凭据删除失败`)
      }
      setSelectedIds(new Set())
    } catch (e) {
      toast.error('批量删除失败')
    }
  }

  const handleExport = async (format: 'json' | 'csv') => {
    try {
      await exportCredentials.mutateAsync(format)
      toast.success(`已导出为 ${format.toUpperCase()} 格式`)
    } catch (e) {
      toast.error('导出失败')
    }
  }

  const handleImportFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    try {
      const text = await file.text()
      let credentials: AddCredentialRequest[] = []

      if (file.name.endsWith('.json')) {
        const parsed = JSON.parse(text)
        credentials = Array.isArray(parsed) ? parsed : parsed.credentials || []
      } else if (file.name.endsWith('.csv')) {
        const lines = text.split('\n').filter(l => l.trim())
        const headers = lines[0].split(',').map(h => h.trim())
        for (let i = 1; i < lines.length; i++) {
          const values = lines[i].split(',').map(v => v.trim())
          const obj: Record<string, string> = {}
          headers.forEach((h, idx) => { obj[h] = values[idx] || '' })
          if (obj.refreshToken || obj.refresh_token) {
            credentials.push({
              refreshToken: obj.refreshToken || obj.refresh_token,
              authMethod: (obj.authMethod || obj.auth_method || 'social') as 'social' | 'idc',
              clientId: obj.clientId || obj.client_id || undefined,
              clientSecret: obj.clientSecret || obj.client_secret || undefined,
              priority: obj.priority ? parseInt(obj.priority) : undefined,
              region: obj.region || undefined,
            })
          }
        }
      }

      if (credentials.length === 0) {
        toast.error('未找到有效的凭据数据')
        return
      }

      const result = await batchImport.mutateAsync({ credentials })
      toast.success(`成功导入 ${result.imported} 个凭据`)
      if (result.failed > 0) {
        toast.error(`${result.failed} 个凭据导入失败`)
      }
    } catch (e) {
      toast.error('导入失败: ' + (e as Error).message)
    }

    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
          <p className="text-muted-foreground">加载中...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background p-4">
        <Card className="w-full max-w-md">
          <CardContent className="pt-6 text-center">
            <div className="text-red-500 mb-4">加载失败</div>
            <p className="text-muted-foreground mb-4">{(error as Error).message}</p>
            <div className="space-x-2">
              <Button onClick={() => refetch()}>重试</Button>
              <Button variant="outline" onClick={handleLogout}>重新登录</Button>
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-background">
      {/* 顶部导航 */}
      <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container flex h-14 items-center justify-between px-4 md:px-8">
          <div className="flex items-center gap-2">
            <Server className="h-5 w-5" />
            <span className="font-semibold">Kiro Admin</span>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="icon" onClick={toggleDarkMode}>
              {darkMode ? <Sun className="h-5 w-5" /> : <Moon className="h-5 w-5" />}
            </Button>
            <Button variant="ghost" size="icon" onClick={handleRefresh}>
              <RefreshCw className="h-5 w-5" />
            </Button>
            <Button variant="ghost" size="icon" onClick={handleLogout}>
              <LogOut className="h-5 w-5" />
            </Button>
          </div>
        </div>
      </header>

      {/* 主内容 */}
      <main className="container px-4 md:px-8 py-6">
        {/* 统计卡片 */}
        <div className="grid gap-4 md:grid-cols-3 mb-6">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                凭据总数
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{data?.total || 0}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                可用凭据
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-green-600">{data?.available || 0}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                当前活跃
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold flex items-center gap-2">
                #{data?.currentId || '-'}
                <Badge variant="success">活跃</Badge>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* 凭据列表 */}
        <div className="space-y-4">
          <div className="flex items-center justify-between flex-wrap gap-2">
            <h2 className="text-xl font-semibold">凭据管理</h2>
            <div className="flex items-center gap-2 flex-wrap">
              {/* 批量操作 */}
              {selectedIds.size > 0 && (
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={handleBatchDelete}
                  disabled={batchDelete.isPending}
                >
                  <Trash2 className="h-4 w-4 mr-2" />
                  删除选中 ({selectedIds.size})
                </Button>
              )}
              {/* 导入导出 */}
              <input
                type="file"
                ref={fileInputRef}
                className="hidden"
                accept=".json,.csv"
                onChange={handleImportFile}
              />
              <Button
                variant="outline"
                size="sm"
                onClick={() => fileInputRef.current?.click()}
                disabled={batchImport.isPending}
              >
                <Upload className="h-4 w-4 mr-2" />
                导入
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleExport('json')}
                disabled={exportCredentials.isPending}
              >
                <Download className="h-4 w-4 mr-2" />
                导出 JSON
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleExport('csv')}
                disabled={exportCredentials.isPending}
              >
                <Download className="h-4 w-4 mr-2" />
                导出 CSV
              </Button>
              <Button onClick={() => setAddDialogOpen(true)} size="sm">
                <Plus className="h-4 w-4 mr-2" />
                添加凭据
              </Button>
            </div>
          </div>

          {/* 全选 */}
          {data && data.credentials.length > 0 && (
            <div className="flex items-center gap-2">
              <Button variant="ghost" size="sm" onClick={handleSelectAll}>
                {selectedIds.size === data.credentials.length ? '取消全选' : '全选当前页'}
              </Button>
              <span className="text-sm text-muted-foreground">
                已选择 {selectedIds.size} 项
              </span>
            </div>
          )}

          {data?.credentials.length === 0 ? (
            <Card>
              <CardContent className="py-8 text-center text-muted-foreground">
                暂无凭据
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {data?.credentials.map((credential) => (
                <CredentialCard
                  key={credential.id}
                  credential={credential}
                  onViewBalance={handleViewBalance}
                  selected={selectedIds.has(credential.id)}
                  onSelectChange={(selected) => handleSelectCredential(credential.id, selected)}
                />
              ))}
            </div>
          )}

          {/* 分页 */}
          {data && data.totalPages > 1 && (
            <div className="flex items-center justify-center gap-4 mt-6">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setPage(p => Math.max(1, p - 1))}
                disabled={page <= 1}
              >
                <ChevronLeft className="h-4 w-4" />
                上一页
              </Button>
              <span className="text-sm text-muted-foreground">
                第 {page} / {data.totalPages} 页，共 {data.total} 条
              </span>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setPage(p => Math.min(data.totalPages, p + 1))}
                disabled={page >= data.totalPages}
              >
                下一页
                <ChevronRight className="h-4 w-4" />
              </Button>
            </div>
          )}
        </div>
      </main>

      {/* 余额对话框 */}
      <BalanceDialog
        credentialId={selectedCredentialId}
        open={balanceDialogOpen}
        onOpenChange={setBalanceDialogOpen}
      />

      {/* 添加凭据对话框 */}
      <AddCredentialDialog
        open={addDialogOpen}
        onOpenChange={setAddDialogOpen}
      />
    </div>
  )
}
