export type RiskLevel = 'low' | 'medium' | 'high'

export type AutomationStep =
  | { type: 'open_url'; url: string }
  | { type: 'open_file'; path: string }
  | { type: 'copy_to_clipboard'; text: string }
  | { type: 'create_note'; content: string }

export interface AutomationEvidence {
  activityId: number
  timestamp: number
  appName: string
  windowTitle: string
}

export interface AutomationProposal {
  id: number
  title: string
  description: string
  confidence: number
  riskLevel: RiskLevel | string
  steps: AutomationStep[]
  evidence: AutomationEvidence[]
  createdAt: number
}

export interface Execution {
  id: number
  proposalId?: number | null
  action: string
  status: 'running' | 'success' | 'failed' | 'cancelled' | string
  createdAt: number
  finishedAt?: number | null
  errorMessage?: string | null
  metadata?: Record<string, unknown> | null
}

export interface ExecutionResult {
  executionId: number
  status: string
}

export interface AgentProposeParams {
  timeWindowHours?: number
  limit?: number
}


