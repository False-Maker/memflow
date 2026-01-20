import { useEffect, useMemo, useRef, useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Send, Loader2, Sparkles, RotateCcw } from 'lucide-react'
import MessageRating from './MessageRating'
import type { LocalChatMessage, ChatMessage } from '../types/chat'

function makeLocalId() {
  return `${Date.now()}-${Math.random().toString(16).slice(2)}`
}

interface QnAProps {
  // 从历史继续对话时传入
  initialSessionId?: number | null
  onSessionCreated?: (sessionId: number) => void
  draft?: string | null
}

export default function QnA({ initialSessionId, onSessionCreated, draft }: QnAProps = {}) {
  const [input, setInput] = useState('')
  const [messages, setMessages] = useState<LocalChatMessage[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [sessionId, setSessionId] = useState<number | null>(initialSessionId ?? null)
  const [isLoadingHistory, setIsLoadingHistory] = useState(false)

  const listRef = useRef<HTMLDivElement | null>(null)
  const canSend = useMemo(() => input.trim().length > 0 && !loading, [input, loading])
  
  // 用于跟踪是否已初始化，避免重复加载
  const isInitializedRef = useRef(false)
  const lastLoadedSessionRef = useRef<number | null>(null)

  // 加载历史消息
  const loadHistoryMessages = useCallback(async (sid: number) => {
    setIsLoadingHistory(true)
    try {
      const dbMessages = await invoke<ChatMessage[]>('get_chat_messages', { sessionId: sid })
      const localMessages: LocalChatMessage[] = dbMessages.map((m) => ({
        localId: makeLocalId(),
        role: m.role as 'user' | 'assistant',
        content: m.content,
        ts: m.createdAt,
        dbId: m.id,
        rating: m.rating,
      }))
      setMessages(localMessages)
    } catch (e) {
      console.error('加载历史消息失败:', e)
    } finally {
      setIsLoadingHistory(false)
    }
  }, [])

  // 初始化：加载历史或显示欢迎消息
  useEffect(() => {
    // 如果已初始化且不是从历史继续对话，不重新加载
    if (isInitializedRef.current && !initialSessionId) {
      return
    }
    
    // 如果是从历史继续对话（外部传入的 sessionId）
    // 注意：创建新会话时父组件可能会把 sessionId 回传进来，如果这里直接 reload 会覆盖刚刚追加的本地消息气泡
    if (initialSessionId && initialSessionId !== lastLoadedSessionRef.current) {
      setSessionId(initialSessionId)
      loadHistoryMessages(initialSessionId)
      lastLoadedSessionRef.current = initialSessionId
      isInitializedRef.current = true
    } else if (!initialSessionId && !isInitializedRef.current) {
      // 首次加载，显示欢迎消息
      setMessages([
        {
          localId: makeLocalId(),
          role: 'assistant',
          content:
            '你可以在这里提问：我会结合你的桌面活动记录（OCR 文本）做检索并回答。\n\n提示：如果还没在"设置"里配置 OpenAI/Anthropic API Key，会返回占位回答。',
          ts: Date.now(),
        },
      ])
      isInitializedRef.current = true
    }
  }, [initialSessionId, loadHistoryMessages])

  // 自动滚动到底部
  useEffect(() => {
    const el = listRef.current
    if (!el) return
    el.scrollTop = el.scrollHeight
  }, [messages.length, loading])

  useEffect(() => {
    if (!draft) return
    setInput(draft)
  }, [draft])

  // 创建新会话
  const createSession = async (firstQuestion: string): Promise<number> => {
    const title = firstQuestion.slice(0, 50) + (firstQuestion.length > 50 ? '...' : '')
    const newSessionId = await invoke<number>('create_chat_session', { title })
    setSessionId(newSessionId)
    // 关键：标记为“已加载该会话”，避免父组件回传 initialSessionId 触发 loadHistoryMessages 覆盖本地消息
    lastLoadedSessionRef.current = newSessionId
    isInitializedRef.current = true
    onSessionCreated?.(newSessionId)
    return newSessionId
  }

  // 保存消息到数据库
  const saveMessage = async (
    sid: number,
    role: 'user' | 'assistant',
    content: string
  ): Promise<number> => {
    const messageId = await invoke<number>('save_chat_message', {
      sessionId: sid,
      role,
      content,
      contextIds: null,
    })
    return messageId
  }

  // 开始新对话
  const startNewConversation = () => {
    setSessionId(null)
    lastLoadedSessionRef.current = null
    setMessages([
      {
        localId: makeLocalId(),
        role: 'assistant',
        content:
          '你可以在这里提问：我会结合你的桌面活动记录（OCR 文本）做检索并回答。\n\n提示：如果还没在"设置"里配置 OpenAI/Anthropic API Key，会返回占位回答。',
        ts: Date.now(),
      },
    ])
    setError(null)
  }

  const send = async () => {
    const q = input.trim()
    if (!q || loading) return

    setError(null)
    setLoading(true)
    setInput('')

    // 1. 添加用户消息到界面
    const userMsg: LocalChatMessage = {
      localId: makeLocalId(),
      role: 'user',
      content: q,
      ts: Date.now(),
    }
    setMessages((prev) => [...prev, userMsg])

    let currentSessionId = sessionId
    // 助手消息占位符ID
    const botLocalId = makeLocalId()

    try {
      // 2. 如果没有会话，创建新会话
      if (!currentSessionId) {
        currentSessionId = await createSession(q)
      }

      // 3. 保存用户消息到数据库
      const userMsgId = await saveMessage(currentSessionId, 'user', q)
      setMessages((prev) =>
        prev.map((m) => (m.localId === userMsg.localId ? { ...m, dbId: userMsgId } : m))
      )

      // 4. 准备助手消息占位符
      // 注意：流式输出时，我们先显示一个空的或者正在思考的消息，然后随着 chunk 到来不断更新它
      const initialBotMsg: LocalChatMessage = {
        localId: botLocalId,
        role: 'assistant',
        content: '', // 初始为空
        ts: Date.now(),
      }
      setMessages((prev) => [...prev, initialBotMsg])

      // 5. 设置流式监听
      let accumulatedResponse = ''
      
      // 监听 chunk 事件
      const unlisten = await listen<string>('ai-chat-chunk', (event) => {
        const chunk = event.payload
        accumulatedResponse += chunk
        
        // 实时更新 UI
        setMessages((prev) => 
          prev.map((m) => 
            m.localId === botLocalId 
              ? { ...m, content: accumulatedResponse } 
              : m
          )
        )
      })

      try {
        // 6. 调用流式命令
        // 这个命令会等待直到流结束才返回
        await invoke('ai_chat_stream', { query: q })
      } finally {
        // 确保取消监听
        unlisten()
      }

      // 7. 流式结束，保存完整消息到数据库
      if (accumulatedResponse.trim()) {
        const botMsgId = await saveMessage(currentSessionId, 'assistant', accumulatedResponse)
        
        // 更新消息的 dbId
        setMessages((prev) =>
          prev.map((m) =>
            m.localId === botLocalId
              ? { ...m, dbId: botMsgId } // 更新 dbId，允许显示评价按钮等
              : m
          )
        )
      } else {
        // 极端情况：没有任何响应
        setError("AI 未返回任何内容")
      }

    } catch (e) {
      const msg = typeof e === 'string' ? e : JSON.stringify(e)
      console.error('AI Chat Error:', e)
      setError(msg)
      
      // 如果出错，更新当前正在生成的消息显示错误信息
      setMessages((prev) => 
        prev.map((m) => 
          m.localId === botLocalId 
            ? { ...m, content: m.content + `\n\n[出错了: ${msg}]` } 
            : m
        )
      )
    } finally {
      setLoading(false)
    }
  }

  // 更新消息评价
  const handleRatingChange = (localId: string, rating: 1 | -1 | null) => {
    setMessages((prev) =>
      prev.map((m) => (m.localId === localId ? { ...m, rating } : m))
    )
  }

  return (
    <div className="h-full flex flex-col min-h-0">
      <div className="glass border-b border-glass-border px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Sparkles className="w-5 h-5 text-neon-blue" />
          <h2 className="text-lg font-semibold text-neon-blue">问答</h2>
          {sessionId && (
            <span className="text-xs text-gray-500 ml-2">
              会话 #{sessionId}
            </span>
          )}
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={startNewConversation}
            className="flex items-center gap-1 px-3 py-1.5 text-sm text-gray-400 hover:text-white hover:bg-surface/50 rounded-lg transition-colors"
            title="开始新对话"
          >
            <RotateCcw className="w-4 h-4" />
            <span>新对话</span>
          </button>
          <div className="text-xs text-gray-500">
            基于 OCR / RAG 检索（本地优先）
          </div>
        </div>
      </div>

      <div ref={listRef} className="flex-1 min-h-0 overflow-y-auto px-6 py-4 space-y-3">
        {isLoadingHistory ? (
          <div className="flex justify-center py-8" role="status" aria-label="加载中">
            <Loader2 className="w-6 h-6 animate-spin text-neon-blue" />
          </div>
        ) : (
          messages.map((m) => (
            <div
              key={m.localId}
              className={`flex ${m.role === 'user' ? 'justify-end' : 'justify-start'}`}
            >
              <div
                className={`max-w-[85%] rounded-2xl px-4 py-3 border ${
                  m.role === 'user'
                    ? 'bg-neon-blue/10 border-neon-blue/20 text-white'
                    : 'bg-surface/50 border-glass-border/50 text-gray-100'
                }`}
              >
                <pre className="whitespace-pre-wrap font-sans text-sm leading-relaxed">
                  {m.content || (m.role === 'assistant' ? '（回复内容为空）' : '（消息内容为空）')}
                </pre>
                {/* 助手消息显示评价按钮 */}
                {m.role === 'assistant' && m.dbId && (
                  <MessageRating
                    messageId={m.dbId}
                    currentRating={m.rating}
                    onRatingChange={(rating) => handleRatingChange(m.localId, rating)}
                  />
                )}
              </div>
            </div>
          ))
        )}

        {loading && (
          <div className="flex justify-start">
            <div className="max-w-[85%] rounded-2xl px-4 py-3 border bg-surface/50 border-glass-border/50 text-gray-100 flex items-center gap-2">
              <Loader2 className="w-4 h-4 animate-spin text-neon-blue" />
              <span className="text-sm text-gray-300">正在思考…</span>
            </div>
          </div>
        )}
      </div>

      <div className="glass border-t border-glass-border px-6 py-4 flex-shrink-0">
        {error && (
          <div className="mb-3 text-sm text-neon-red/90">
            {error}
          </div>
        )}

        <div className="flex items-end gap-3">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault()
                void send()
              }
            }}
            placeholder="输入你的问题（Enter 发送，Shift+Enter 换行）"
            className="flex-1 min-h-[44px] max-h-40 resize-none px-4 py-2.5 bg-surface border border-glass-border rounded-lg text-white placeholder:text-gray-500 hover:border-neon-blue/30 transition-colors focus:outline-none focus:ring-2 focus:ring-neon-blue/30"
          />
          <button
            onClick={() => void send()}
            disabled={!canSend}
            className="px-4 py-2.5 rounded-lg bg-neon-blue/20 text-neon-blue hover:bg-neon-blue/30 transition-colors flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Send className="w-4 h-4" />
            <span>发送</span>
          </button>
        </div>
      </div>
    </div>
  )
}
