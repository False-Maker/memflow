import { useEffect, useMemo, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { ChevronLeft, ChevronRight, Lightbulb, Clock, ExternalLink, Search, Copy, Check, Sparkles } from 'lucide-react'
import { useApp } from '../contexts/AppContext'
import AgentModal from './AgentModal'

type SuggestedAction = {
  label: string
  action: string // "open_url" | "search" | "copy"
  value: string
}

type ContextSuggestionPayload = {
  context: {
    triggeredAt: number
    appName: string
    windowTitle: string
  }
  relatedMemories: Array<{
    id: number
    timestamp: number
    appName: string
    windowTitle: string
    score?: number | null
  }>
  suggestedActions: SuggestedAction[]
}

export default function ContextSidebar() {
  const { state, dispatch, searchActivities } = useApp()
  const [open, setOpen] = useState(true)
  const [displayed, setDisplayed] = useState<ContextSuggestionPayload | null>(null)
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null)
  const [isAgentOpen, setIsAgentOpen] = useState(false)
  const pendingRef = useRef<ContextSuggestionPayload | null>(null)
  const timerRef = useRef<number | null>(null)

  useEffect(() => {
    const unlisten = listen<ContextSuggestionPayload>('context-suggestion', (event) => {
      pendingRef.current = event.payload
      if (timerRef.current) window.clearTimeout(timerRef.current)
      timerRef.current = window.setTimeout(() => {
        setDisplayed(pendingRef.current)
      }, 4000)
    })

    return () => {
      if (timerRef.current) window.clearTimeout(timerRef.current)
      unlisten.then((fn) => fn())
    }
  }, [])

  const headerTitle = useMemo(() => {
    if (!displayed) return '上下文助理'
    return displayed.context.appName || '上下文助理'
  }, [displayed])

  const formatTime = (timestampSeconds: number) =>
    new Date(timestampSeconds * 1000).toLocaleString('zh-CN', {
      timeZone: 'Asia/Shanghai',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    })

  const jumpToMemory = async (m: ContextSuggestionPayload['relatedMemories'][number]) => {
    dispatch({ type: 'SET_VIEW', payload: 'timeline' })
    const fromTs = Math.max(0, m.timestamp - 30 * 60)
    const toTs = m.timestamp + 30 * 60
    await searchActivities({
      appName: m.appName,
      fromTs,
      toTs,
      limit: 200,
      offset: 0,
      orderBy: 'time',
    })
  }

  const handleAction = async (action: SuggestedAction, index: number) => {
    switch (action.action) {
      case 'open_url':
        try {
          await invoke('open_external_url', { url: action.value })
        } catch (e) {
          console.error('Failed to open URL', e)
        }
        break
      case 'search':
        dispatch({ type: 'SET_VIEW', payload: 'timeline' })
        await searchActivities({
          query: action.value,
          limit: 100,
          orderBy: 'rank', // Use rank if available, otherwise time
        })
        break
      case 'copy':
        try {
          await navigator.clipboard.writeText(action.value)
          setCopiedIndex(index)
          setTimeout(() => setCopiedIndex(null), 2000)
        } catch (e) {
          console.error('Failed to copy', e)
        }
        break
    }
  }

  const getActionIcon = (type: string, index: number) => {
    if (type === 'copy' && copiedIndex === index) return <Check className="w-4 h-4 text-green-400" />
    switch (type) {
      case 'open_url': return <ExternalLink className="w-4 h-4 text-neon-blue" />
      case 'search': return <Search className="w-4 h-4 text-neon-purple" />
      case 'copy': return <Copy className="w-4 h-4 text-gray-400" />
      default: return <Lightbulb className="w-4 h-4 text-yellow-400" />
    }
  }

  const modelLabel = state.config.chatModel || 'gpt-4o-mini'
  const proactiveEnabled =
    state.config.aiEnabled && state.config.enableProactiveAssistant && !state.config.privacyModeEnabled
  const proactiveReady = proactiveEnabled && state.isRecording

  const disabledReason = (() => {
    if (!state.isRecording) return '未开始录制'
    if (state.config.privacyModeEnabled) return '隐私模式已开启'
    if (!state.config.aiEnabled) return 'AI 未启用'
    if (!state.config.enableProactiveAssistant) return '主动助理未启用'
    return null
  })()

  return (
    <>
      <AgentModal open={isAgentOpen} onClose={() => setIsAgentOpen(false)} />
      <aside
        className={`h-full border-l border-zinc-800 bg-void transition-all duration-300 ${open ? 'w-[320px]' : 'w-[52px]'
          } flex flex-col shrink-0 z-20`}
      >
        <div className="h-full flex flex-col overflow-hidden">
          <div className="flex items-center justify-between px-3 py-3 border-b border-zinc-800 bg-zinc-900/30">
            <div className="flex items-center gap-3 min-w-0">
              {/* Technical Monogram Logo (Mini) */}
              <div className={`w-8 h-8 flex items-center justify-center border border-zinc-700 bg-void transition-all duration-300 hover:border-signal shrink-0 ${!open && 'mx-auto'}`}>
                <span className="font-mono font-bold text-lg text-zinc-100 transition-colors duration-300 hover:text-signal">E</span>
              </div>
              {open && (
                <div className="min-w-0 flex-1 animate-in fade-in slide-in-from-left-2 duration-200">
                  <div className="text-xs font-bold text-zinc-100 uppercase tracking-wider truncate">{headerTitle}</div>
                  {displayed && (
                    <div className="text-[10px] font-mono text-zinc-500 truncate">{displayed.context.windowTitle}</div>
                  )}
                </div>
              )}
            </div>
            {open && (
              <button
                onClick={() => setOpen(false)}
                className="p-1.5 rounded-md text-gray-500 hover:text-white hover:bg-white/10 transition-colors"
              >
                <ChevronRight className="w-4 h-4" />
              </button>
            )}
          </div>

          {!open && (
            <button
              onClick={() => setOpen(true)}
              className="mt-2 mx-auto p-2 rounded-md text-gray-500 hover:text-white hover:bg-white/10 transition-colors"
            >
              <ChevronLeft className="w-4 h-4" />
            </button>
          )}

          {open && (
            <div className="flex-1 overflow-y-auto p-4 space-y-6 custom-scrollbar">
              <div className="flex items-center justify-between px-1 text-[10px] font-mono text-zinc-600 uppercase tracking-wider">
                <span>Model: {modelLabel}</span>
                <span>{proactiveReady ? 'ACTIVE' : disabledReason ? disabledReason : 'INACTIVE'}</span>
              </div>

              {/* Agent Trigger */}
              <button
                onClick={() => setIsAgentOpen(true)}
                className="w-full flex items-center gap-3 p-3 border border-zinc-800 hover:border-signal/50 hover:bg-zinc-900/50 transition-all group"
              >
                <div className="p-2 bg-zinc-900 text-zinc-400 group-hover:text-signal transition-colors">
                  <Sparkles className="w-4 h-4" />
                </div>
                <div className="text-left">
                  <div className="text-xs font-bold text-zinc-300 group-hover:text-signal uppercase tracking-wider transition-colors">Deep Automation</div>
                  <div className="text-[10px] text-zinc-600 font-mono">GENERATE_WORKFLOW</div>
                </div>
              </button>

              {!displayed ? (
                <div className="flex flex-col items-center justify-center h-40 text-gray-500 gap-2 opacity-60">
                  <div className="w-2 h-2 bg-neon-blue rounded-full animate-ping" />
                  <span className="text-xs">
                    {proactiveReady ? '等待上下文触发...' : disabledReason ? `上下文助理不可用：${disabledReason}` : '上下文助理未启用'}
                  </span>
                </div>
              ) : (
                <>
                  {/* 1. Context Info */}
                  <div className="flex items-center gap-2 text-xs text-gray-500 px-1">
                    <Clock className="w-3 h-3" />
                    <span>触发于 {formatTime(displayed.context.triggeredAt)}</span>
                  </div>

                  {/* 2. Suggested Actions */}
                  {displayed.suggestedActions.length > 0 && (
                    <section className="space-y-3 animate-in fade-in slide-in-from-bottom-2 duration-300 delay-100">
                      <h3 className="text-xs font-bold text-zinc-500 uppercase tracking-widest px-1 font-mono">
                        SUGGESTED_ACTIONS
                      </h3>
                      <div className="grid gap-2">
                        {displayed.suggestedActions.map((action, idx) => (
                          <button
                            key={idx}
                            onClick={() => handleAction(action, idx)}
                            className="flex items-center gap-3 p-3 bg-void border border-zinc-800 hover:border-signal/50 hover:bg-zinc-900 transition-all group text-left"
                          >
                            <div className="shrink-0 p-2 bg-zinc-900 group-hover:bg-zinc-800 transition-colors">
                              {getActionIcon(action.action, idx)}
                            </div>
                            <div className="min-w-0">
                              <div className="text-xs font-bold text-zinc-300 group-hover:text-signal transition-colors uppercase">
                                {action.label}
                              </div>
                              <div className="text-[10px] text-zinc-600 font-mono truncate">
                                {action.action === 'open_url' ? 'LINK' : action.action === 'search' ? 'SEARCH' : 'COPY'}
                              </div>
                            </div>
                          </button>
                        ))}
                      </div>
                    </section>
                  )}

                  {/* 3. Related Memories */}
                  <section className="space-y-3 animate-in fade-in slide-in-from-bottom-2 duration-300 delay-200">
                    <h3 className="text-xs font-bold text-zinc-500 uppercase tracking-widest px-1 font-mono">
                      RELATED_MEMORIES
                    </h3>
                    {displayed.relatedMemories.length === 0 ? (
                      <div className="text-xs text-zinc-600 font-mono italic px-1">NO_DATA</div>
                    ) : (
                      <div className="space-y-2">
                        {displayed.relatedMemories.map((m) => (
                          <button
                            key={m.id}
                            type="button"
                            onClick={() => void jumpToMemory(m)}
                            className="w-full text-left p-3 bg-void border border-zinc-800 hover:border-signal/50 hover:bg-zinc-900 transition-all group relative"
                          >
                            <div className="absolute left-0 top-0 bottom-0 w-0.5 bg-transparent group-hover:bg-signal transition-colors" />
                            <div className="text-xs font-bold text-zinc-300 group-hover:text-white truncate mb-1">
                              {m.windowTitle}
                            </div>
                            <div className="flex items-center justify-between text-[10px] text-zinc-600 font-mono">
                              <span className="truncate max-w-[70%] uppercase">{m.appName}</span>
                              <span>{new Date(m.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                            </div>
                          </button>
                        ))}
                      </div>
                    )}
                  </section>
                </>
              )}
            </div>
          )}
        </div>
      </aside>
    </>
  )
}
