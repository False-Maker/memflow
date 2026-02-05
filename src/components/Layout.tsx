import { useState, useEffect } from 'react'
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
    <div className="flex flex-col h-screen bg-void font-sans selection:bg-signal selection:text-black">
      {/* 顶部工具栏 - Technical Brutalism Header */}
      <header className="bg-void border-b border-zinc-800 px-4 py-3 flex items-center justify-between z-50 relative">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-3 group cursor-pointer">
            {/* Technical Monogram Logo */}
            <div className="w-8 h-8 flex items-center justify-center border border-zinc-700 bg-void transition-all duration-300 group-hover:border-signal group-hover:shadow-[0_0_10px_rgba(245,158,11,0.2)]">
              <span className="font-mono font-bold text-lg text-zinc-100 transition-colors duration-300 group-hover:text-signal">E</span>
            </div>
            {/* Brand Text */}
            <span className="text-sm font-bold font-mono tracking-widest text-zinc-100 transition-colors duration-300 group-hover:text-zinc-300">
              MEMFLOW
            </span>
          </div>

          {/* 录制控制 - Mechanical Switch */}
          <div className="flex items-center gap-3 pl-6 border-l border-zinc-800 h-8">
            <button
              onClick={state.isRecording ? stopRecording : startRecording}
              className={`flex items-center gap-2 px-3 py-1 text-xs font-mono tracking-wider uppercase transition-all border ${state.isRecording
                ? 'bg-signal text-black border-signal hover:bg-amber-400'
                : 'bg-transparent text-zinc-400 border-zinc-700 hover:border-zinc-500 hover:text-zinc-200'
                }`}
            >
              {state.isRecording ? (
                <>
                  <span className="w-2 h-2 bg-black"></span>
                  <span>STOP REC</span>
                </>
              ) : (
                <>
                  <span className="w-2 h-2 border border-current"></span>
                  <span>START REC</span>
                </>
              )}
            </button>

            {/* 状态指示器 - Blinking Cursor */}
            <div className="flex items-center gap-2">
              <div
                className={`w-2 h-4 ${state.isRecording ? 'bg-signal animate-blink' : 'bg-zinc-800'}`}
              />
              <span className={`text-xs font-mono ${state.isRecording ? 'text-signal' : 'text-zinc-600'}`}>
                {state.isRecording ? 'RECORDING_ACTIVE' : 'SYSTEM_IDLE'}
              </span>
            </div>
          </div>
        </div>

        {/* 视图切换 - Tab Bar */}
        <div className="flex items-center gap-1">
          {[
            { id: 'timeline', label: 'TIMELINE' },
            { id: 'gallery', label: 'GALLERY' },
            { id: 'replay', label: 'REPLAY' },
            { id: 'graph', label: 'GRAPH' },
            { id: 'stats', label: 'STATS' },
            { id: 'qa', label: 'Q&A' },
          ].map((view) => (
            <button
              key={view.id}
              onClick={() => setCurrentView(view.id as any)}
              className={`px-4 py-2 text-xs font-mono border-b-2 transition-all ${currentView === view.id
                ? 'border-signal text-signal bg-zinc-900/50'
                : 'border-transparent text-zinc-500 hover:text-zinc-300 hover:bg-zinc-900/30'
                }`}
            >
              {view.label}
            </button>
          ))}
        </div>

        {/* 右侧操作按钮 - Minimal Icons */}
        <div className="flex items-center gap-1">
          <button
            onClick={() => setHeatmapOpen(true)}
            className="p-2 text-zinc-500 hover:text-signal hover:bg-zinc-900 transition-all rounded-sm"
            title="Activity"
          >
            <Calendar className="w-4 h-4" />
          </button>
          <button
            onClick={onOpenChatHistory}
            className="p-2 text-zinc-500 hover:text-signal hover:bg-zinc-900 transition-all rounded-sm"
            title="History"
          >
            <History className="w-4 h-4" />
          </button>
          <button
            onClick={onOpenPerformance}
            className="p-2 text-zinc-500 hover:text-signal hover:bg-zinc-900 transition-all rounded-sm"
            title="Performance"
          >
            <BarChart3 className="w-4 h-4" />
          </button>
          <button
            onClick={onOpenFeedback}
            className="p-2 text-zinc-500 hover:text-signal hover:bg-zinc-900 transition-all rounded-sm"
            title="Feedback"
          >
            <MessageSquare className="w-4 h-4" />
          </button>
          <button
            onClick={onOpenSettings}
            className="p-2 text-zinc-500 hover:text-zinc-100 hover:bg-zinc-900 transition-all rounded-sm"
            title="Settings"
          >
            <Settings className="w-4 h-4" />
          </button>
        </div>
      </header>

      {/* 主内容区 */}
      <main className="flex-1 overflow-hidden min-h-0 relative">
        <div className="flex h-full">
          <div className="flex-1 overflow-hidden min-h-0">
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
          <div className="absolute inset-0 z-50 bg-black/80 flex items-center justify-center">
            <div className="bg-void border border-zinc-800 w-[800px] max-w-[90vw] shadow-2xl">
              <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-800 bg-zinc-900/30">
                <h3 className="text-sm font-bold text-zinc-100 uppercase tracking-widest flex items-center gap-2">
                  <Calendar className="w-4 h-4 text-signal" />
                  Activity_Heatmap
                </h3>
                <button
                  onClick={() => setHeatmapOpen(false)}
                  className="text-zinc-500 hover:text-signal transition-colors"
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
