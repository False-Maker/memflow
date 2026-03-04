import { useCallback, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Loader2, RefreshCw, X } from 'lucide-react'
import type { Execution } from '../types/agent'

interface AgentHistoryModalProps {
  open: boolean
  onClose: () => void
}

export default function AgentHistoryModal({ open, onClose }: AgentHistoryModalProps) {
  const [executions, setExecutions] = useState<Execution[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const formatTime = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString('zh-CN', {
      timeZone: 'Asia/Shanghai',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    })
  }

  const statusLabel = (status: string) => {
    if (status === 'running') return { text: '执行中', cls: 'text-neon-cyan' }
    if (status === 'success') return { text: '成功', cls: 'text-neon-green' }
    if (status === 'failed') return { text: '失败', cls: 'text-neon-red' }
    if (status === 'cancelled') return { text: '已取消', cls: 'text-gray-400' }
    return { text: status, cls: 'text-gray-400' }
  }

  const buildExecutionSummary = (ex: Execution) => {
    const meta = ex.metadata || undefined
    const asNumber = (value: unknown) => {
      if (typeof value === 'number' && Number.isFinite(value)) return value
      if (typeof value === 'string') {
        const n = Number(value)
        if (Number.isFinite(n)) return n
      }
      return null
    }

    const stepsTotal = asNumber(meta?.steps_total) ?? asNumber(meta?.stepsTotal)
    const stepsSuccess = asNumber(meta?.steps_success) ?? asNumber(meta?.stepsSuccess)
    const durationS = asNumber(meta?.duration_s) ?? asNumber(meta?.durationS)

    const parts: string[] = []
    if (stepsTotal !== null && stepsSuccess !== null) parts.push(`步骤：${stepsSuccess}/${stepsTotal}`)
    if (durationS !== null) parts.push(`耗时：${Math.round(durationS)}s`)
    return parts.join(' · ')
  }

  const loadExecutions = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<Execution[]>('agent_list_executions', { limit: 100, offset: 0 })
      setExecutions(result)
    } catch (e) {
      console.error('加载执行历史失败:', e)
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    if (!open) return
    void loadExecutions()
  }, [open, loadExecutions])

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="glass w-full max-w-3xl max-h-[80vh] rounded-lg flex flex-col">
        <div className="flex items-center justify-between p-4 border-b border-glass-border">
          <div>
            <h2 className="text-xl font-bold text-white">Agent 执行历史</h2>
            <p className="text-xs text-gray-500 mt-1">展示最近的自动化执行记录，便于审计与排错</p>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-surface transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="flex items-center justify-between px-4 pt-3">
          <div className="text-xs text-gray-500">最近 100 条执行记录</div>
          <button
            onClick={loadExecutions}
            className="px-3 py-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-surface transition-colors flex items-center gap-2 text-xs"
          >
            <RefreshCw className="w-3.5 h-3.5" />
            刷新
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-4">
          {error && !loading && (
            <div className="mb-3 p-3 bg-neon-red/10 border border-neon-red/30 rounded-lg text-neon-red text-sm break-words">
              加载失败：{error}
            </div>
          )}

          {loading ? (
            <div className="flex justify-center py-10">
              <Loader2 className="w-6 h-6 animate-spin text-neon-cyan" />
            </div>
          ) : executions.length === 0 ? (
            <div className="text-center py-10 text-gray-500">暂无执行记录</div>
          ) : (
            <div className="space-y-2">
              {executions.map((ex) => {
                const st = statusLabel(ex.status)
                const summary = buildExecutionSummary(ex)
                return (
                  <div
                    key={ex.id}
                    className="p-4 bg-surface/50 border border-glass-border/50 rounded-lg"
                  >
                    <div className="flex items-center justify-between gap-3">
                      <div className="min-w-0">
                        <div className="text-white font-medium">
                          #{ex.id} · {ex.action}
                        </div>
                        <div className="text-xs text-gray-500 mt-1">
                          开始：{formatTime(ex.createdAt)}
                          {ex.finishedAt ? ` · 结束：${formatTime(ex.finishedAt)}` : ''}
                          {summary ? ` · ${summary}` : ''}
                        </div>
                      </div>
                      <div className={`text-sm font-semibold ${st.cls}`}>{st.text}</div>
                    </div>
                    {ex.errorMessage && (
                      <div className="mt-2 text-xs text-neon-red break-words">{ex.errorMessage}</div>
                    )}
                  </div>
                )
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

