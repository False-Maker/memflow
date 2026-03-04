import { useEffect, useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  X,
  Search,
  Trash2,
  MessageSquare,
  ChevronLeft,
  ArrowRight,
  AlertTriangle,
  Loader2,
} from 'lucide-react'
import type { ChatSession, ChatMessage } from '../types/chat'

interface ChatHistoryModalProps {
  open: boolean
  onClose: () => void
  onContinueChat?: (sessionId: number) => void
}

type ViewMode = 'list' | 'detail'

export default function ChatHistoryModal({
  open,
  onClose,
  onContinueChat,
}: ChatHistoryModalProps) {
  const [viewMode, setViewMode] = useState<ViewMode>('list')
  const [sessions, setSessions] = useState<ChatSession[]>([])
  const [selectedSession, setSelectedSession] = useState<ChatSession | null>(null)
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [loading, setLoading] = useState(false)
  const [loadingMessages, setLoadingMessages] = useState(false)
  const [showClearConfirm, setShowClearConfirm] = useState(false)
  const [deletingId, setDeletingId] = useState<number | null>(null)

  // 加载会话列表
  const loadSessions = useCallback(async (search?: string) => {
    setLoading(true)
    try {
      const result = await invoke<ChatSession[]>('get_chat_sessions', {
        limit: 100,
        offset: 0,
        search: search || null,
      })
      setSessions(result)
    } catch (e) {
      console.error('加载会话列表失败:', e)
    } finally {
      setLoading(false)
    }
  }, [])

  // 加载会话消息
  const loadMessages = useCallback(async (sessionId: number) => {
    setLoadingMessages(true)
    try {
      const result = await invoke<ChatMessage[]>('get_chat_messages', { sessionId })
      setMessages(result)
    } catch (e) {
      console.error('加载消息失败:', e)
    } finally {
      setLoadingMessages(false)
    }
  }, [])

  // 初始化加载
  useEffect(() => {
    if (open) {
      loadSessions()
      setViewMode('list')
      setSelectedSession(null)
      setSearchQuery('')
    }
  }, [open, loadSessions])

  // 搜索防抖
  useEffect(() => {
    const timer = setTimeout(() => {
      if (open && viewMode === 'list') {
        loadSessions(searchQuery)
      }
    }, 300)
    return () => clearTimeout(timer)
  }, [searchQuery, open, viewMode, loadSessions])

  // 查看会话详情
  const handleViewDetail = (session: ChatSession) => {
    setSelectedSession(session)
    setViewMode('detail')
    loadMessages(session.id)
  }

  // 返回列表
  const handleBackToList = () => {
    setViewMode('list')
    setSelectedSession(null)
    setMessages([])
  }

  // 删除单个会话
  const handleDelete = async (sessionId: number, e?: React.MouseEvent) => {
    e?.stopPropagation()
    setDeletingId(sessionId)
    try {
      await invoke('delete_chat_session', { sessionId })
      setSessions((prev) => prev.filter((s) => s.id !== sessionId))
      if (selectedSession?.id === sessionId) {
        handleBackToList()
      }
    } catch (e) {
      console.error('删除会话失败:', e)
    } finally {
      setDeletingId(null)
    }
  }

  // 清空所有历史
  const handleClearAll = async () => {
    try {
      await invoke('clear_all_chat_history')
      setSessions([])
      setShowClearConfirm(false)
      handleBackToList()
    } catch (e) {
      console.error('清空历史失败:', e)
    }
  }

  // 继续对话
  const handleContinue = () => {
    if (selectedSession && onContinueChat) {
      onContinueChat(selectedSession.id)
      onClose()
    }
  }

  // 格式化时间
  // 将时间戳转换为毫秒（后端返回的可能是秒级或毫秒级时间戳）
  const toMs = (ts: number) => (ts < 1e12 ? ts * 1000 : ts)

  const formatTime = (timestamp: number) => {
    const date = new Date(toMs(timestamp))
    const now = new Date()
    const isToday = date.toDateString() === now.toDateString()

    if (isToday) {
      return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', timeZone: 'Asia/Shanghai' })
    }
    return date.toLocaleDateString('zh-CN', {
      timeZone: 'Asia/Shanghai',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* 背景遮罩 */}
      <div className="absolute inset-0 bg-black/70 backdrop-blur-md" onClick={onClose} />
      
      {/* 弹窗主体 */}
      <div className="relative w-full max-w-3xl max-h-[80vh] bg-black/90 backdrop-blur-xl border border-white/10 rounded-xl overflow-hidden shadow-2xl animate-in zoom-in-95 duration-200">
        {/* 斜切装饰 */}
        <div className="absolute top-0 right-0 w-24 h-24 pointer-events-none">
          <div className="absolute top-0 right-0 w-full h-full bg-gradient-to-bl from-purple-500/10 to-transparent transform skew-x-12 translate-x-8 -translate-y-4" />
        </div>
        
        {/* 顶部装饰线 */}
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-purple-500/50 to-transparent" />
        
        {/* 头部 */}
        <div className="flex items-center justify-between p-4 border-b border-white/5">
          {viewMode === 'detail' && selectedSession ? (
            <>
              <div className="flex items-center gap-3">
                <button
                  onClick={handleBackToList}
                  className="p-1.5 rounded-lg hover:bg-white/5 transition-colors"
                >
                  <ChevronLeft className="w-5 h-5" />
                </button>
                <h2 className="text-lg font-semibold text-white truncate max-w-md">
                  {selectedSession.title}
                </h2>
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={handleContinue}
                  className="flex items-center gap-2 px-4 py-2 bg-purple-500/20 text-purple-400 rounded-lg hover:bg-purple-500/30 transition-colors"
                >
                  <span>继续对话</span>
                  <ArrowRight className="w-4 h-4" />
                </button>
                <button
                  onClick={onClose}
                  className="p-2 rounded-lg hover:bg-white/5 transition-colors"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>
            </>
          ) : (
            <>
              <h2 className="text-lg font-bold text-white flex items-center gap-3">
                <div className="w-8 h-8 rounded-lg bg-purple-500/20 text-purple-400 flex items-center justify-center">
                  <MessageSquare className="w-4 h-4" />
                </div>
                对话历史
              </h2>
              <button
                onClick={onClose}
                className="p-2 rounded-lg hover:bg-white/5 transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </>
          )}
        </div>

        {/* 内容区 */}
        {viewMode === 'list' ? (
          <>
            {/* 搜索栏 */}
            <div className="p-4 border-b border-glass-border">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="搜索对话..."
                  className="w-full pl-10 pr-4 py-2 bg-surface border border-glass-border rounded-lg text-white placeholder:text-gray-500 focus:outline-none focus:ring-2 focus:ring-neon-cyan/30"
                />
              </div>
            </div>

            {/* 会话列表 */}
            <div className="flex-1 overflow-y-auto p-4 space-y-2">
              {loading ? (
                <div className="flex justify-center py-8">
                  <Loader2 className="w-6 h-6 animate-spin text-neon-cyan" />
                </div>
              ) : sessions.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-12 text-gray-500">
                  <MessageSquare className="w-12 h-12 mb-4 opacity-50" />
                  <p>暂无对话历史</p>
                  <p className="text-sm mt-1">开始一段新对话吧</p>
                </div>
              ) : (
                sessions.map((session) => (
                  <div
                    key={session.id}
                    onClick={() => handleViewDetail(session)}
                    className="group flex items-center justify-between p-4 bg-surface/50 border border-glass-border/50 rounded-lg hover:border-neon-cyan/30 hover:bg-surface/80 transition-all cursor-pointer"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <MessageSquare className="w-4 h-4 text-neon-cyan flex-shrink-0" />
                        <span className="text-white truncate">{session.title}</span>
                      </div>
                      <div className="flex items-center gap-3 mt-1 text-xs text-gray-500">
                        <span>{formatTime(session.updatedAt)}</span>
                        <span>·</span>
                        <span>{session.messageCount} 条消息</span>
                      </div>
                    </div>
                    <button
                      onClick={(e) => handleDelete(session.id, e)}
                      disabled={deletingId === session.id}
                      className="p-2 rounded-lg text-gray-500 hover:text-red-400 hover:bg-red-400/10 opacity-0 group-hover:opacity-100 transition-all"
                    >
                      {deletingId === session.id ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <Trash2 className="w-4 h-4" />
                      )}
                    </button>
                  </div>
                ))
              )}
            </div>

            {/* 底部操作 */}
            <div className="flex items-center justify-between p-4 border-t border-glass-border">
              <button
                onClick={() => setShowClearConfirm(true)}
                disabled={sessions.length === 0}
                className="flex items-center gap-2 text-sm text-red-400/70 hover:text-red-400 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                <AlertTriangle className="w-4 h-4" />
                <span>清空所有历史</span>
              </button>
              <span className="text-sm text-gray-500">
                共 {sessions.length} 条对话
              </span>
            </div>
          </>
        ) : (
          /* 详情视图 */
          <div className="flex-1 overflow-y-auto p-4 space-y-3">
            {loadingMessages ? (
              <div className="flex justify-center py-8">
                <Loader2 className="w-6 h-6 animate-spin text-neon-cyan" />
              </div>
            ) : (
              messages.map((m) => (
                <div
                  key={m.id}
                  className={`flex ${m.role === 'user' ? 'justify-end' : 'justify-start'}`}
                >
                  <div className="max-w-[85%]">
                    <div className="flex items-center gap-2 mb-1">
                      <span className={`text-xs ${m.role === 'user' ? 'text-neon-cyan' : 'text-gray-400'}`}>
                        {m.role === 'user' ? '👤 用户' : '🤖 助手'}
                      </span>
                      <span className="text-xs text-gray-500">
                        {formatTime(m.createdAt)}
                      </span>
                    </div>
                    <div
                      className={`rounded-2xl px-4 py-3 border ${
                        m.role === 'user'
                          ? 'bg-neon-cyan/10 border-neon-cyan/20 text-white'
                          : 'bg-surface/50 border-glass-border/50 text-gray-100'
                      }`}
                    >
                      <pre className="whitespace-pre-wrap font-sans text-sm leading-relaxed">
                        {m.content}
                      </pre>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {/* 清空确认弹窗 */}
        {showClearConfirm && (
          <div className="absolute inset-0 flex items-center justify-center bg-black/50 rounded-lg">
            <div className="bg-surface border border-glass-border rounded-lg p-6 max-w-sm">
              <div className="flex items-center gap-3 mb-4">
                <AlertTriangle className="w-6 h-6 text-red-400" />
                <h3 className="text-lg font-semibold text-white">确认清空</h3>
              </div>
              <p className="text-gray-400 mb-6">
                确定要清空所有对话历史吗？此操作不可撤销。
              </p>
              <div className="flex justify-end gap-3">
                <button
                  onClick={() => setShowClearConfirm(false)}
                  className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
                >
                  取消
                </button>
                <button
                  onClick={handleClearAll}
                  className="px-4 py-2 bg-red-500/20 text-red-400 rounded-lg hover:bg-red-500/30 transition-colors"
                >
                  确认清空
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}




