import { useState, useCallback } from 'react'
import Layout from './components/Layout'
import SettingsModal from './components/SettingsModal'
import AgentProposalModal from './components/AgentProposalModal'
import ChatHistoryModal from './components/ChatHistoryModal'
import FeedbackModal from './components/FeedbackModal'
import PerformanceModal from './components/PerformanceModal'
import { AppProvider } from './contexts/AppContext'

function App() {
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [agentProposalOpen, setAgentProposalOpen] = useState(false)
  const [chatHistoryOpen, setChatHistoryOpen] = useState(false)
  const [feedbackOpen, setFeedbackOpen] = useState(false)
  const [performanceOpen, setPerformanceOpen] = useState(false)

  // 对话会话状态
  const [currentSessionId, setCurrentSessionId] = useState<number | null>(null)
  const [shouldSwitchToQA, setShouldSwitchToQA] = useState(false)
  const [qaDraft, setQaDraft] = useState<string | null>(null)

  // 从历史继续对话
  const handleContinueChat = useCallback((sessionId: number) => {
    setCurrentSessionId(sessionId)
    setShouldSwitchToQA(true)
  }, [])

  // 重置切换标记
  const handleViewSwitched = useCallback(() => {
    setShouldSwitchToQA(false)
  }, [])

  // 新会话创建后的回调
  const handleSessionCreated = useCallback((sessionId: number) => {
    setCurrentSessionId(sessionId)
  }, [])

  // 开始新对话（清除当前会话）
  const handleStartNewChat = useCallback(() => {
    setCurrentSessionId(null)
  }, [])

  const handleSendToQA = useCallback((text: string) => {
    setQaDraft(text)
    setCurrentSessionId(null)
    setShouldSwitchToQA(true)
    setAgentProposalOpen(false)
  }, [])

  return (
    <AppProvider>
      <Layout
        onOpenSettings={() => setSettingsOpen(true)}
        onOpenAgentProposal={() => setAgentProposalOpen(true)}
        onOpenChatHistory={() => setChatHistoryOpen(true)}
        onOpenFeedback={() => setFeedbackOpen(true)}
        onOpenPerformance={() => setPerformanceOpen(true)}
        currentSessionId={currentSessionId}
        shouldSwitchToQA={shouldSwitchToQA}
        onViewSwitched={handleViewSwitched}
        onSessionCreated={handleSessionCreated}
        onStartNewChat={handleStartNewChat}
        qaDraft={qaDraft}
      />
      
      <SettingsModal 
        open={settingsOpen} 
        onClose={() => setSettingsOpen(false)} 
      />
      <AgentProposalModal 
        open={agentProposalOpen} 
        onClose={() => setAgentProposalOpen(false)} 
        onSendToQA={handleSendToQA}
      />
      <ChatHistoryModal 
        open={chatHistoryOpen} 
        onClose={() => setChatHistoryOpen(false)}
        onContinueChat={handleContinueChat}
      />
      <FeedbackModal 
        open={feedbackOpen} 
        onClose={() => setFeedbackOpen(false)}
        currentSessionId={currentSessionId}
      />
      <PerformanceModal 
        open={performanceOpen} 
        onClose={() => setPerformanceOpen(false)} 
      />
    </AppProvider>
  )
}

export default App
