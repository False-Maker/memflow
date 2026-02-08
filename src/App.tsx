import { useState, useCallback, useEffect, useMemo } from 'react'
import Layout from './components/Layout'
import SettingsModal from './components/SettingsModal'
import ChatHistoryModal from './components/ChatHistoryModal'
import FeedbackModal from './components/FeedbackModal'
import PerformanceModal from './components/PerformanceModal'
import { AppProvider } from './contexts/AppContext'
import { invoke } from '@tauri-apps/api/core'

interface SystemStatus {
  recording: boolean
  ocrServiceRunning: boolean
  lastActivity?: {
    timestamp: number
    appName: string
    windowTitle: string
  }
  dbSizeBytes: number
  screenshotsSizeBytes: number
  ocrQueue: {
    pending: number
    processing: number
    done: number
    failed: number
  }
  mcpStatus: string
}

function DebugPanel() {
  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null)
  const [statusLoading, setStatusLoading] = useState(true)
  const [statusError, setStatusError] = useState<string | null>(null)

  useEffect(() => {
    let mounted = true
    let initial = true
    const fetchStatus = async () => {
      if (initial) {
        setStatusLoading(true)
      }
      try {
        const data = await invoke<SystemStatus>('get_system_status')
        if (mounted) {
          setSystemStatus(data)
          setStatusError(null)
        }
      } catch (error) {
        if (mounted) {
          const message = error instanceof Error ? error.message : String(error)
          setStatusError(message)
        }
      } finally {
        if (mounted) {
          setStatusLoading(false)
        }
        initial = false
      }
    }
    fetchStatus()
    const interval = setInterval(fetchStatus, 5000)
    return () => {
      mounted = false
      clearInterval(interval)
    }
  }, [])

  const formatBytes = (bytes: number) => {
    if (!Number.isFinite(bytes)) return '0 MB'
    const mb = bytes / 1024 / 1024
    if (mb < 1024) {
      return `${mb.toFixed(1)} MB`
    }
    const gb = mb / 1024
    return `${gb.toFixed(2)} GB`
  }

  const formatTimestamp = (ts: number) => {
    const ms = ts < 1e12 ? ts * 1000 : ts
    return new Date(ms).toLocaleString()
  }

  const handleToggleRecording = async () => {
    if (systemStatus?.recording) {
      await invoke('stop_recording')
    } else {
      await invoke('start_recording')
    }
    const data = await invoke<SystemStatus>('get_system_status')
    setSystemStatus(data)
  }

  return (
    <div className="min-h-screen bg-void text-zinc-100 font-sans">
      <div className="max-w-3xl mx-auto p-6 space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <div className="text-xs font-mono text-zinc-400">MEMFLOW</div>
            <h1 className="text-xl font-semibold">调试面板</h1>
          </div>
          <button
            onClick={() => window.location.replace('index.html')}
            className="px-3 py-1.5 text-xs font-mono uppercase border border-zinc-700 text-zinc-300 hover:text-white hover:border-signal transition-colors"
          >
            打开完整界面
          </button>
        </div>

        {statusLoading && <div className="text-zinc-500">加载中...</div>}
        {statusError && <div className="text-red-400">状态获取失败: {statusError}</div>}
        {systemStatus && (
          <>
            <div className="grid grid-cols-2 gap-4">
              <div className="glass p-4 rounded-lg">
                <div className="text-sm text-gray-400 mb-1">录制状态</div>
                <div className={`text-xl font-semibold ${systemStatus.recording ? 'text-signal' : 'text-zinc-400'}`}>
                  {systemStatus.recording ? '录制中' : '未录制'}
                </div>
              </div>
              <div className="glass p-4 rounded-lg">
                <div className="text-sm text-gray-400 mb-1">OCR 服务</div>
                <div className={`text-xl font-semibold ${systemStatus.ocrServiceRunning ? 'text-neon-green' : 'text-zinc-400'}`}>
                  {systemStatus.ocrServiceRunning ? '运行中' : '未运行'}
                </div>
              </div>
              <div className="glass p-4 rounded-lg">
                <div className="text-sm text-gray-400 mb-1">MCP 状态</div>
                <div className="text-xl font-semibold text-white">
                  {systemStatus.mcpStatus}
                </div>
              </div>
              <div className="glass p-4 rounded-lg">
                <div className="text-sm text-gray-400 mb-1">数据库大小</div>
                <div className="text-xl font-semibold text-white">
                  {formatBytes(systemStatus.dbSizeBytes)}
                </div>
              </div>
              <div className="glass p-4 rounded-lg">
                <div className="text-sm text-gray-400 mb-1">截图占用</div>
                <div className="text-xl font-semibold text-white">
                  {formatBytes(systemStatus.screenshotsSizeBytes)}
                </div>
              </div>
              <div className="glass p-4 rounded-lg">
                <div className="text-sm text-gray-400 mb-1">OCR 队列</div>
                <div className="grid grid-cols-4 gap-2 text-sm">
                  <div>
                    <div className="text-zinc-500">Pending</div>
                    <div className="text-white">{systemStatus.ocrQueue.pending}</div>
                  </div>
                  <div>
                    <div className="text-zinc-500">Processing</div>
                    <div className="text-white">{systemStatus.ocrQueue.processing}</div>
                  </div>
                  <div>
                    <div className="text-zinc-500">Done</div>
                    <div className="text-white">{systemStatus.ocrQueue.done}</div>
                  </div>
                  <div>
                    <div className="text-zinc-500">Failed</div>
                    <div className="text-white">{systemStatus.ocrQueue.failed}</div>
                  </div>
                </div>
              </div>
            </div>

            <div className="glass p-4 rounded-lg">
              <div className="text-sm text-gray-400 mb-2">最近活动</div>
              {systemStatus.lastActivity ? (
                <div className="space-y-1">
                  <div className="text-white font-semibold">{systemStatus.lastActivity.appName}</div>
                  <div className="text-sm text-zinc-400 truncate">{systemStatus.lastActivity.windowTitle}</div>
                  <div className="text-xs text-zinc-500">{formatTimestamp(systemStatus.lastActivity.timestamp)}</div>
                </div>
              ) : (
                <div className="text-zinc-500">暂无活动记录</div>
              )}
            </div>
          </>
        )}

        <div className="flex items-center gap-3">
          <button
            onClick={handleToggleRecording}
            className={`px-4 py-2 text-xs font-mono uppercase border ${systemStatus?.recording
              ? 'bg-signal text-black border-signal hover:bg-amber-400'
              : 'bg-transparent text-zinc-400 border-zinc-700 hover:border-zinc-500 hover:text-zinc-200'
              }`}
          >
            {systemStatus?.recording ? '停止录制' : '开始录制'}
          </button>
          <div className="text-xs text-zinc-500">通过托盘菜单可进入完整界面</div>
        </div>
      </div>
    </div>
  )
}

function App() {
  const isDebugView = useMemo(() => {
    const params = new URLSearchParams(window.location.search)
    return params.get('debug') === '1'
  }, [])

  const [settingsOpen, setSettingsOpen] = useState(false)
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
  }, [])

  if (isDebugView) {
    return <DebugPanel />
  }

  return (
    <AppProvider>
      <Layout
        onOpenSettings={() => setSettingsOpen(true)}
        onOpenChatHistory={() => setChatHistoryOpen(true)}
        onOpenFeedback={() => setFeedbackOpen(true)}
        onOpenPerformance={() => setPerformanceOpen(true)}
        currentSessionId={currentSessionId}
        shouldSwitchToQA={shouldSwitchToQA}
        onViewSwitched={handleViewSwitched}
        onSessionCreated={handleSessionCreated}
        onStartNewChat={handleStartNewChat}
        qaDraft={qaDraft}
        onSendToQA={handleSendToQA}
      />

      <SettingsModal
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
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
