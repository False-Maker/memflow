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
import { Play, Pause, Settings, Zap, History, MessageSquare, BarChart3, Calendar, X } from 'lucide-react'

interface LayoutProps {
  onOpenSettings: () => void
  onOpenAgentProposal: () => void
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
}

export default function Layout({
  // ... (props unchanged)
  onOpenSettings,
  onOpenAgentProposal,
  onOpenChatHistory,
  onOpenFeedback,
  onOpenPerformance,
  currentSessionId,
  shouldSwitchToQA,
  onViewSwitched,
  onSessionCreated,
  qaDraft,
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
    <div className="flex flex-col h-screen bg-void">
      {/* 顶部工具栏 */}
      <header className="glass border-b border-glass-border px-4 py-3 flex items-center justify-between">
        {/* ... (Logo and Recording controls unchanged) */}
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-bold text-neon-blue">MemFlow</h1>
          
          {/* 录制控制 */}
          <button
            onClick={state.isRecording ? stopRecording : startRecording}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-all ${
              state.isRecording
                ? 'bg-neon-red/20 text-neon-red hover:bg-neon-red/30'
                : 'bg-neon-green/20 text-neon-green hover:bg-neon-green/30'
            }`}
          >
            {state.isRecording ? (
              <>
                <Pause className="w-4 h-4" />
                <span>停止录制</span>
              </>
            ) : (
              <>
                <Play className="w-4 h-4" />
                <span>开始录制</span>
              </>
            )}
          </button>

          {/* 状态指示器 */}
          <div className="flex items-center gap-2">
            <div
              className={`w-2 h-2 rounded-full ${
                state.isRecording ? 'bg-neon-red animate-pulse-slow' : 'bg-gray-500'
              }`}
            />
            <span className="text-sm text-gray-400">
              {state.isRecording ? '录制中' : '已停止'}
            </span>
          </div>
        </div>

        {/* 视图切换 */}
        <div className="flex items-center gap-2">
          {/* ... (view buttons unchanged) */}
          <button
            onClick={() => setCurrentView('timeline')}
            className={`px-3 py-2 rounded-lg transition-all ${
              currentView === 'timeline'
                ? 'bg-neon-blue/20 text-neon-blue'
                : 'text-gray-400 hover:text-white'
            }`}
          >
            时间轴
          </button>
          <button
            onClick={() => setCurrentView('gallery')}
            className={`px-3 py-2 rounded-lg transition-all ${
              currentView === 'gallery'
                ? 'bg-neon-pink/20 text-neon-pink'
                : 'text-gray-400 hover:text-white'
            }`}
          >
            画廊
          </button>
          <button
            onClick={() => setCurrentView('replay')}
            className={`px-3 py-2 rounded-lg transition-all ${
              currentView === 'replay'
                ? 'bg-neon-blue/20 text-neon-blue'
                : 'text-gray-400 hover:text-white'
            }`}
          >
            时光机
          </button>
          <button
            onClick={() => setCurrentView('graph')}
            className={`px-3 py-2 rounded-lg transition-all ${
              currentView === 'graph'
                ? 'bg-neon-purple/20 text-neon-purple'
                : 'text-gray-400 hover:text-white'
            }`}
          >
            知识图谱
          </button>
          <button
            onClick={() => setCurrentView('stats')}
            className={`px-3 py-2 rounded-lg transition-all ${
              currentView === 'stats'
                ? 'bg-neon-green/20 text-neon-green'
                : 'text-gray-400 hover:text-white'
            }`}
          >
            统计
          </button>
          <button
            onClick={() => setCurrentView('qa')}
            className={`px-3 py-2 rounded-lg transition-all ${
              currentView === 'qa'
                ? 'bg-neon-blue/20 text-neon-blue'
                : 'text-gray-400 hover:text-white'
            }`}
          >
            问答
          </button>
        </div>

        {/* 右侧操作按钮 */}
        <div className="flex items-center gap-2">
          <button
            onClick={() => setHeatmapOpen(true)}
            className="p-2 rounded-lg text-gray-400 hover:text-neon-blue hover:bg-neon-blue/10 transition-all"
            title="活动热力图"
          >
            <Calendar className="w-5 h-5" />
          </button>
          <button
            onClick={onOpenAgentProposal}
            className="p-2 rounded-lg text-gray-400 hover:text-neon-blue hover:bg-neon-blue/10 transition-all"
            title="自动化动作"
          >
            <Zap className="w-5 h-5" />
          </button>
          <button
            onClick={onOpenChatHistory}
            className="p-2 rounded-lg text-gray-400 hover:text-neon-purple hover:bg-neon-purple/10 transition-all"
            title="对话历史"
          >
            <History className="w-5 h-5" />
          </button>
          <button
            onClick={onOpenPerformance}
            className="p-2 rounded-lg text-gray-400 hover:text-neon-green hover:bg-neon-green/10 transition-all"
            title="性能监控"
          >
            <BarChart3 className="w-5 h-5" />
          </button>
          <button
            onClick={onOpenFeedback}
            className="p-2 rounded-lg text-gray-400 hover:text-neon-blue hover:bg-neon-blue/10 transition-all"
            title="反馈"
          >
            <MessageSquare className="w-5 h-5" />
          </button>
          <button
            onClick={onOpenSettings}
            className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-surface transition-all"
            title="设置"
          >
            <Settings className="w-5 h-5" />
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
          <ContextSidebar />
        </div>
        
        {/* Heatmap Modal Overlay */}
        {heatmapOpen && (
          <div className="absolute inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-start justify-center pt-20">
             <div className="bg-[#121214] border border-glass-border rounded-lg shadow-2xl w-[800px] max-w-[90vw] animate-in fade-in zoom-in-95 duration-200">
                <div className="flex items-center justify-between px-4 py-3 border-b border-glass-border">
                   <h3 className="text-lg font-semibold text-neon-blue flex items-center gap-2">
                     <Calendar className="w-5 h-5" />
                     活动热力图
                   </h3>
                   <button 
                     onClick={() => setHeatmapOpen(false)}
                     className="text-gray-400 hover:text-white transition-colors"
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
