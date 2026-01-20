import { describe, it, expect } from 'vitest'
import type {
  RiskLevel,
  AutomationStep,
  AutomationEvidence,
  AutomationProposal,
  Execution,
  ExecutionResult,
  AgentProposeParams,
} from './agent'

describe('agent types', () => {
  describe('类型定义', () => {
    it('RiskLevel 应该是有效的风险级别', () => {
      const riskLevels: RiskLevel[] = ['low', 'medium', 'high']
      
      riskLevels.forEach(level => {
        expect(['low', 'medium', 'high']).toContain(level)
      })
    })

    it('AutomationStep 应该有正确的结构', () => {
      const steps: AutomationStep[] = [
        { type: 'open_url', url: 'https://example.com' },
        { type: 'open_file', path: '/path/to/file.txt' },
        { type: 'copy_to_clipboard', text: '测试文本' },
        { type: 'create_note', content: '笔记内容' },
      ]

      steps.forEach(step => {
        expect(step).toHaveProperty('type')
        expect(['open_url', 'open_file', 'copy_to_clipboard', 'create_note']).toContain(step.type)
        
        if (step.type === 'open_url') {
          expect(step).toHaveProperty('url')
          expect(step.url).toBeTypeOf('string')
        } else if (step.type === 'open_file') {
          expect(step).toHaveProperty('path')
          expect(step.path).toBeTypeOf('string')
        } else if (step.type === 'copy_to_clipboard') {
          expect(step).toHaveProperty('text')
          expect(step.text).toBeTypeOf('string')
        } else if (step.type === 'create_note') {
          expect(step).toHaveProperty('content')
          expect(step.content).toBeTypeOf('string')
        }
      })
    })

    it('AutomationEvidence 应该有正确的结构', () => {
      const evidence: AutomationEvidence = {
        activityId: 1,
        timestamp: Date.now(),
        appName: '测试应用',
        windowTitle: '测试窗口',
      }

      expect(evidence.activityId).toBeTypeOf('number')
      expect(evidence.timestamp).toBeTypeOf('number')
      expect(evidence.appName).toBeTypeOf('string')
      expect(evidence.windowTitle).toBeTypeOf('string')
    })

    it('AutomationProposal 应该有正确的结构', () => {
      const proposal: AutomationProposal = {
        id: 1,
        title: '测试提案',
        description: '这是一个测试提案',
        confidence: 0.85,
        riskLevel: 'medium',
        steps: [
          { type: 'open_url', url: 'https://example.com' },
        ],
        evidence: [
          {
            activityId: 1,
            timestamp: Date.now(),
            appName: '测试应用',
            windowTitle: '测试窗口',
          },
        ],
        createdAt: Date.now(),
      }

      expect(proposal.id).toBeTypeOf('number')
      expect(proposal.title).toBeTypeOf('string')
      expect(proposal.description).toBeTypeOf('string')
      expect(proposal.confidence).toBeTypeOf('number')
      expect(proposal.confidence).toBeGreaterThanOrEqual(0)
      expect(proposal.confidence).toBeLessThanOrEqual(1)
      expect(proposal.riskLevel).toBeTypeOf('string')
      expect(proposal.steps).toBeInstanceOf(Array)
      expect(proposal.evidence).toBeInstanceOf(Array)
      expect(proposal.createdAt).toBeTypeOf('number')
    })

    it('Execution 应该有正确的结构', () => {
      const execution: Execution = {
        id: 1,
        proposalId: 1,
        action: 'open_url',
        status: 'running',
        createdAt: Date.now(),
        finishedAt: null,
        errorMessage: null,
        metadata: { url: 'https://example.com' },
      }

      expect(execution.id).toBeTypeOf('number')
      expect(execution.action).toBeTypeOf('string')
      expect(['running', 'success', 'failed', 'cancelled']).toContain(execution.status)
      expect(execution.createdAt).toBeTypeOf('number')
    })

    it('ExecutionResult 应该有正确的结构', () => {
      const result: ExecutionResult = {
        executionId: 1,
        status: 'success',
      }

      expect(result.executionId).toBeTypeOf('number')
      expect(result.status).toBeTypeOf('string')
    })

    it('AgentProposeParams 应该有正确的结构', () => {
      const params: AgentProposeParams = {
        timeWindowHours: 24,
        limit: 10,
      }

      if (params.timeWindowHours !== undefined) {
        expect(params.timeWindowHours).toBeTypeOf('number')
        expect(params.timeWindowHours).toBeGreaterThan(0)
      }
      
      if (params.limit !== undefined) {
        expect(params.limit).toBeTypeOf('number')
        expect(params.limit).toBeGreaterThan(0)
      }
    })
  })
})

