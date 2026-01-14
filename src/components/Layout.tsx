import { useState, useEffect } from 'react'
import Timeline from './Timeline'
import KnowledgeGraph from './KnowledgeGraph'
import FlowState from './FlowState'
import QnA from './QnA'
import { useApp } from '../contexts/AppContext'
import { Play, Pause, Settings, Zap, History, MessageSquare, BarChart3 } from 'lucide-react'

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
  const { state, startRecording, stopRecording } = useApp()
  const [currentView, setCurrentView] = useState<'timeline' | 'graph' | 'stats' | 'qa'>('timeline')

  // 当需要切换到问答视图时自动切换
  useEffect(() => {
    if (shouldSwitchToQA) {
      setCurrentView('qa')
      onViewSwitched?.()
    }
  }, [shouldSwitchToQA, onViewSwitched])

  return (
    <div className="flex flex-col h-screen bg-void">
      {/* 顶部工具栏 */}
      <header className="glass border-b border-glass-border px-4 py-3 flex items-center justify-between">
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
      <main className="flex-1 overflow-hidden min-h-0">
        {currentView === 'timeline' && <Timeline />}
        {currentView === 'graph' && <KnowledgeGraph />}
        {currentView === 'stats' && <FlowState />}
        {currentView === 'qa' && (
          <QnA
            initialSessionId={currentSessionId}
            onSessionCreated={onSessionCreated}
            draft={qaDraft}
          />
        )}
      </main>
    </div>
  )
}
