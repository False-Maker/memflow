import { useState, useEffect } from 'react'
import Timeline from './Timeline'
import KnowledgeGraph from './KnowledgeGraph'
import FlowState from './FlowState'
import GalleryView from './GalleryView'
import ActivityHeatmap from './ActivityHeatmap'
import ContextSidebar from './ContextSidebar'
import ImmersiveReplay from './ImmersiveReplay'
import { useApp } from '../contexts/AppContext'
import { invoke } from '@tauri-apps/api/core'
import { Settings, BarChart3, Calendar, X } from 'lucide-react'

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

interface LayoutProps {
  onOpenSettings: () => void
  onOpenPerformance: () => void
}

export default function Layout({
  onOpenSettings,
  onOpenPerformance,
}: LayoutProps) {
  const { state, dispatch, startRecording, stopRecording } = useApp()
  const [heatmapOpen, setHeatmapOpen] = useState(false)
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
          // Set fallback system status to prevent black screen
          setSystemStatus({
            recording: state.isRecording,
            ocrServiceRunning: false,
            lastActivity: undefined,
            dbSizeBytes: 0,
            screenshotsSizeBytes: 0,
            ocrQueue: {
              pending: 0,
              processing: 0,
              done: 0,
              failed: 0
            },
            mcpStatus: 'error'
          })
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

const setCurrentView = (view: 'dashboard' | 'timeline' | 'graph' | 'stats' | 'gallery' | 'replay') => {
    dispatch({ type: 'SET_VIEW', payload: view })
  }

  const currentView = state.currentView as string
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
            { id: 'dashboard', label: 'DASHBOARD' },
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
            onClick={onOpenPerformance}
            className="p-2 text-zinc-500 hover:text-signal hover:bg-zinc-900 transition-all rounded-sm"
            title="Performance"
          >
            <BarChart3 className="w-4 h-4" />
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
            {currentView === 'dashboard' && (
              <div className="h-full overflow-y-auto p-6">
                <div className="glass border-b border-glass-border px-6 py-4 mb-6">
                  <h2 className="text-lg font-semibold text-neon-green">系统概览</h2>
                </div>
                {statusLoading && (
                  <div className="text-zinc-500">加载中...</div>
                )}
                {statusError && (
                  <div className="text-red-400">状态获取失败: {statusError}</div>
                )}
                {systemStatus && (
                  <>
                    <div className="grid grid-cols-3 gap-4 mb-6">
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
                        <div className="text-xl font-semibold text-zinc-200">
                          {systemStatus.mcpStatus}
                        </div>
                      </div>
                    </div>

                    <div className="glass p-4 rounded-lg mb-6">
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

                    <div className="grid grid-cols-2 gap-4 mb-6">
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
                    </div>

                    <div className="glass p-4 rounded-lg">
                      <div className="text-sm text-gray-400 mb-2">OCR 队列</div>
                      <div className="grid grid-cols-4 gap-4">
                        <div>
                          <div className="text-xs text-zinc-500">Pending</div>
                          <div className="text-lg font-semibold text-white">{systemStatus.ocrQueue.pending}</div>
                        </div>
                        <div>
                          <div className="text-xs text-zinc-500">Processing</div>
                          <div className="text-lg font-semibold text-white">{systemStatus.ocrQueue.processing}</div>
                        </div>
                        <div>
                          <div className="text-xs text-zinc-500">Done</div>
                          <div className="text-lg font-semibold text-white">{systemStatus.ocrQueue.done}</div>
                        </div>
                        <div>
                          <div className="text-xs text-zinc-500">Failed</div>
                          <div className="text-lg font-semibold text-white">{systemStatus.ocrQueue.failed}</div>
                        </div>
                      </div>
                    </div>
                  </>
                )}
              </div>
            )}
          {currentView === 'timeline' && <Timeline />}
            {currentView === 'gallery' && <GalleryView />}
            {currentView === 'replay' && <ImmersiveReplay />}
            {currentView === 'graph' && <KnowledgeGraph />}
            {currentView === 'stats' && <FlowState />}
          </div>
          {currentView !== 'dashboard' && <ContextSidebar />}
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
