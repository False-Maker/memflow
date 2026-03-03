import { useEffect, useMemo, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { ChevronLeft, Lightbulb, Clock, ExternalLink, Search, Copy, Check, Sparkles } from 'lucide-react'
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

export default function ContextSidebar({ onSendToQA }: { onSendToQA?: (text: string) => void }) {
  const { state, dispatch, searchActivities } = useApp()
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
      <AgentModal open={isAgentOpen} onClose={() => setIsAgentOpen(false)} onSendToQA={onSendToQA} />
      {/* 悬浮展开式侧边栏 */}
      <aside
        className="fixed right-0 top-14 bottom-0 w-12 bg-void/95 backdrop-blur border-l border-zinc-800 transition-all duration-300 group z-30 hover:w-72"
      >
        {/* 默认状态：仅图标 */}
        <div className="w-12 h-full flex flex-col items-center py-3 gap-3">
          {/* 状态指示 */}
          <div 
            className={`w-2 h-2 rounded-full transition-all ${
proactiveReady 
                ? 'bg-neon-cyan shadow-[0_0_8px_rgba(0,240,255,0.5)] animate-pulse' 
                : 'bg-zinc-800'
            }`} 
            title={proactiveReady ? 'AI 助理活跃' : disabledReason || '未启用'}
          />
          
          {/* 展开提示 - 悬停自动展开 */}
          <div className="p-2 text-zinc-600 group-hover:text-neon-cyan transition-colors">
            <Sparkles className="w-4 h-4" />
          </div>
          
          <div className="flex-1" />
          
          {/* 展开箭头 */}
          <ChevronLeft className="w-3 h-3 text-zinc-700 group-hover:rotate-180 transition-transform duration-300" />
        </div>
        
        {/* 展开内容 - 悬浮显示 */}
        <div className="absolute left-12 top-0 w-60 h-full opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none group-hover:pointer-events-auto">
          <div className="h-full flex flex-col overflow-hidden border-l border-zinc-800/50 bg-void/95">
            {/* 头部 */}
            <div className="flex items-center justify-between px-3 py-3 border-b border-zinc-800 bg-zinc-900/30">
              <div className="min-w-0 flex-1">
                <div className="text-xs font-bold text-zinc-100 uppercase tracking-wider truncate">
                  {headerTitle}
                </div>
                {displayed && (
                  <div className="text-[10px] font-mono text-zinc-500 truncate">
                    {displayed.context.windowTitle}
                  </div>
                )}
              </div>
            </div>

            {/* 内容区 */}
            <div className="flex-1 overflow-y-auto p-3 space-y-4 custom-scrollbar">
              {/* 状态栏 */}
              <div className="flex items-center justify-between px-1 text-[10px] font-mono text-zinc-600 uppercase tracking-wider">
                <span className="truncate">{modelLabel}</span>
                <span className={proactiveReady ? 'text-neon-cyan' : ''}>
                  {proactiveReady ? 'ACTIVE' : disabledReason ? disabledReason : 'INACTIVE'}
                </span>
              </div>

              {/* Deep Automation 按钮 */}
              <button
                onClick={() => setIsAgentOpen(true)}
className="w-full flex items-center gap-2 p-2 border border-zinc-800 hover:border-neon-cyan/50 hover:bg-zinc-900/50 transition-all group/btn"
              >
                <Sparkles className="w-4 h-4 text-zinc-500 group-hover/btn:text-neon-cyan transition-colors" />
                <span className="text-xs font-bold text-zinc-400 group-hover/btn:text-neon-cyan uppercase tracking-wider transition-colors">
                  Deep Automation
                </span>
              </button>

              {/* 等待状态 */}
              {!displayed ? (
                <div className="flex flex-col items-center justify-center py-8 text-zinc-600 gap-2">
                  <div className={`w-1.5 h-1.5 rounded-full ${proactiveReady ? 'bg-neon-cyan animate-pulse' : 'bg-zinc-800'}`} />
                  <span className="text-[10px] font-mono text-center">
                    {proactiveReady ? '等待触发...' : disabledReason || '未启用'}
                  </span>
                </div>
              ) : (
                <>
                  {/* 触发时间 */}
                  <div className="flex items-center gap-2 text-[10px] text-zinc-500 px-1">
                    <Clock className="w-3 h-3" />
                    <span>{formatTime(displayed.context.triggeredAt)}</span>
                  </div>

                  {/* 建议操作 */}
                  {displayed.suggestedActions.length > 0 && (
                    <div className="space-y-2">
                      <span className="text-[10px] font-mono text-zinc-600 uppercase px-1 block">
                        Actions
                      </span>
                      {displayed.suggestedActions.map((action, idx) => (
                        <button
                          key={idx}
                          onClick={() => handleAction(action, idx)}
                          className="w-full flex items-center gap-2 p-2 bg-void border border-zinc-800 hover:border-neon-cyan/50 transition-all group text-left"
                        >
                          <div className="shrink-0 p-1.5 bg-zinc-900 group-hover:bg-zinc-800 transition-colors">
                            {getActionIcon(action.action, idx)}
                          </div>
                          <span className="text-xs text-zinc-400 group-hover:text-zinc-300 truncate">
                            {action.label}
                          </span>
                        </button>
                      ))}
                    </div>
                  )}

                  {/* 相关记忆 */}
                  {displayed.relatedMemories.length > 0 && (
                    <div className="space-y-2">
                      <span className="text-[10px] font-mono text-zinc-600 uppercase px-1 block">
                        Memories
                      </span>
                      {displayed.relatedMemories.map((m) => (
                        <button
                          key={m.id}
                          type="button"
                          onClick={() => void jumpToMemory(m)}
className="w-full text-left p-2 bg-void border border-zinc-800 hover:border-neon-cyan/50 transition-all group relative"
                        >
                          <div className="absolute left-0 top-0 bottom-0 w-0.5 bg-transparent group-hover:bg-neon-cyan transition-colors" />
                          <div className="text-xs text-zinc-400 group-hover:text-zinc-300 truncate">
                            {m.windowTitle}
                          </div>
                          <div className="flex items-center justify-between text-[10px] text-zinc-600 font-mono mt-1">
                            <span className="truncate uppercase">{m.appName}</span>
                            <span>{new Date(m.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
                          </div>
                        </button>
                      ))}
                    </div>
                  )}
                </>
              )}
            </div>
          </div>
        </div>
      </aside>
    </>
  )
}
