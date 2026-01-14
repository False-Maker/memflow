import { useEffect, useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { X, Send, Loader2 } from 'lucide-react'
import type {
  UserFeedback,
  FeedbackCategory,
  FeedbackFormData,
} from '../types/chat'
import { FEEDBACK_CATEGORIES, FEEDBACK_STATUS_LABELS } from '../types/chat'

interface FeedbackModalProps {
  open: boolean
  onClose: () => void
  currentSessionId?: number | null  // 当前对话会话 ID
}

type TabType = 'submit' | 'list'

export default function FeedbackModal({
  open,
  onClose,
  currentSessionId,
}: FeedbackModalProps) {
  const [activeTab, setActiveTab] = useState<TabType>('submit')
  const [feedbacks, setFeedbacks] = useState<UserFeedback[]>([])
  const [loading, setLoading] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [submitSuccess, setSubmitSuccess] = useState(false)

  // 表单状态
  const [formData, setFormData] = useState<FeedbackFormData>({
    category: 'bug',
    title: '',
    content: '',
    contextSessionId: currentSessionId ?? undefined,
  })
  const [errors, setErrors] = useState<{ title?: string; content?: string }>({})

  // 加载反馈列表
  const loadFeedbacks = useCallback(async () => {
    setLoading(true)
    try {
      const result = await invoke<UserFeedback[]>('get_user_feedbacks', { limit: 50 })
      setFeedbacks(result)
    } catch (e) {
      console.error('加载反馈列表失败:', e)
    } finally {
      setLoading(false)
    }
  }, [])

  // 打开时加载数据
  useEffect(() => {
    if (open) {
      setSubmitSuccess(false)
      if (activeTab === 'list') {
        loadFeedbacks()
      }
      // 更新当前会话 ID
      setFormData((prev) => ({
        ...prev,
        contextSessionId: currentSessionId ?? undefined,
      }))
    }
  }, [open, activeTab, loadFeedbacks, currentSessionId])

  // 切换 Tab 时加载数据
  useEffect(() => {
    if (open && activeTab === 'list') {
      loadFeedbacks()
    }
  }, [activeTab, open, loadFeedbacks])

  // 表单验证
  const validateForm = (): boolean => {
    const newErrors: { title?: string; content?: string } = {}

    if (!formData.title.trim()) {
      newErrors.title = '请输入标题'
    } else if (formData.title.length > 100) {
      newErrors.title = '标题不能超过 100 个字符'
    }

    if (!formData.content.trim()) {
      newErrors.content = '请输入详细描述'
    } else if (formData.content.length < 10) {
      newErrors.content = '描述至少需要 10 个字符'
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  // 提交反馈
  const handleSubmit = async () => {
    if (!validateForm()) return

    setSubmitting(true)
    try {
      await invoke('submit_feedback', {
        category: formData.category,
        title: formData.title.trim(),
        content: formData.content.trim(),
        screenshotPath: formData.screenshotPath || null,
        contextSessionId: formData.contextSessionId || null,
      })

      setSubmitSuccess(true)
      // 重置表单
      setFormData({
        category: 'bug',
        title: '',
        content: '',
        contextSessionId: currentSessionId ?? undefined,
      })
      setErrors({})

      // 3 秒后清除成功状态
      setTimeout(() => setSubmitSuccess(false), 3000)
    } catch (e) {
      console.error('提交反馈失败:', e)
    } finally {
      setSubmitting(false)
    }
  }

  // 格式化时间
  // 将时间戳转换为毫秒（后端返回的可能是秒级或毫秒级时间戳）
  const toMs = (ts: number) => (ts < 1e12 ? ts * 1000 : ts)

  const formatTime = (timestamp: number) => {
    return new Date(toMs(timestamp)).toLocaleDateString('zh-CN', {
      timeZone: 'Asia/Shanghai',
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="glass w-full max-w-2xl max-h-[80vh] rounded-lg flex flex-col">
        {/* 头部 */}
        <div className="flex items-center justify-between p-4 border-b border-glass-border">
          <h2 className="text-xl font-bold text-white">反馈</h2>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-surface transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tab 切换 */}
        <div className="flex border-b border-glass-border">
          <button
            onClick={() => setActiveTab('submit')}
            className={`flex-1 py-3 text-center transition-colors ${
              activeTab === 'submit'
                ? 'text-neon-blue border-b-2 border-neon-blue'
                : 'text-gray-400 hover:text-white'
            }`}
          >
            提交反馈
          </button>
          <button
            onClick={() => setActiveTab('list')}
            className={`flex-1 py-3 text-center transition-colors ${
              activeTab === 'list'
                ? 'text-neon-blue border-b-2 border-neon-blue'
                : 'text-gray-400 hover:text-white'
            }`}
          >
            我的反馈
          </button>
        </div>

        {/* 内容区 */}
        <div className="flex-1 overflow-y-auto p-6">
          {activeTab === 'submit' ? (
            /* 提交反馈表单 */
            <div className="space-y-6">
              {submitSuccess && (
                <div className="p-4 bg-green-500/10 border border-green-500/30 rounded-lg text-green-400 text-sm">
                  ✓ 反馈提交成功，感谢您的反馈！
                </div>
              )}

              {/* 反馈类型 */}
              <div>
                <label className="block text-sm text-gray-400 mb-2">反馈类型</label>
                <div className="grid grid-cols-4 gap-2">
                  {(Object.keys(FEEDBACK_CATEGORIES) as FeedbackCategory[]).map((cat) => {
                    const config = FEEDBACK_CATEGORIES[cat]
                    return (
                      <button
                        key={cat}
                        onClick={() => setFormData((prev) => ({ ...prev, category: cat }))}
                        className={`flex flex-col items-center gap-1 p-3 rounded-lg border transition-all ${
                          formData.category === cat
                            ? 'border-neon-blue bg-neon-blue/10 text-neon-blue'
                            : 'border-glass-border bg-surface/50 text-gray-400 hover:border-neon-blue/50'
                        }`}
                      >
                        <span className="text-xl">{config.icon}</span>
                        <span className="text-xs">{config.label}</span>
                      </button>
                    )
                  })}
                </div>
              </div>

              {/* 标题 */}
              <div>
                <label className="block text-sm text-gray-400 mb-2">
                  标题 <span className="text-red-400">*</span>
                </label>
                <input
                  type="text"
                  value={formData.title}
                  onChange={(e) => {
                    setFormData((prev) => ({ ...prev, title: e.target.value }))
                    setErrors((prev) => ({ ...prev, title: undefined }))
                  }}
                  placeholder="简要描述你遇到的问题或建议..."
                  className={`w-full px-4 py-2.5 bg-surface border rounded-lg text-white placeholder:text-gray-500 focus:outline-none focus:ring-2 focus:ring-neon-blue/30 ${
                    errors.title ? 'border-red-400' : 'border-glass-border'
                  }`}
                />
                {errors.title && (
                  <p className="mt-1 text-xs text-red-400">{errors.title}</p>
                )}
              </div>

              {/* 详细描述 */}
              <div>
                <label className="block text-sm text-gray-400 mb-2">
                  详细描述 <span className="text-red-400">*</span>
                </label>
                <textarea
                  value={formData.content}
                  onChange={(e) => {
                    setFormData((prev) => ({ ...prev, content: e.target.value }))
                    setErrors((prev) => ({ ...prev, content: undefined }))
                  }}
                  placeholder={`请详细描述：\n- 问题的具体表现\n- 复现步骤（如果是 Bug）\n- 期望的行为`}
                  rows={6}
                  className={`w-full px-4 py-2.5 bg-surface border rounded-lg text-white placeholder:text-gray-500 focus:outline-none focus:ring-2 focus:ring-neon-blue/30 resize-none ${
                    errors.content ? 'border-red-400' : 'border-glass-border'
                  }`}
                />
                {errors.content && (
                  <p className="mt-1 text-xs text-red-400">{errors.content}</p>
                )}
              </div>

              {/* 关联会话提示 */}
              {formData.contextSessionId && (
                <div className="text-xs text-gray-500">
                  将关联当前对话会话 #{formData.contextSessionId}
                </div>
              )}
            </div>
          ) : (
            /* 我的反馈列表 */
            <div className="space-y-3">
              {loading ? (
                <div className="flex justify-center py-8">
                  <Loader2 className="w-6 h-6 animate-spin text-neon-blue" />
                </div>
              ) : feedbacks.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-12 text-gray-500">
                  <p>暂无反馈记录</p>
                  <p className="text-sm mt-1">提交一条反馈吧</p>
                </div>
              ) : (
                feedbacks.map((fb) => {
                  const categoryConfig = FEEDBACK_CATEGORIES[fb.category as FeedbackCategory]
                  const statusConfig = FEEDBACK_STATUS_LABELS[fb.status]
                  return (
                    <div
                      key={fb.id}
                      className="p-4 bg-surface/50 border border-glass-border/50 rounded-lg"
                    >
                      <div className="flex items-start justify-between gap-4">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            <span className={categoryConfig?.color || 'text-gray-400'}>
                              {categoryConfig?.icon} {categoryConfig?.label}
                            </span>
                            <span
                              className={`text-xs px-2 py-0.5 rounded ${statusConfig?.color || 'text-gray-400'}`}
                            >
                              {statusConfig?.label}
                            </span>
                          </div>
                          <h3 className="text-white font-medium truncate">{fb.title}</h3>
                          <p className="text-sm text-gray-400 mt-1 line-clamp-2">
                            {fb.content}
                          </p>
                          <div className="text-xs text-gray-500 mt-2">
                            {formatTime(fb.createdAt)}
                          </div>
                        </div>
                      </div>
                    </div>
                  )
                })
              )}
            </div>
          )}
        </div>

        {/* 底部操作 */}
        {activeTab === 'submit' && (
          <div className="flex justify-end gap-3 p-4 border-t border-glass-border">
            <button
              onClick={onClose}
              className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
            >
              取消
            </button>
            <button
              onClick={handleSubmit}
              disabled={submitting}
              className="flex items-center gap-2 px-6 py-2 bg-neon-blue/20 text-neon-blue rounded-lg hover:bg-neon-blue/30 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {submitting ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  <span>提交中...</span>
                </>
              ) : (
                <>
                  <Send className="w-4 h-4" />
                  <span>提交</span>
                </>
              )}
            </button>
          </div>
        )}

        {activeTab === 'list' && (
          <div className="flex justify-center p-4 border-t border-glass-border">
            <span className="text-sm text-gray-500">
              共 {feedbacks.length} 条反馈
            </span>
          </div>
        )}
      </div>
    </div>
  )
}
