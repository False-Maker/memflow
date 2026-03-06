/**
 * Tauri API Mock 辅助工具
 * 
 * 提供更完善的 mock 功能：
 * 1. 精确控制 get_config / get_activities 的调用顺序
 * 2. 异步加载行为模拟（支持延迟）
 * 3. Loading 状态模拟
 * 4. 组件内部状态管理的测试支持
 */

import { vi } from 'vitest'
import type { ActivityLog, AppConfig } from '../contexts/AppContext'

// 命令名称类型
export type TauriCommand = 
  | 'get_config'
  | 'get_activities'
  | 'search_activities'
  | 'parse_query_intent'
  | 'start_recording'
  | 'stop_recording'

// Mock 配置选项
export interface MockOptions {
  /** 命令延迟时间（毫秒） */
  delay?: number
  /** 是否模拟错误 */
  shouldError?: boolean
  /** 错误消息 */
  errorMessage?: string
}

// 默认的活动数据
export const defaultActivities: ActivityLog[] = [
  {
    id: 1,
    timestamp: Date.now() - 3600000,
    appName: 'Chrome',
    windowTitle: 'React Documentation',
    imagePath: '/test1.png',
    ocrText: 'Getting Started with React',
  },
  {
    id: 2,
    timestamp: Date.now() - 7200000,
    appName: 'VS Code',
    windowTitle: 'src/App.tsx',
    imagePath: '/test2.png',
    ocrText: 'function App() {',
  },
  {
    id: 3,
    timestamp: Date.now() - 86400000,
    appName: 'Slack',
    windowTitle: 'general',
    imagePath: '/test3.png',
  },
]

// 默认配置
export const defaultConfig: AppConfig = {
  recordingInterval: 5000,
  ocrEnabled: true,
  ocrEngine: 'rapidocr',
  ocrRedactionEnabled: true,
  ocrRedactionLevel: 'basic',
  compressionQuality: 80,
  targetResolutionScale: 1.0,
  aiEnabled: true,
  enableFocusAnalytics: true,
  enableProactiveAssistant: false,
  retentionDays: 30,
  chatModel: 'gpt-4o-mini',
  embeddingModel: 'text-embedding-3-small',
  blocklistEnabled: false,
  blocklistMode: 'blocklist',
  privacyModeEnabled: false,
  autostartEnabled: false,
}

/**
 * 创建可配置的 Tauri API mock
 * 
 * @example
 * // 基本用法 - 按顺序返回
 * const { mockInvoke, resetMock } = createTauriMock([
 *   { command: 'get_config', response: mockConfig },
 *   { command: 'get_activities', response: mockActivities },
 * ])
 * 
 * @example
 * // 带延迟的异步加载
 * const { mockInvoke, resetMock } = createTauriMock([
 *   { command: 'get_config', response: mockConfig, delay: 100 },
 *   { command: 'get_activities', response: mockActivities, delay: 200 },
 * ])
 * 
 * @example
 * // 模拟错误
 * const { mockInvoke, resetMock } = createTauriMock([
 *   { command: 'get_config', shouldError: true, errorMessage: 'Config not found' },
 * ])
 */
export function createTauriMock(configs: Array<{
  command: TauriCommand
  response?: unknown
  delay?: number
  shouldError?: boolean
  errorMessage?: string
  /** 匹配特定参数时才返回此响应 */
  matcher?: (args: Record<string, unknown>) => boolean
}>) {
  // 维护调用历史
  const callHistory: Array<{ command: string; args: Record<string, unknown> }> = []
  
  // 维护每个命令的调用计数
  const commandCallCounts: Record<string, number> = {}
  
  // 创建 mock 实现
  const mockFn = vi.fn(async (command: string, args?: Record<string, unknown>) => {
    // 记录调用
    callHistory.push({ command, args: args || {} })
    commandCallCounts[command] = (commandCallCounts[command] || 0) + 1
    
    // 找到匹配的配置
    let matchedConfig = configs.find(config => {
      if (config.command !== command) return false
      if (config.matcher && args) {
        return config.matcher(args)
      }
      return true
    })
    
    // 如果没有找到精确匹配，尝试找第一个匹配命令的配置（用于 mockResolvedValueOnce 场景）
    if (!matchedConfig) {
      const configsForCommand = configs.filter(c => c.command === command)
      const callCount = commandCallCounts[command] || 0
      if (callCount <= configsForCommand.length) {
        matchedConfig = configsForCommand[callCount - 1]
      }
    }
    
    // 如果还是没有匹配的配置，抛出错误
    if (!matchedConfig) {
      throw new Error(`No mock configured for command: ${command}`)
    }
    
    // 如果配置了错误
    if (matchedConfig.shouldError) {
      throw new Error(matchedConfig.errorMessage || `Command ${command} failed`)
    }
    
    // 如果配置了延迟
    if (matchedConfig.delay && matchedConfig.delay > 0) {
      await new Promise(resolve => setTimeout(resolve, matchedConfig.delay))
    }
    
    return matchedConfig.response
  })
  
  // 重置函数
  const resetMock = () => {
    callHistory.length = 0
    Object.keys(commandCallCounts).forEach(key => delete commandCallCounts[key])
    mockFn.mockClear()
  }
  
  // 获取调用历史
  const getCallHistory = () => [...callHistory]
  
  // 获取特定命令的调用次数
  const getCommandCallCount = (command: string) => commandCallCounts[command] || 0
  
  // 检查是否已调用特定命令
  const hasCalledCommand = (command: string) => callHistory.some(c => c.command === command)
  
  return {
    mockFn,
    resetMock,
    getCallHistory,
    getCommandCallCount,
    hasCalledCommand,
  }
}

/**
 * 创建简化的 Timeline 测试 mock
 * 
 * @example
 * const { mockApi, renderWithTimeline } = createTimelineMock()
 * 
 * renderWithTimeline()
 * 
 * // 等待加载完成
 * await waitFor(() => {
 *   expect(mockApi.hasCalledCommand('get_config')).toBe(true)
 *   expect(mockApi.hasCalledCommand('get_activities')).toBe(true)
 * })
 */
export function createTimelineMock(customConfigs?: Array<{
  command: TauriCommand
  response?: unknown
  delay?: number
  shouldError?: boolean
  errorMessage?: string
}>) {
  // 默认配置：先 get_config，再 get_activities
  const defaultMockConfigs = [
    { command: 'get_config' as const, response: defaultConfig, delay: 50 },
    { command: 'get_activities' as const, response: defaultActivities, delay: 100 },
    { command: 'search_activities' as const, response: { items: defaultActivities, total: defaultActivities.length } },
    { command: 'parse_query_intent' as const, response: { app_name: 'Chrome', keywords: ['test'], date_range: null, has_ocr: null } },
  ]
  
  // 合并自定义配置
  const configs = customConfigs 
    ? [...defaultMockConfigs.filter(d => !customConfigs.some(c => c.command === d.command)), ...customConfigs]
    : defaultMockConfigs
  
  const { mockFn, resetMock, getCallHistory, getCommandCallCount, hasCalledCommand } = createTauriMock(configs)
  
  return {
    mockFn,
    resetMock,
    getCallHistory,
    getCommandCallCount,
    hasCalledCommand,
    // 便捷方法：创建带自定义响应的 mock
    withConfig: (newConfigs: typeof configs) => createTimelineMock(newConfigs),
    // 默认数据
    defaultActivities,
    defaultConfig,
  }
}

/**
 * 创建用于测试异步加载行为的 mock
 * 
 * @example
 * const { mockApi, advanceTimersByDelay } = createAsyncMock()
 * 
 * // 模拟组件渲染时触发加载
 * render(<Timeline />, { wrapper })
 * 
 * // 检查初始 loading 状态
 * expect(screen.getByText(/加载中/i)).toBeInTheDocument()
 * 
 * // 推进时间并等待异步完成
 * await advanceTimersByDelay(200)
 * 
 * // 验证数据加载完成
 * await waitFor(() => {
 *   expect(screen.queryByText(/加载中/i)).not.toBeInTheDocument()
 * })
 */
export function createAsyncMock() {
  const timers: Array<{ delay: number; callback: () => void }> = []
  let timerId = 0
  
  // 模拟 setTimeout
  const originalSetTimeout = globalThis.setTimeout
  const mockSetTimeout = vi.fn((callback: () => void, delay: number) => {
    const id = ++timerId
    timers.push({ delay, callback })
    return id
  })
  
  // 模拟 clearTimeout
  const mockClearTimeout = vi.fn((id: number) => {
    const index = timers.findIndex(t => (t.callback as unknown as number) === id)
    if (index !== -1) {
      timers.splice(index, 1)
    }
  })
  
  // 推进时间
  const advanceTimersByDelay = async (delay: number) => {
    const timersToRun = timers.filter(t => t.delay <= delay)
    for (const timer of timersToRun) {
      timer.callback()
      const idx = timers.indexOf(timer)
      if (idx !== -1) timers.splice(idx, 1)
    }
  }
  
  return {
    mockSetTimeout,
    mockClearTimeout,
    advanceTimersByDelay,
    timers,
  }
}

/**
 * 创建一个可以模拟不同加载阶段的 mock
 * 
 * @example
 * const loadingMock = createLoadingStateMock()
 * 
 * // 初始状态：未加载
 * loadingMock.setLoadingState('initial')
 * 
 * // 模拟加载中
 * loadingMock.setLoadingState('loading')
 * render(<Timeline />, { wrapper })
 * expect(screen.getByTestId('loading-spinner')).toBeInTheDocument()
 * 
 * // 模拟加载完成
 * loadingMock.setLoadingState('loaded')
 * await waitFor(() => {
 *   expect(screen.queryByTestId('loading-spinner')).not.toBeInTheDocument()
 * })
 */
export function createLoadingStateMock() {
  type LoadingState = 'initial' | 'loading' | 'loaded' | 'error'
  let currentState: LoadingState = 'initial'
  
  const stateCallbacks: Array<(state: LoadingState) => void> = []
  
  const setLoadingState = (state: LoadingState) => {
    currentState = state
    stateCallbacks.forEach(cb => cb(state))
  }
  
  const subscribe = (callback: (state: LoadingState) => void) => {
    stateCallbacks.push(callback)
    return () => {
      const idx = stateCallbacks.indexOf(callback)
      if (idx !== -1) stateCallbacks.splice(idx, 1)
    }
  }
  
  return {
    getState: () => currentState,
    setLoadingState,
    subscribe,
    isInitial: () => currentState === 'initial',
    isLoading: () => currentState === 'loading',
    isLoaded: () => currentState === 'loaded',
    isError: () => currentState === 'error',
  }
}

/**
 * 验证调用顺序
 * 
 * @example
 * const { getCallHistory } = createTauriMock([...])
 * 
 * // 验证调用顺序
 * const history = getCallHistory()
 * expect(history[0].command).toBe('get_config')
 * expect(history[1].command).toBe('get_activities')
 * 
 * // 或使用验证函数
 * expectCallsInOrder(history, ['get_config', 'get_activities'])
 */
export function expectCallsInOrder(
  history: Array<{ command: string; args: Record<string, unknown> }>,
  expectedCommands: string[]
) {
  expect(history.length).toBeGreaterThanOrEqual(expectedCommands.length)
  
  for (let i = 0; i < expectedCommands.length; i++) {
    expect(history[i].command).toBe(expectedCommands[i])
  }
}

/**
 * 等待指定时间
 */
export const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

/**
 * 创建一个延迟的 Promise
 * 
 * @example
 * await delayedResolve(100, { data: 'test' })
 * // 等待 100ms 后返回 { data: 'test' }
 */
export function delayedResolve<T>(ms: number, value: T): Promise<T> {
  return new Promise(resolve => setTimeout(() => resolve(value), ms))
}

/**
 * 创建一个延迟的拒绝 Promise
 * 
 * @example
 * await delayedReject(100, new Error('Failed'))
 * // 等待 100ms 后抛出错误
 */
export function delayedReject<T>(ms: number, error: T): Promise<T> {
  return new Promise((_, reject) => setTimeout(() => reject(error), ms))
}
