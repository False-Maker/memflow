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
import { Settings, History, BarChart3, Calendar, X } from 'lucide-react'

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
      {/* 顶部工具栏 - 全套科技感改造 */}
      <header className="h-16 bg-gradient-to-r from-[#030712] via-[#0c1222] to-[#030712] border-b border-white/10 px-6 flex items-center justify-between z-50 relative overflow-hidden">
        {/* 背景装饰 - 网格效果 */}
        <div className="absolute inset-0 opacity-5">
          <div className="absolute inset-0" style={{
            backgroundImage: `
              linear-gradient(rgba(0, 240, 255, 0.1) 1px, transparent 1px),
              linear-gradient(90deg, rgba(0, 240, 255, 0.1) 1px, transparent 1px)
            `,
            backgroundSize: '20px 20px'
          }} />
        </div>

        {/* 顶部扫描线动画 */}
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-neon-cyan to-transparent opacity-50">
          <div className="h-full w-full animate-scan" style={{
            background: 'linear-gradient(90deg, transparent, rgba(0, 240, 255, 0.8), transparent)',
            animation: 'scan 3s linear infinite'
          }} />
        </div>

        {/* 底部发光分割线 */}
        <div className="absolute bottom-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-neon-cyan to-transparent">
          <div className="h-full w-full shadow-[0_0_10px_rgba(0,240,255,0.5)]" />
        </div>

        <div className="flex items-center gap-4 relative z-10">
          {/* Logo 区域 - 增强版 */}
          <div className="flex items-center gap-3 group cursor-pointer">
            {/* 斜切几何 Logo - 发光效果 */}
            <div className="w-10 h-10 relative overflow-hidden flex-shrink-0">
              <div className="absolute inset-0 bg-white transform -skew-x-12 group-hover:skew-x-0 transition-transform duration-500">
                {/* 内部发光 */}
                <div className="absolute inset-0 bg-gradient-to-br from-white/20 to-transparent" />
              </div>
              <div className="absolute top-1/2 left-0 w-full h-[2px] bg-black -rotate-12 transform scale-x-150">
                {/* 发光效果 */}
                <div className="absolute inset-0 shadow-[0_0_8px_rgba(0,240,255,0.8)]" />
              </div>
              {/* 外发光圈 */}
              <div className="absolute inset-0 border-2 border-neon-cyan/30 rounded-lg animate-pulse" />
            </div>
            {/* Brand Text - 增强效果 */}
            <div className="flex flex-col">
              <span className="text-sm font-bold tracking-[0.3em] text-zinc-300 group-hover:text-neon-cyan transition-colors duration-300 hidden sm:block">
                MEMFLOW
              </span>
              <span className="text-[8px] font-mono tracking-[0.2em] text-zinc-600 group-hover:text-neon-cyan/70 transition-colors duration-300">
                INTELLIGENT ASSISTANT
              </span>
            </div>
          </div>

          {/* 录制控制 - 增强版 */}
          <div className="flex items-center gap-3 pl-4 border-l border-white/10">
            <div className="relative">
              {/* 发光背景圈 */}
              {state.isRecording && (
                <div className="absolute inset-0 rounded-full animate-ping opacity-75 bg-neon-cyan/20" />
              )}
              <button
                onClick={state.isRecording ? stopRecording : startRecording}
                className={`relative w-10 h-10 rounded-full flex items-center justify-center transition-all duration-300 hover:scale-110 active:scale-95 ${
                  state.isRecording
                    ? 'bg-neon-cyan shadow-[0_0_20px_rgba(0,240,255,0.6)] border-2 border-neon-cyan/50'
                    : 'border-2 border-zinc-700 hover:border-neon-cyan hover:shadow-[0_0_15px_rgba(0,240,255,0.4)] bg-zinc-900/50'
                }`}
                title={state.isRecording ? '停止录制' : '开始录制'}
              >
                {state.isRecording ? (
                  <div className="w-3.5 h-3.5 bg-black rounded-sm animate-pulse" />
                ) : (
                  <div className="w-3.5 h-3.5 border-2 border-zinc-500 rounded-full" />
                )}
              </button>
            </div>

            {/* 状态指示器 - 增强版 */}
            <div className="flex flex-col">
              <span className={`text-[10px] font-mono font-bold tracking-wider ${
                state.isRecording 
                  ? 'text-neon-cyan animate-pulse' 
                  : 'text-zinc-600'
              }`}>
                {state.isRecording ? '● REC' : '○ IDLE'}
              </span>
              <span className={`text-[8px] font-mono tracking-wider ${
                state.isRecording ? 'text-neon-cyan/70' : 'text-zinc-700'
              }`}>
                {state.isRecording ? 'LIVE' : 'READY'}
              </span>
            </div>
          </div>
        </div>

        {/* 视图切换 - 高级胶囊样式 */}
        <div className="flex items-center gap-1 bg-zinc-900/80 rounded-xl p-1.5 border border-white/5 backdrop-blur-sm relative overflow-hidden">
          {/* 背景发光 */}
          <div className="absolute inset-0 bg-gradient-to-r from-neon-cyan/5 via-neon-cyan/10 to-neon-cyan/5 opacity-0 group-hover:opacity-100 transition-opacity duration-500" />

          {[
            { id: 'timeline', label: 'T', full: 'TIME' },
            { id: 'gallery', label: 'G', full: 'GALLERY' },
            { id: 'replay', label: 'R', full: 'REPLAY' },
            { id: 'graph', label: 'K', full: 'KNOWLEDGE' },
            { id: 'stats', label: 'S', full: 'STATS' },
            { id: 'qa', label: 'Q', full: 'Q&A' },
          ].map((view) => (
            <button
              key={view.id}
              onClick={() => setCurrentView(view.id as any)}
              className={`relative px-3 py-2 text-[10px] font-mono font-bold tracking-widest rounded-lg transition-all duration-300 hover:scale-105 active:scale-95 ${
                currentView === view.id
                  ? 'bg-neon-cyan text-black shadow-[0_0_20px_rgba(0,240,255,0.5)]'
                  : 'text-zinc-500 hover:text-neon-cyan hover:bg-white/5'
              }`}
              title={view.full}
            >
              {currentView === view.id && (
                <div className="absolute inset-0 bg-neon-cyan/20 rounded-lg animate-pulse" />
              )}
              <span className="relative z-10">{view.label}</span>
            </button>
          ))}
        </div>

        {/* 右侧操作按钮 - 增强图标样式 */}
        <div className="flex items-center gap-1">
          <div className="flex items-center gap-1 bg-zinc-900/60 rounded-lg p-1 border border-white/5">
            <IconButton icon={Calendar} label="Activity" onClick={() => setHeatmapOpen(true)} />
            <IconButton icon={History} label="History" onClick={onOpenChatHistory} />
            <IconButton icon={BarChart3} label="Performance" onClick={onOpenPerformance} />
            <IconButton icon={Settings} label="Settings" onClick={onOpenSettings} active={false} />
          </div>
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
