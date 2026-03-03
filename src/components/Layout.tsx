import { useState, useEffect, ComponentType } from 'react'
import Timeline from './Timeline'
import KnowledgeGraph from './KnowledgeGraph'
import FlowState from './FlowState'
import QnA from './QnA'
import GalleryView from './GalleryView'
import ActivityHeatmap from './ActivityHeatmap'
import ContextSidebar from './ContextSidebar'
import ImmersiveReplay from './ImmersiveReplay'
import { useApp } from '../contexts/AppContext'
import { Settings, History, MessageSquare, BarChart3, Calendar, X } from 'lucide-react'

// 统一图标按钮组件
function IconButton({
  icon: Icon,
  label,
  active = false,
  onClick,
}: {
  icon: ComponentType<{ className?: string }>
  label: string
  active?: boolean
  onClick?: () => void
}) {
  return (
    <button
      onClick={onClick}
      className={`p-2 rounded-lg transition-all duration-200 hover:scale-105 active:scale-95 ${
        active
          ? 'text-neon-cyan bg-neon-cyan/10'
          : 'text-zinc-500 hover:text-neon-cyan hover:bg-white/5'
      }`}
      title={label}
    >
      <Icon className="w-4 h-4" />
    </button>
  )
}

interface LayoutProps {
  onOpenSettings: () => void
  onOpenChatHistory: () => void
  onOpenFeedback: () => void
  onOpenPerformance: () => void
  // 对话会话相关
  currentSessionId?: number | null
  shouldSwitchToQA?: boolean
  onViewSwitched?: () => void
  onSessionCreated?: (sessionId: number) => void
  onStartNewChat?: () => void
  qaDraft?: string | null
  onSendToQA?: (text: string) => void
}

export default function Layout({
  // ... (props unchanged)
  onOpenSettings,
  onOpenChatHistory,
  onOpenFeedback,
  onOpenPerformance,
  currentSessionId,
  shouldSwitchToQA,
  onViewSwitched,
  onSessionCreated,
  qaDraft,
  onSendToQA,
}: LayoutProps) {
  const { state, dispatch, startRecording, stopRecording } = useApp()
  const [heatmapOpen, setHeatmapOpen] = useState(false)

  // 当需要切换到问答视图时自动切换
  useEffect(() => {
    if (shouldSwitchToQA) {
      dispatch({ type: 'SET_VIEW', payload: 'qa' })
      onViewSwitched?.()
    }
  }, [shouldSwitchToQA, onViewSwitched, dispatch])

  const setCurrentView = (view: 'timeline' | 'graph' | 'stats' | 'qa' | 'gallery' | 'replay') => {
    dispatch({ type: 'SET_VIEW', payload: view })
  }

  const currentView = state.currentView as string

  return (
    <div className="flex flex-col h-screen bg-void font-sans selection:bg-neon-cyan selection:text-black">
      {/* 顶部工具栏 - 紧凑版 Header */}
      <header className="h-14 bg-void border-b border-white/10 px-4 flex items-center justify-between z-50 relative">
        <div className="flex items-center gap-4">
          {/* Logo 区域 - 斜切效果 */}
          <div className="flex items-center gap-3 group cursor-pointer">
            {/* 斜切几何 Logo - 参考官网 */}
            <div className="w-8 h-8 relative overflow-hidden flex-shrink-0">
              <div className="absolute inset-0 bg-white transform -skew-x-12 group-hover:skew-x-0 transition-transform duration-500"></div>
              <div className="absolute top-1/2 left-0 w-full h-[2px] bg-black -rotate-12 transform scale-x-150"></div>
            </div>
            {/* Brand Text */}
            <span className="text-xs font-bold tracking-[0.3em] text-zinc-400 group-hover:text-white transition-colors hidden sm:block">
              MEMFLOW
            </span>
          </div>

          {/* 录制控制 - 紧凑圆形按钮 */}
          <div className="flex items-center gap-2 pl-3 border-l border-white/10">
            <button
              onClick={state.isRecording ? stopRecording : startRecording}
              className={`w-8 h-8 rounded-full flex items-center justify-center transition-all duration-300 hover:scale-110 active:scale-95 ${
                state.isRecording
                  ? 'bg-neon-cyan shadow-[0_0_12px_rgba(0,240,255,0.5)]'
                  : 'border border-zinc-700 hover:border-neon-cyan hover:shadow-[0_0_8px_rgba(0,240,255,0.3)]'
              }`}
              title={state.isRecording ? '停止录制' : '开始录制'}
            >
              {state.isRecording ? (
                <div className="w-3 h-3 bg-black rounded-sm" />
              ) : (
                <div className="w-3 h-3 border-2 border-zinc-500 rounded-full" />
              )}
            </button>

            {/* 状态指示器 - 简洁文字 */}
            <span className={`text-xs font-mono ${state.isRecording ? 'text-neon-cyan' : 'text-zinc-600'}`}>
              {state.isRecording ? '● REC' : '○ IDLE'}
            </span>
          </div>
        </div>

        {/* 视图切换 - Pill 胶囊样式 */}
        <div className="flex items-center gap-0.5 bg-zinc-900/50 rounded-lg p-0.5">
          {[
            { id: 'timeline', label: 'T' },
            { id: 'gallery', label: 'G' },
            { id: 'replay', label: 'R' },
            { id: 'graph', label: 'K' },
            { id: 'stats', label: 'S' },
            { id: 'qa', label: 'Q' },
          ].map((view) => (
            <button
              key={view.id}
              onClick={() => setCurrentView(view.id as any)}
              className={`px-2.5 py-1.5 text-[10px] font-mono tracking-wider rounded transition-all duration-200 hover:scale-105 active:scale-95 ${
                currentView === view.id
                  ? 'bg-neon-cyan text-black font-bold shadow-[0_0_10px_rgba(0,240,255,0.4)]'
                  : 'text-zinc-500 hover:text-neon-cyan hover:bg-white/5'
              }`}
              title={view.id.toUpperCase()}
            >
              {view.label}
            </button>
          ))}
        </div>

        {/* 右侧操作按钮 - 统一图标样式 */}
        <div className="flex items-center gap-0.5">
          <IconButton icon={Calendar} label="Activity" onClick={() => setHeatmapOpen(true)} />
          <IconButton icon={History} label="History" onClick={onOpenChatHistory} />
          <IconButton icon={BarChart3} label="Performance" onClick={onOpenPerformance} />
          <IconButton icon={Settings} label="Settings" onClick={onOpenSettings} active={false} />
        </div>
      </header>

      {/* 主内容区 */}
      <main className="flex-1 overflow-hidden min-h-0 relative">
        <div className="flex h-full">
          <div className="flex-1 overflow-hidden min-h-0 pr-12">
            {currentView === 'timeline' && <Timeline />}
            {currentView === 'gallery' && <GalleryView />}
            {currentView === 'replay' && <ImmersiveReplay />}
            {currentView === 'graph' && <KnowledgeGraph />}
            {currentView === 'stats' && <FlowState />}
            {currentView === 'qa' && (
              <QnA
                initialSessionId={currentSessionId}
                onSessionCreated={onSessionCreated}
                draft={qaDraft}
              />
            )}
          </div>
          <ContextSidebar onSendToQA={onSendToQA} />
        </div>

        {/* Heatmap Modal Overlay */}
        {heatmapOpen && (
          <div className="absolute inset-0 z-50 flex items-center justify-center">
            {/* 背景遮罩 */}
            <div className="absolute inset-0 bg-black/70 backdrop-blur-md" onClick={() => setHeatmapOpen(false)} />
            
            {/* 弹窗主体 */}
            <div className="relative bg-black/90 backdrop-blur-xl border border-white/10 rounded-xl w-[800px] max-w-[90vw] shadow-2xl animate-in zoom-in-95 duration-200">
              {/* 斜切装饰 */}
              <div className="absolute top-0 right-0 w-20 h-20 pointer-events-none">
                <div className="absolute top-0 right-0 w-full h-full bg-gradient-to-bl from-neon-cyan/10 to-transparent transform skew-x-12 translate-x-6 -translate-y-3" />
              </div>
              
              {/* 顶部装饰线 */}
              <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-neon-cyan/50 to-transparent" />
              
              <div className="flex items-center justify-between px-4 py-3 border-b border-white/5">
                <h3 className="text-sm font-bold text-white uppercase tracking-widest flex items-center gap-2">
                  <Calendar className="w-4 h-4 text-neon-cyan" />
                  Activity Heatmap
                </h3>
                <button
                  onClick={() => setHeatmapOpen(false)}
                  className="p-2 text-zinc-500 hover:text-white hover:bg-white/5 rounded-lg transition-colors"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>
              <div className="p-4">
                <ActivityHeatmap onClose={() => setHeatmapOpen(false)} />
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  )
}
