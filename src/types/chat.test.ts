import { describe, it, expect } from 'vitest'
import { 
  FEEDBACK_CATEGORIES, 
  FEEDBACK_STATUS_LABELS,
  type FeedbackCategory,
  type FeedbackStatus,
  type ChatSession,
  type ChatMessage,
  type LocalChatMessage,
  type UserFeedback,
  type FeedbackFormData,
} from './chat'

describe('chat types', () => {
  describe('FEEDBACK_CATEGORIES', () => {
    it('应该包含所有反馈分类', () => {
      expect(FEEDBACK_CATEGORIES).toHaveProperty('bug')
      expect(FEEDBACK_CATEGORIES).toHaveProperty('feature')
      expect(FEEDBACK_CATEGORIES).toHaveProperty('experience')
      expect(FEEDBACK_CATEGORIES).toHaveProperty('other')
    })

    it('每个分类应该有正确的结构', () => {
      const categories: FeedbackCategory[] = ['bug', 'feature', 'experience', 'other']
      
      categories.forEach(category => {
        const config = FEEDBACK_CATEGORIES[category]
        expect(config).toHaveProperty('icon')
        expect(config).toHaveProperty('label')
        expect(config).toHaveProperty('color')
        expect(typeof config.icon).toBe('string')
        expect(typeof config.label).toBe('string')
        expect(typeof config.color).toBe('string')
      })
    })
  })

  describe('FEEDBACK_STATUS_LABELS', () => {
    it('应该包含所有状态标签', () => {
      expect(FEEDBACK_STATUS_LABELS).toHaveProperty('pending')
      expect(FEEDBACK_STATUS_LABELS).toHaveProperty('noted')
      expect(FEEDBACK_STATUS_LABELS).toHaveProperty('resolved')
    })

    it('每个状态应该有正确的结构', () => {
      const statuses: FeedbackStatus[] = ['pending', 'noted', 'resolved']
      
      statuses.forEach(status => {
        const config = FEEDBACK_STATUS_LABELS[status]
        expect(config).toHaveProperty('label')
        expect(config).toHaveProperty('color')
        expect(typeof config.label).toBe('string')
        expect(typeof config.color).toBe('string')
      })
    })
  })

  describe('类型定义', () => {
    it('ChatSession 应该有正确的结构', () => {
      const session: ChatSession = {
        id: 1,
        title: '测试会话',
        createdAt: Date.now(),
        updatedAt: Date.now(),
        messageCount: 5,
      }

      expect(session.id).toBeTypeOf('number')
      expect(session.title).toBeTypeOf('string')
      expect(session.createdAt).toBeTypeOf('number')
      expect(session.updatedAt).toBeTypeOf('number')
      expect(session.messageCount).toBeTypeOf('number')
    })

    it('ChatMessage 应该有正确的结构', () => {
      const message: ChatMessage = {
        id: 1,
        sessionId: 1,
        role: 'user',
        content: '测试消息',
        contextIds: [1, 2, 3],
        createdAt: Date.now(),
        rating: 1,
      }

      expect(message.id).toBeTypeOf('number')
      expect(message.sessionId).toBeTypeOf('number')
      expect(['user', 'assistant']).toContain(message.role)
      expect(message.content).toBeTypeOf('string')
      expect(message.contextIds).toBeInstanceOf(Array)
      expect(message.createdAt).toBeTypeOf('number')
      expect([1, -1, null, undefined]).toContain(message.rating)
    })

    it('LocalChatMessage 应该有正确的结构', () => {
      const localMessage: LocalChatMessage = {
        localId: 'local-123',
        role: 'assistant',
        content: '本地消息',
        ts: Date.now(),
        dbId: 1,
        rating: null,
      }

      expect(localMessage.localId).toBeTypeOf('string')
      expect(['user', 'assistant']).toContain(localMessage.role)
      expect(localMessage.content).toBeTypeOf('string')
      expect(localMessage.ts).toBeTypeOf('number')
    })

    it('UserFeedback 应该有正确的结构', () => {
      const feedback: UserFeedback = {
        id: 1,
        category: 'bug',
        title: '测试反馈',
        content: '这是一个测试反馈',
        screenshotPath: '/path/to/screenshot.png',
        contextSessionId: 1,
        status: 'pending',
        createdAt: Date.now(),
      }

      expect(feedback.id).toBeTypeOf('number')
      expect(['bug', 'feature', 'experience', 'other']).toContain(feedback.category)
      expect(feedback.title).toBeTypeOf('string')
      expect(feedback.content).toBeTypeOf('string')
      expect(['pending', 'noted', 'resolved']).toContain(feedback.status)
      expect(feedback.createdAt).toBeTypeOf('number')
    })

    it('FeedbackFormData 应该有正确的结构', () => {
      const formData: FeedbackFormData = {
        category: 'feature',
        title: '功能建议',
        content: '建议添加新功能',
        screenshotPath: '/path/to/screenshot.png',
        contextSessionId: 1,
      }

      expect(['bug', 'feature', 'experience', 'other']).toContain(formData.category)
      expect(formData.title).toBeTypeOf('string')
      expect(formData.content).toBeTypeOf('string')
    })
  })
})

