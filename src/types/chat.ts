/**
 * 对话历史与反馈系统 - 类型定义
 */

// ============================================
// 对话会话类型
// ============================================

export interface ChatSession {
  id: number
  title: string
  createdAt: number
  updatedAt: number
  messageCount: number
}

// ============================================
// 对话消息类型
// ============================================

export interface ChatMessage {
  id: number
  sessionId: number
  role: 'user' | 'assistant'
  content: string
  contextIds?: number[]
  createdAt: number
  rating?: 1 | -1 | null
}

// 前端临时消息类型（用于发送前）
export interface LocalChatMessage {
  localId: string
  role: 'user' | 'assistant'
  content: string
  ts: number
  dbId?: number  // 持久化后填充
  rating?: 1 | -1 | null
}

// ============================================
// 用户反馈类型
// ============================================

export type FeedbackCategory = 'bug' | 'feature' | 'experience' | 'other'
export type FeedbackStatus = 'pending' | 'noted' | 'resolved'

export interface UserFeedback {
  id: number
  category: FeedbackCategory
  title: string
  content: string
  screenshotPath?: string
  contextSessionId?: number
  status: FeedbackStatus
  createdAt: number
}

// 反馈提交表单
export interface FeedbackFormData {
  category: FeedbackCategory
  title: string
  content: string
  screenshotPath?: string
  contextSessionId?: number
}

// ============================================
// 反馈分类配置
// ============================================

export const FEEDBACK_CATEGORIES: Record<FeedbackCategory, { icon: string; label: string; color: string }> = {
  bug: { icon: '🐛', label: 'Bug', color: 'text-red-400' },
  feature: { icon: '💡', label: '功能建议', color: 'text-yellow-400' },
  experience: { icon: '🎨', label: '体验问题', color: 'text-purple-400' },
  other: { icon: '💬', label: '其他', color: 'text-blue-400' },
}

export const FEEDBACK_STATUS_LABELS: Record<FeedbackStatus, { label: string; color: string }> = {
  pending: { label: '待处理', color: 'text-yellow-400 bg-yellow-400/10' },
  noted: { label: '已记录', color: 'text-blue-400 bg-blue-400/10' },
  resolved: { label: '已解决', color: 'text-green-400 bg-green-400/10' },
}







