import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { render } from '@testing-library/react'
import { AppProvider, useApp } from './AppContext'
import type { AppState, ActivityLog, AppConfig, SearchParams } from './AppContext'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// 由于 AppContext 使用了 Tauri API，我们需要 mock 它
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `tauri://localhost/${path}`),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

const mockInvoke = vi.mocked(invoke)
const mockListen = vi.mocked(listen)

describe('AppContext', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockInvoke.mockResolvedValue(undefined)
    mockListen.mockResolvedValue(() => {})
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('appReducer', () => {
    const initialState: AppState = {
      isRecording: false,
      activities: [],
      currentView: 'timeline',
      config: {
        recordingInterval: 5000,
        ocrEnabled: true,
        ocrRedactionEnabled: true,
        ocrRedactionLevel: 'basic',
        aiEnabled: false,
        enableFocusAnalytics: false,
        enableProactiveAssistant: false,
        retentionDays: 30,
        blocklistEnabled: false,
        blocklistMode: 'blocklist',
        privacyModeEnabled: false,
      },
      configLoaded: false,
      configError: null,
    }

    it('应该设置录制状态', () => {
      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      act(() => {
        result.current.dispatch({ type: 'SET_RECORDING', payload: true })
      })

      expect(result.current.state.isRecording).toBe(true)
    })

    it('应该添加活动', () => {
      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      const activity: ActivityLog = {
        id: 1,
        timestamp: Date.now(),
        appName: 'Test App',
        windowTitle: 'Test Window',
        imagePath: '/path/to/image.png',
        ocrText: 'test text',
      }

      act(() => {
        result.current.dispatch({ type: 'ADD_ACTIVITY', payload: activity })
      })

      expect(result.current.state.activities).toHaveLength(1)
      expect(result.current.state.activities[0]).toEqual(activity)
    })

    it('应该更新活动的 OCR 文本', () => {
      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      const activity: ActivityLog = {
        id: 1,
        timestamp: Date.now(),
        appName: 'Test App',
        windowTitle: 'Test Window',
        imagePath: '/path/to/image.png',
      }

      act(() => {
        result.current.dispatch({ type: 'ADD_ACTIVITY', payload: activity })
      })

      act(() => {
        result.current.dispatch({
          type: 'UPDATE_ACTIVITY_OCR',
          payload: { id: 1, ocrText: 'updated text' },
        })
      })

      expect(result.current.state.activities[0].ocrText).toBe('updated text')
    })

    it('应该设置活动列表', () => {
      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      const activities: ActivityLog[] = [
        {
          id: 1,
          timestamp: Date.now(),
          appName: 'App 1',
          windowTitle: 'Window 1',
          imagePath: '/path/to/image1.png',
        },
        {
          id: 2,
          timestamp: Date.now(),
          appName: 'App 2',
          windowTitle: 'Window 2',
          imagePath: '/path/to/image2.png',
        },
      ]

      act(() => {
        result.current.dispatch({ type: 'SET_ACTIVITIES', payload: activities })
      })

      expect(result.current.state.activities).toEqual(activities)
    })

    it('应该设置当前视图', () => {
      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      act(() => {
        result.current.dispatch({ type: 'SET_VIEW', payload: 'graph' })
      })

      expect(result.current.state.currentView).toBe('graph')
    })

    it('应该设置配置', () => {
      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      const newConfig: AppConfig = {
        recordingInterval: 3000,
        ocrEnabled: false,
        aiEnabled: true,
        enableFocusAnalytics: true,
        enableProactiveAssistant: true,
        retentionDays: 60,
        blocklistEnabled: true,
        blocklistMode: 'allowlist',
        privacyModeEnabled: true,
      }

      act(() => {
        result.current.dispatch({ type: 'SET_CONFIG', payload: newConfig })
      })

      expect(result.current.state.config).toEqual(newConfig)
    })

    it('应该设置搜索参数', () => {
      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      const searchParams: SearchParams = {
        query: 'test query',
        appName: 'Test App',
        limit: 10,
      }

      act(() => {
        result.current.dispatch({ type: 'SET_SEARCH_PARAMS', payload: searchParams })
      })

      expect(result.current.state.lastSearchParams).toEqual(searchParams)
    })
  })

  describe('useApp hook', () => {
    it('应该在 AppProvider 外使用时抛出错误', () => {
      // 抑制控制台错误输出和 React 错误
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      const originalError = console.error
      console.error = () => {} // 完全抑制错误输出

      try {
        expect(() => {
          renderHook(() => useApp())
        }).toThrow('useApp must be used within an AppProvider')
      } finally {
        console.error = originalError
        consoleErrorSpy.mockRestore()
      }
    })

    it('应该在 AppProvider 内正常使用', () => {
      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      expect(result.current.state).toBeDefined()
      expect(result.current.dispatch).toBeDefined()
      expect(result.current.startRecording).toBeDefined()
      expect(result.current.stopRecording).toBeDefined()
      expect(result.current.loadActivities).toBeDefined()
      expect(result.current.searchActivities).toBeDefined()
    })
  })

  describe('AppProvider 初始化', () => {
    it('应该加载配置和活动列表', async () => {
      const mockConfig: AppConfig = {
        recordingInterval: 3000,
        ocrEnabled: true,
        aiEnabled: true,
        enableFocusAnalytics: true,
        enableProactiveAssistant: true,
        retentionDays: 60,
        blocklistEnabled: true,
        blocklistMode: 'allowlist',
        privacyModeEnabled: true,
      }

      const mockActivities: ActivityLog[] = [
        {
          id: 1,
          timestamp: Date.now(),
          appName: 'Test App',
          windowTitle: 'Test Window',
          imagePath: '/path/to/image.png',
        },
      ]

      mockInvoke
        .mockResolvedValueOnce(mockConfig)
        .mockResolvedValueOnce(mockActivities)

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_config')
        expect(mockInvoke).toHaveBeenCalledWith('get_activities', { limit: 100 })
      })

      await waitFor(() => {
        expect(result.current.state.config).toEqual(mockConfig)
        expect(result.current.state.activities).toEqual(mockActivities)
      })
    })

    it('应该在配置加载失败时记录错误状态', async () => {
      mockInvoke
        .mockRejectedValueOnce(new Error('Config not found'))
        .mockResolvedValueOnce([])

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_config')
      })

      await waitFor(() => {
        expect(result.current.state.configLoaded).toBe(true)
        expect(result.current.state.configError).toBe('Config not found')
      })

      expect(consoleErrorSpy).toHaveBeenCalledWith(
        'Failed to load config from backend:',
        'Config not found'
      )
      // 配置保持初始占位值
      expect(result.current.state.config).toBeDefined()

      consoleErrorSpy.mockRestore()
    })

    it('应该监听后端事件', async () => {
      let recordingCallback: (event: { payload: boolean }) => void
      let activityCallback: (event: { payload: ActivityLog }) => void
      let ocrCallback: (event: { payload: { id: number; ocrText: string } }) => void

      mockListen.mockImplementation((eventName: string) => {
        if (eventName === 'recording-status') {
          return Promise.resolve(() => {
            if (recordingCallback) recordingCallback({ payload: true })
          })
        }
        if (eventName === 'new-activity') {
          return Promise.resolve(() => {
            if (activityCallback) {
              activityCallback({
                payload: {
                  id: 1,
                  timestamp: Date.now(),
                  appName: 'Test',
                  windowTitle: 'Test',
                  imagePath: '/test.png',
                },
              })
            }
          })
        }
        if (eventName === 'ocr-updated') {
          return Promise.resolve(() => {
            if (ocrCallback) {
              ocrCallback({ payload: { id: 1, ocrText: 'updated' } })
            }
          })
        }
        if (eventName === 'backend-log') {
          return Promise.resolve(() => {})
        }
        return Promise.resolve(() => {})
      })

      mockInvoke.mockResolvedValue([])

      renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(mockListen).toHaveBeenCalledWith('recording-status', expect.any(Function))
        expect(mockListen).toHaveBeenCalledWith('new-activity', expect.any(Function))
        expect(mockListen).toHaveBeenCalledWith('ocr-updated', expect.any(Function))
        expect(mockListen).toHaveBeenCalledWith('backend-log', expect.any(Function))
      })
    })
  })

  describe('startRecording', () => {
    it('应该成功启动录制', async () => {
      mockInvoke.mockResolvedValue(undefined)
      mockInvoke.mockResolvedValueOnce(undefined).mockResolvedValueOnce([])

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      const consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {})

      await act(async () => {
        await result.current.startRecording()
      })

      expect(mockInvoke).toHaveBeenCalledWith('start_recording')
      expect(result.current.state.isRecording).toBe(true)

      consoleLogSpy.mockRestore()
    })

    it('应该在启动录制失败时显示错误', async () => {
      const error = new Error('Failed to start')
      // 初始化时的调用：get_config, get_activities
      mockInvoke
        .mockResolvedValueOnce(undefined) // get_config
        .mockResolvedValueOnce([]) // get_activities
        .mockRejectedValueOnce(error) // start_recording

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {})

      await act(async () => {
        await result.current.startRecording()
      })

      expect(consoleErrorSpy).toHaveBeenCalledWith('Failed to start recording:', error)
      expect(alertSpy).toHaveBeenCalled()

      consoleErrorSpy.mockRestore()
      alertSpy.mockRestore()
    })
  })

  describe('stopRecording', () => {
    it('应该成功停止录制', async () => {
      mockInvoke.mockResolvedValue(undefined)
      mockInvoke.mockResolvedValueOnce(undefined).mockResolvedValueOnce([])

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      await act(async () => {
        await result.current.stopRecording()
      })

      expect(mockInvoke).toHaveBeenCalledWith('stop_recording')
      expect(result.current.state.isRecording).toBe(false)
    })

    it('应该在停止录制失败时记录错误', async () => {
      const error = new Error('Failed to stop')
      // 初始化时的调用：get_config, get_activities
      mockInvoke
        .mockResolvedValueOnce(undefined) // get_config
        .mockResolvedValueOnce([]) // get_activities
        .mockRejectedValueOnce(error) // stop_recording

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      await act(async () => {
        await result.current.stopRecording()
      })

      expect(consoleErrorSpy).toHaveBeenCalledWith('Failed to stop recording:', error)

      consoleErrorSpy.mockRestore()
    })
  })

  describe('loadActivities', () => {
    it('应该成功加载活动列表', async () => {
      const mockActivities: ActivityLog[] = [
        {
          id: 1,
          timestamp: Date.now(),
          appName: 'App 1',
          windowTitle: 'Window 1',
          imagePath: '/path/to/image1.png',
        },
        {
          id: 2,
          timestamp: Date.now(),
          appName: 'App 2',
          windowTitle: 'Window 2',
          imagePath: '/path/to/image2.png',
        },
      ]

      // 初始化时的调用：get_config, get_activities (第一次)
      // 然后测试中调用 loadActivities (第二次 get_activities)
      mockInvoke
        .mockResolvedValueOnce(undefined) // get_config
        .mockResolvedValueOnce([]) // get_activities (初始化)
        .mockResolvedValueOnce(mockActivities) // get_activities (测试中调用)

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      await act(async () => {
        await result.current.loadActivities()
      })

      expect(mockInvoke).toHaveBeenCalledWith('get_activities', { limit: 100 })
      expect(result.current.state.activities).toEqual(mockActivities)
    })

    it('应该在加载活动失败时记录错误', async () => {
      const error = new Error('Failed to load')
      // 初始化时的调用：get_config, get_activities
      // 然后测试中调用 loadActivities (会失败)
      mockInvoke
        .mockResolvedValueOnce(undefined) // get_config
        .mockResolvedValueOnce([]) // get_activities (初始化)
        .mockRejectedValueOnce(error) // get_activities (测试中调用，会失败)

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      await act(async () => {
        await result.current.loadActivities()
      })

      expect(consoleErrorSpy).toHaveBeenCalledWith('Failed to load activities:', error)

      consoleErrorSpy.mockRestore()
    })
  })

  describe('searchActivities', () => {
    it('应该成功搜索活动', async () => {
      const mockSearchResult = {
        items: [
          {
            id: 1,
            timestamp: Date.now(),
            appName: 'Test App',
            windowTitle: 'Test Window',
            imagePath: '/path/to/image.png',
          },
        ],
        total: 1,
      }

      const searchParams: SearchParams = {
        query: 'test',
        appName: 'Test App',
        limit: 10,
      }

      mockInvoke
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce(mockSearchResult)

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      await act(async () => {
        await result.current.searchActivities(searchParams)
      })

      expect(mockInvoke).toHaveBeenCalledWith('search_activities', {
        query: 'test',
        appName: 'Test App',
        fromTs: undefined,
        toTs: undefined,
        hasOcr: undefined,
        limit: 10,
        offset: undefined,
        orderBy: undefined,
      })
      expect(result.current.state.activities).toEqual(mockSearchResult.items)
      expect(result.current.state.lastSearchParams).toEqual(searchParams)
      expect(result.current.state.searchTotal).toBe(1)
    })

    it('应该处理完整的搜索参数', async () => {
      const mockSearchResult = { items: [], total: 0 }
      const searchParams: SearchParams = {
        query: 'test query',
        appName: 'Chrome',
        fromTs: 1000000000,
        toTs: 2000000000,
        hasOcr: true,
        limit: 20,
        offset: 0,
        orderBy: 'rank',
      }

      mockInvoke
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce(mockSearchResult)

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      await act(async () => {
        await result.current.searchActivities(searchParams)
      })

      expect(mockInvoke).toHaveBeenCalledWith('search_activities', {
        query: 'test query',
        appName: 'Chrome',
        fromTs: 1000000000,
        toTs: 2000000000,
        hasOcr: true,
        limit: 20,
        offset: 0,
        orderBy: 'rank',
      })
    })

    it('应该在搜索失败时记录错误', async () => {
      const error = new Error('Search failed')
      // 初始化时的调用：get_config, get_activities
      // 然后测试中调用 searchActivities (会失败)
      mockInvoke
        .mockResolvedValueOnce(undefined) // get_config
        .mockResolvedValueOnce([]) // get_activities (初始化)
        .mockRejectedValueOnce(error) // search_activities (测试中调用，会失败)

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      await act(async () => {
        await result.current.searchActivities({ query: 'test' })
      })

      expect(consoleErrorSpy).toHaveBeenCalledWith('Failed to search activities:', error)

      consoleErrorSpy.mockRestore()
    })
  })

  describe('reducer edge cases', () => {
    it('应该处理未知的 action 类型', () => {
      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      const initialState = result.current.state

      act(() => {
        // @ts-expect-error - 测试未知 action 类型
        result.current.dispatch({ type: 'UNKNOWN_ACTION', payload: null })
      })

      expect(result.current.state).toEqual(initialState)
    })

    it('应该正确更新已存在活动的 OCR 文本', () => {
      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      const activities: ActivityLog[] = [
        {
          id: 1,
          timestamp: Date.now(),
          appName: 'App 1',
          windowTitle: 'Window 1',
          imagePath: '/path/to/image1.png',
        },
        {
          id: 2,
          timestamp: Date.now(),
          appName: 'App 2',
          windowTitle: 'Window 2',
          imagePath: '/path/to/image2.png',
        },
      ]

      act(() => {
        result.current.dispatch({ type: 'SET_ACTIVITIES', payload: activities })
      })

      act(() => {
        result.current.dispatch({
          type: 'UPDATE_ACTIVITY_OCR',
          payload: { id: 1, ocrText: 'updated text' },
        })
      })

      expect(result.current.state.activities[0].ocrText).toBe('updated text')
      expect(result.current.state.activities[1].ocrText).toBeUndefined()
    })

    it('应该更新不存在的活动的 OCR 文本（无副作用）', () => {
      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      const activities: ActivityLog[] = [
        {
          id: 1,
          timestamp: Date.now(),
          appName: 'App 1',
          windowTitle: 'Window 1',
          imagePath: '/path/to/image1.png',
        },
      ]

      act(() => {
        result.current.dispatch({ type: 'SET_ACTIVITIES', payload: activities })
      })

      act(() => {
        result.current.dispatch({
          type: 'UPDATE_ACTIVITY_OCR',
          payload: { id: 999, ocrText: 'should not update' },
        })
      })

      expect(result.current.state.activities[0].ocrText).toBeUndefined()
    })
  })
})

