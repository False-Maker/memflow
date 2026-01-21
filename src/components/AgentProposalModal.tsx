import { useCallback, useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  AlertTriangle,
  ChevronLeft,
  FileText,
  Globe,
  Loader2,
  Play,
  RefreshCw,
  StopCircle,
  X,
} from 'lucide-react'
import type { AgentProposeParams, AutomationProposal, Execution, ExecutionResult, RiskLevel } from '../types/agent'

interface AgentProposalModalProps {
  open: boolean
  onClose: () => void
  onSendToQA?: (text: string) => void
}

export default function AgentProposalModal({ open, onClose, onSendToQA }: AgentProposalModalProps) {
  const [activeTab, setActiveTab] = useState<'proposals' | 'history'>('proposals')
  const [viewMode, setViewMode] = useState<'list' | 'detail'>('list')
  const [loading, setLoading] = useState(false)
  const [loadingHistory, setLoadingHistory] = useState(false)

  const [proposals, setProposals] = useState<AutomationProposal[]>([])
  const [selected, setSelected] = useState<AutomationProposal | null>(null)
  const [proposalsError, setProposalsError] = useState<string | null>(null)
  const [historyError, setHistoryError] = useState<string | null>(null)

  const [executions, setExecutions] = useState<Execution[]>([])
  const [executingId, setExecutingId] = useState<number | null>(null)
  const [polling, setPolling] = useState(false)
  const [actionNotice, setActionNotice] = useState<string | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)
  const [executing, setExecuting] = useState(false)
  const [handoffText, setHandoffText] = useState<string | null>(null)

  const asNumber = (value: unknown) => {
    if (typeof value === 'number' && Number.isFinite(value)) return value
    if (typeof value === 'string') {
      const n = Number(value)
      if (Number.isFinite(n)) return n
    }
    return null
  }

  const buildExecutionSummary = (ex: Execution) => {
    const meta = ex.metadata || undefined
    const stepsTotal = asNumber(meta?.steps_total) ?? asNumber(meta?.stepsTotal)
    const stepsSuccess = asNumber(meta?.steps_success) ?? asNumber(meta?.stepsSuccess)
    const durationS = asNumber(meta?.duration_s) ?? asNumber(meta?.durationS)

    const parts: string[] = []
    if (stepsTotal !== null && stepsSuccess !== null) parts.push(`步骤：${stepsSuccess}/${stepsTotal}`)
    if (durationS !== null) parts.push(`耗时：${Math.round(durationS)}s`)
    return parts.join(' · ')
  }

  const buildCompletionNotice = (ex: Execution) => {
    if (ex.status === 'success') {
      const actions = ex.action.split('+').map((s) => s.trim())
      const effects: string[] = []
      if (actions.includes('copy_to_clipboard')) effects.push('已复制到剪贴板')
      if (actions.includes('create_note')) effects.push('已写入笔记（Documents/memflow_notes.md）')
      if (actions.includes('open_url')) effects.push('已打开链接')
      if (actions.includes('open_file')) effects.push('已打开文件')
      const extra = buildExecutionSummary(ex)
      const head = effects.length > 0 ? `执行完成：${effects.join('，')}` : '执行完成'
      return extra ? `${head}（${extra}）` : head
    }
    if (ex.status === 'cancelled') return '执行已取消'
    if (ex.status === 'failed') return ex.errorMessage ? `执行失败：${ex.errorMessage}` : '执行失败'
    return null
  }

  const proposeParams: AgentProposeParams = useMemo(
    () => ({ timeWindowHours: 24, limit: 10 }),
    []
  )

  const withTimeout = useCallback(async <T,>(p: Promise<T>, ms: number, label: string) => {
    const timeoutPromise = new Promise<T>((_, reject) => {
      const timer = window.setTimeout(() => {
        reject(new Error(label))
      }, ms)
      void p.finally(() => window.clearTimeout(timer))
    })
    return Promise.race([p, timeoutPromise])
  }, [])

  const loadProposals = useCallback(async () => {
    setLoading(true)
    setProposalsError(null)
    try {
      const result = await withTimeout(
        invoke<AutomationProposal[]>('agent_propose_automation', {
          params: proposeParams,
        }),
        90_000,
        '加载智能提案超时（90s），请检查后端日志或网络/模型配置'
      )
      setProposals(result)
    } catch (e) {
      console.error('加载自动化提案失败:', e)
      setProposalsError(String(e))
    } finally {
      setLoading(false)
    }
  }, [proposeParams, withTimeout])

  const refreshExecutionsSilent = useCallback(async () => {
    try {
      const result = await withTimeout(
        invoke<Execution[]>('agent_list_executions', { limit: 50, offset: 0 }),
        20_000,
        '加载执行历史超时（20s）'
      )
      setExecutions(result)
      return result
    } catch (e) {
      console.error('加载执行历史失败:', e)
      return null
    }
  }, [withTimeout])

  const loadExecutions = useCallback(async () => {
    setLoadingHistory(true)
    setHistoryError(null)
    try {
      const res = await refreshExecutionsSilent()
      if (!res) setHistoryError('加载执行历史失败，请查看控制台或后端日志')
    } finally {
      setLoadingHistory(false)
    }
  }, [refreshExecutionsSilent])

  // 打开时初始化
  useEffect(() => {
    if (!open) return
    setActiveTab('proposals')
    setViewMode('list')
    setSelected(null)
    setExecutingId(null)
    setPolling(false)
    setActionNotice(null)
    setActionError(null)
    setExecuting(false)
    setHandoffText(null)
    loadProposals()
    loadExecutions()
  }, [open, loadProposals, loadExecutions])

  // 执行后轮询执行历史（直到该条不再 running 或超时）
  useEffect(() => {
    if (!open) return
    if (!executingId) return
    if (!polling) return

    let cancelled = false
    const startedAt = Date.now()
    const timer = setInterval(async () => {
      if (cancelled) return
      const list = await refreshExecutionsSilent()
      if (!list) {
        setPolling(false)
        setActionError('轮询执行历史失败，请查看控制台或后端日志')
        return
      }
      const current = list?.find((x) => x.id === executingId)
      const done = current && current.status !== 'running'
      const timeout = Date.now() - startedAt > 30_000
      if (done || timeout) {
        setPolling(false)
        if (timeout) {
          setActionNotice(`轮询超时：#${executingId}（可在“执行历史”中手动刷新查看结果）`)
          return
        }
        if (current) {
          const msg = buildCompletionNotice(current)
          if (msg) setActionNotice(msg)

          if (current.status === 'success' && selected) {
            setHandoffText(selected.description)
          }
        }
      }
    }, 1000)

    return () => {
      cancelled = true
      clearInterval(timer)
    }
  }, [open, executingId, polling, refreshExecutionsSilent])

  const formatTime = (timestamp: number) => {
    // 后端目前以 seconds 写入（历史表/agent表一致），显示北京时间
    return new Date(timestamp * 1000).toLocaleString('zh-CN', {
      timeZone: 'Asia/Shanghai',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    })
  }

  const riskLabel = (risk: RiskLevel | string) => {
    if (risk === 'low') return { text: '低风险', cls: 'bg-neon-green/15 text-neon-green border-neon-green/30' }
    if (risk === 'medium') return { text: '中风险', cls: 'bg-yellow-500/15 text-yellow-400 border-yellow-500/30' }
    if (risk === 'high') return { text: '高风险', cls: 'bg-neon-red/15 text-neon-red border-neon-red/30' }
    return { text: String(risk), cls: 'bg-surface/50 text-gray-300 border-glass-border' }
  }

  const statusLabel = (status: string) => {
    if (status === 'running') return { text: '执行中', cls: 'text-neon-blue' }
    if (status === 'success') return { text: '成功', cls: 'text-neon-green' }
    if (status === 'failed') return { text: '失败', cls: 'text-neon-red' }
    if (status === 'cancelled') return { text: '已取消', cls: 'text-gray-400' }
    return { text: status, cls: 'text-gray-400' }
  }

  const handleViewDetail = (p: AutomationProposal) => {
    setSelected(p)
    setViewMode('detail')
  }

  const handleBack = () => {
    setViewMode('list')
    setSelected(null)
  }

  const handleExecute = async () => {
    if (!selected) return
    try {
      setExecuting(true)
      setActionError(null)
      const clipboardStep = selected.steps.find((s) => s.type === 'copy_to_clipboard')
      setHandoffText(clipboardStep?.type === 'copy_to_clipboard' ? clipboardStep.text : null)
      setActionNotice('已提交执行请求...')
      const res = await withTimeout(
        invoke<ExecutionResult>('agent_execute_automation', { proposalId: selected.id }),
        90_000,
        '提交执行请求超时（90s）'
      )
      setExecutingId(res.executionId)
      setPolling(true)
      setActionNotice(`执行已开始：#${res.executionId}`)
      const list = await refreshExecutionsSilent()
      if (!list) {
        setActionError('加载执行历史失败，请查看控制台或后端日志')
      } else {
        setActiveTab('history')
        setViewMode('list')
        setSelected(null)
      }
    } catch (e) {
      console.error('执行自动化失败:', e)
      setActionNotice(null)
      setActionError(String(e))
    }
    finally {
      setExecuting(false)
    }
  }

  const handleCancel = async () => {
    if (!executingId) return
    try {
      setActionError(null)
      setActionNotice(`正在请求停止：#${executingId} ...`)
      await withTimeout(
        invoke('agent_cancel_execution', { executionId: executingId }),
        15_000,
        '发送停止请求超时（15s）'
      )
      setPolling(true)
      const list = await refreshExecutionsSilent()
      if (!list) {
        setActionError('加载执行历史失败，请查看控制台或后端日志')
      }
      setActionNotice(`已发送停止请求：#${executingId}`)
    } catch (e) {
      console.error('取消执行失败:', e)
      setActionNotice(null)
      setActionError(String(e))
    }
  }

  const truncate = (s: string, maxChars: number) => {
    if (s.length <= maxChars) return s
    return `${s.slice(0, maxChars)}...`
  }

  const stepTitle = (s: AutomationProposal['steps'][number]) => {
    if (s.type === 'copy_to_clipboard') return '复制到剪贴板'
    if (s.type === 'open_url') return '打开链接'
    if (s.type === 'open_file') return '打开文件'
    if (s.type === 'create_note') return '归档摘要 (Create Note)'
    return '未知步骤'
  }

  const stepSummary = (s: AutomationProposal['steps'][number]) => {
    if (s.type === 'copy_to_clipboard') return truncate(s.text.replace(/\s+/g, ' ').trim(), 180)
    if (s.type === 'open_url') return s.url
    if (s.type === 'open_file') return s.path
    if (s.type === 'create_note') return truncate(s.content.replace(/\s+/g, ' ').trim(), 180)
    return ''
  }

  if (!open) return null

  const getResourceCounts = (steps: AutomationProposal['steps']) => {
    const urls = steps.filter(s => s.type === 'open_url').length
    const files = steps.filter(s => s.type === 'open_file').length
    const notes = steps.filter(s => s.type === 'create_note').length
    return { urls, files, notes }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="glass w-full max-w-3xl max-h-[80vh] rounded-lg flex flex-col">
        {/* 头部 */}
        <div className="flex items-center justify-between p-4 border-b border-glass-border">
          {viewMode === 'detail' && selected ? (
            <div className="flex items-center gap-3 min-w-0">
              <button onClick={handleBack} className="p-1.5 rounded-lg hover:bg-surface transition-colors">
                <ChevronLeft className="w-5 h-5" />
              </button>
              <h2 className="text-lg font-semibold text-white truncate">{selected.title}</h2>
            </div>
          ) : (
            <div className="min-w-0">
              <h2 className="text-xl font-bold text-white">智能回顾与上下文恢复</h2>
              <div className="text-xs text-gray-500 mt-1">
                MemFlow 自动识别了你今天的主要任务上下文。点击执行可自动归档摘要并打开相关链接，一键恢复工作现场。
              </div>
            </div>
          )}
          <button onClick={onClose} className="p-2 rounded-lg hover:bg-surface transition-colors">
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tab */}
        <div className="flex border-b border-glass-border">
          <button
            onClick={() => {
              setActiveTab('proposals')
              setViewMode('list')
              setSelected(null)
            }}
            className={`flex-1 py-3 text-center transition-colors ${activeTab === 'proposals'
                ? 'text-neon-blue border-b-2 border-neon-blue'
                : 'text-gray-400 hover:text-white'
              }`}
          >
            今日任务
          </button>
          <button
            onClick={() => {
              setActiveTab('history')
              setViewMode('list')
              setSelected(null)
            }}
            className={`flex-1 py-3 text-center transition-colors ${activeTab === 'history'
                ? 'text-neon-blue border-b-2 border-neon-blue'
                : 'text-gray-400 hover:text-white'
              }`}
          >
            回顾历史
          </button>
        </div>

        {/* 内容 */}
        <div className="flex-1 overflow-y-auto p-4">
          {actionNotice && (
            <div className="mb-3 p-3 bg-neon-blue/10 border border-neon-blue/30 rounded-lg text-neon-blue text-sm">
              {actionNotice}
              {handoffText && onSendToQA && (
                <div className="mt-2">
                  <button
                    onClick={() => onSendToQA(`基于以下活动摘要，帮我继续分析并给出下一步建议：\n\n${handoffText}`)}
                    className="px-3 py-1.5 rounded-lg bg-neon-blue/15 text-neon-blue hover:bg-neon-blue/25 transition-colors text-xs"
                  >
                    带到问答继续追问
                  </button>
                </div>
              )}
            </div>
          )}
          {actionError && (
            <div className="mb-3 p-3 bg-neon-red/10 border border-neon-red/30 rounded-lg text-neon-red text-sm break-words">
              {actionError}
            </div>
          )}
          {activeTab === 'proposals' ? (
            viewMode === 'detail' && selected ? (
              <div className="space-y-4">
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="text-white font-semibold">{selected.title}</div>
                    <div className="text-sm text-gray-400 mt-1">{selected.description}</div>
                    <div className="flex items-center gap-2 mt-3 text-xs">
                      <span
                        className={`px-2 py-1 rounded border ${riskLabel(selected.riskLevel).cls}`}
                      >
                        {riskLabel(selected.riskLevel).text}
                      </span>
                      <span className="text-gray-500">置信度</span>
                      <span className="text-gray-200">{Math.round(selected.confidence * 100)}%</span>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={handleExecute}
                      disabled={executing}
                      className="px-4 py-2 rounded-lg bg-neon-blue/20 text-neon-blue hover:bg-neon-blue/30 transition-colors flex items-center gap-2"
                    >
                      {executing ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <Play className="w-4 h-4" />
                      )}
                      {executing ? '执行中...' : '归档并恢复'}
                    </button>
                    <button
                      onClick={handleCancel}
                      disabled={!executingId}
                      className="px-4 py-2 rounded-lg bg-neon-red/15 text-neon-red hover:bg-neon-red/25 transition-colors flex items-center gap-2 disabled:opacity-50"
                    >
                      <StopCircle className="w-4 h-4" />
                      停止
                    </button>
                  </div>
                </div>

                <div className="glass p-4 rounded-lg">
                  <div className="text-sm text-gray-300 mb-2">步骤预览</div>
                  <div className="space-y-2 text-sm">
                    {selected.steps.map((s, idx) => (
                      <div key={idx} className="p-3 bg-surface/50 border border-glass-border/50 rounded-lg">
                        <div className="text-gray-200 font-medium">{stepTitle(s)}</div>
                        <div className="text-xs text-gray-400 mt-1 whitespace-pre-wrap break-words">{stepSummary(s)}</div>
                        <details className="mt-2">
                          <summary className="text-xs text-gray-500 cursor-pointer select-none">查看原始 JSON</summary>
                          <pre className="text-xs text-gray-400 mt-2 whitespace-pre-wrap break-words">
                            {JSON.stringify(s, null, 2)}
                          </pre>
                        </details>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="glass p-4 rounded-lg">
                  <div className="text-sm text-gray-300 mb-2">证据（最近活动片段）</div>
                  {selected.evidence.length === 0 ? (
                    <div className="text-sm text-gray-500">暂无证据</div>
                  ) : (
                    <div className="space-y-2 text-sm">
                      {selected.evidence.map((ev) => (
                        <div
                          key={ev.activityId}
                          className="p-3 bg-surface/50 border border-glass-border/50 rounded-lg"
                        >
                          <div className="flex items-center justify-between gap-3">
                            <div className="text-gray-200 truncate">{ev.appName}</div>
                            <div className="text-xs text-gray-500 flex-shrink-0">{formatTime(ev.timestamp)}</div>
                          </div>
                          <div className="text-xs text-gray-500 mt-1 break-words">{ev.windowTitle}</div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                {executingId && (
                  <div className="flex items-center justify-between p-3 bg-surface/40 border border-glass-border/50 rounded-lg">
                    <div className="text-sm text-gray-300">
                      当前执行：#{executingId}
                      {polling && (
                        <span className="ml-2 text-xs text-gray-500">(轮询中...)</span>
                      )}
                    </div>
                    <button
                      onClick={loadExecutions}
                      className="px-3 py-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-surface transition-colors flex items-center gap-2"
                    >
                      <RefreshCw className="w-4 h-4" />
                      刷新
                    </button>
                  </div>
                )}

                <div className="flex items-start gap-2 text-xs text-gray-500">
                  <AlertTriangle className="w-4 h-4 text-yellow-400/80 flex-shrink-0 mt-0.5" />
                  <div>
                    仅允许低风险步骤执行（allowlist）。当前 MVP 默认提供“复制活动摘要到剪贴板”的提案。
                  </div>
                </div>
              </div>
            ) : (
              <>
                <div className="flex items-center justify-between mb-3">
                  <div className="text-sm text-gray-500">基于最近 {proposeParams.timeWindowHours} 小时活动生成</div>
                  <button
                    onClick={loadProposals}
                    className="px-3 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-surface transition-colors flex items-center gap-2"
                  >
                    <RefreshCw className="w-4 h-4" />
                    刷新
                  </button>
                </div>

                {proposalsError && !loading && (
                  <div className="mb-3 p-3 bg-neon-red/10 border border-neon-red/30 rounded-lg text-neon-red text-sm break-words">
                    加载失败：{proposalsError}
                  </div>
                )}

                {loading ? (
                  <div className="flex justify-center py-10">
                    <Loader2 className="w-6 h-6 animate-spin text-neon-blue" />
                  </div>
                ) : proposals.length === 0 ? (
                  <div className="text-center py-10 text-gray-500">
                    暂无提案（需要先产生一些活动记录）
                  </div>
                ) : (
                  <div className="space-y-2">
                    {proposals.map((p) => (
                      <button
                        key={p.id}
                        onClick={() => handleViewDetail(p)}
                        className="w-full text-left p-4 bg-surface/50 border border-glass-border/50 rounded-lg hover:border-neon-blue/30 hover:bg-surface/80 transition-all"
                      >
                        <div className="flex items-start justify-between gap-3">
                          <div className="min-w-0">
                            <div className="text-white font-medium truncate">{p.title}</div>
                            <div className="text-sm text-gray-500 mt-1 line-clamp-2">{p.description}</div>
                            <div className="flex items-center gap-4 mt-3 text-xs text-gray-500">
                              {(() => {
                                const { urls, files, notes } = getResourceCounts(p.steps)
                                return (
                                  <>
                                    {urls > 0 && (
                                      <div className="flex items-center gap-1.5 text-neon-blue/80">
                                        <Globe className="w-3.5 h-3.5" />
                                        <span>包含 {urls} 个链接</span>
                                      </div>
                                    )}
                                    {files > 0 && (
                                      <div className="flex items-center gap-1.5 text-neon-purple/80">
                                        <FileText className="w-3.5 h-3.5" />
                                        <span>包含 {files} 个文件</span>
                                      </div>
                                    )}
                                    {notes > 0 && (
                                      <div className="flex items-center gap-1.5 text-neon-green/80">
                                        <FileText className="w-3.5 h-3.5" />
                                        <span>生成归档笔记</span>
                                      </div>
                                    )}
                                  </>
                                )
                              })()}
                            </div>
                          </div>
                          <div className="flex flex-col items-end gap-2 flex-shrink-0">
                            <span className={`text-xs px-2 py-1 rounded border ${riskLabel(p.riskLevel).cls}`}>
                              {riskLabel(p.riskLevel).text}
                            </span>
                            <span className="text-xs text-gray-400">{Math.round(p.confidence * 100)}%</span>
                          </div>
                        </div>
                      </button>
                    ))}
                  </div>
                )}
              </>
            )
          ) : (
            <>
              <div className="flex items-center justify-between mb-3">
                <div className="text-sm text-gray-500">最近 50 条执行记录</div>
                <button
                  onClick={loadExecutions}
                  className="px-3 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-surface transition-colors flex items-center gap-2"
                >
                  <RefreshCw className="w-4 h-4" />
                  刷新
                </button>
              </div>

              {historyError && !loadingHistory && (
                <div className="mb-3 p-3 bg-neon-red/10 border border-neon-red/30 rounded-lg text-neon-red text-sm break-words">
                  {historyError}
                </div>
              )}

              {loadingHistory ? (
                <div className="flex justify-center py-10">
                  <Loader2 className="w-6 h-6 animate-spin text-neon-blue" />
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
            </>
          )}
        </div>
      </div>
    </div>
  )
}
