import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { AppProvider, useApp } from '../../contexts/AppContext'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ActivityLog, AppConfig } from '../../contexts/AppContext'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `tauri://localhost/${path}`),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

const mockInvoke = vi.mocked(invoke)
const mockListen = vi.mocked(listen)

describe('AppContext Integration Tests', () => {
  let unlistenFunctions: (() => void)[] = []
  let eventHandlers: Record<string, (event: { payload: unknown }) => void> = {}

  beforeEach(() => {
    vi.clearAllMocks()
    unlistenFunctions = []
    eventHandlers = {}

    // Mock listen to capture event handlers
    mockListen.mockImplementation((eventName: string, handler: (event: { payload: unknown }) => void) => {
      eventHandlers[eventName] = handler
      const unlisten = () => {
        delete eventHandlers[eventName]
      }
      unlistenFunctions.push(unlisten)
      return Promise.resolve(unlisten)
    })
  })

  afterEach(() => {
    unlistenFunctions.forEach((fn) => fn())
    unlistenFunctions = []
    eventHandlers = {}
  })

  describe('完整的录制流程', () => {
    it('应该完成从启动到停止录制的完整流程', async () => {
      const config: AppConfig = {
        recordingInterval: 5000,
        ocrEnabled: true,
        aiEnabled: false,
        enableFocusAnalytics: false,
        enableProactiveAssistant: false,
        retentionDays: 30,
        blocklistEnabled: false,
        blocklistMode: 'blocklist',
        privacyModeEnabled: false,
      }

      mockInvoke
        .mockResolvedValueOnce(config)
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce(undefined) // start_recording
        .mockResolvedValueOnce(undefined) // stop_recording

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      // 等待初始化
      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      // 启动录制
      await act(async () => {
        await result.current.startRecording()
      })

      expect(result.current.state.isRecording).toBe(true)

      // 模拟后端发送录制状态事件
      if (eventHandlers['recording-status']) {
        act(() => {
          eventHandlers['recording-status']({ payload: true })
        })
      }

      await waitFor(() => {
        expect(result.current.state.isRecording).toBe(true)
      })

      // 停止录制
      await act(async () => {
        await result.current.stopRecording()
      })

      expect(result.current.state.isRecording).toBe(false)
    })
  })

  describe('活动生命周期', () => {
    it('应该处理从创建到 OCR 更新的完整活动生命周期', async () => {
      const config: AppConfig = {
        recordingInterval: 5000,
        ocrEnabled: true,
        aiEnabled: false,
        enableFocusAnalytics: false,
        enableProactiveAssistant: false,
        retentionDays: 30,
        blocklistEnabled: false,
        blocklistMode: 'blocklist',
        privacyModeEnabled: false,
      }

      mockInvoke
        .mockResolvedValueOnce(config)
        .mockResolvedValueOnce([])

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      // 添加活动
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

      expect(result.current.state.activities).toHaveLength(1)
      expect(result.current.state.activities[0].ocrText).toBeUndefined()

      // 模拟后端发送 OCR 更新事件
      if (eventHandlers['ocr-updated']) {
        act(() => {
          eventHandlers['ocr-updated']({
            payload: { id: 1, ocrText: 'Updated OCR text' },
          })
        })
      }

      await waitFor(() => {
        expect(result.current.state.activities[0].ocrText).toBe('Updated OCR text')
      })
    })

    it('应该处理多个活动的新增', async () => {
      const config: AppConfig = {
        recordingInterval: 5000,
        ocrEnabled: true,
        aiEnabled: false,
        enableFocusAnalytics: false,
        enableProactiveAssistant: false,
        retentionDays: 30,
        blocklistEnabled: false,
        blocklistMode: 'blocklist',
        privacyModeEnabled: false,
      }

      mockInvoke
        .mockResolvedValueOnce(config)
        .mockResolvedValueOnce([])

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      // 添加多个活动
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
          timestamp: Date.now() + 1000,
          appName: 'App 2',
          windowTitle: 'Window 2',
          imagePath: '/path/to/image2.png',
        },
      ]

      activities.forEach((activity) => {
        act(() => {
          result.current.dispatch({ type: 'ADD_ACTIVITY', payload: activity })
        })
      })

      expect(result.current.state.activities).toHaveLength(2)

      // 模拟后端通过事件添加新活动
      if (eventHandlers['new-activity']) {
        const newActivity: ActivityLog = {
          id: 3,
          timestamp: Date.now() + 2000,
          appName: 'App 3',
          windowTitle: 'Window 3',
          imagePath: '/path/to/image3.png',
        }

        act(() => {
          eventHandlers['new-activity']({ payload: newActivity })
        })

        await waitFor(() => {
          expect(result.current.state.activities.length).toBeGreaterThanOrEqual(3)
          expect(result.current.state.activities[0]).toEqual(newActivity)
        })
      }
    })
  })

  describe('搜索流程', () => {
    it('应该完成从搜索到更新结果的完整流程', async () => {
      const config: AppConfig = {
        recordingInterval: 5000,
        ocrEnabled: true,
        aiEnabled: false,
        enableFocusAnalytics: false,
        enableProactiveAssistant: false,
        retentionDays: 30,
        blocklistEnabled: false,
        blocklistMode: 'blocklist',
        privacyModeEnabled: false,
      }

      const initialActivities: ActivityLog[] = []
      const searchResults: ActivityLog[] = [
        {
          id: 1,
          timestamp: Date.now(),
          appName: 'Chrome',
          windowTitle: 'Search Result',
          imagePath: '/path/to/image.png',
          ocrText: 'search content',
        },
      ]

      mockInvoke
        .mockResolvedValueOnce(config)
        .mockResolvedValueOnce(initialActivities)
        .mockResolvedValueOnce({ items: searchResults, total: 1 })

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      // 执行搜索
      await act(async () => {
        await result.current.searchActivities({
          query: 'search',
          appName: 'Chrome',
          limit: 10,
        })
      })

      await waitFor(() => {
        expect(result.current.state.activities).toEqual(searchResults)
        expect(result.current.state.lastSearchParams?.query).toBe('search')
        expect(result.current.state.lastSearchParams?.appName).toBe('Chrome')
      })
    })
  })

  describe('配置管理', () => {
    it('应该处理配置加载和更新的完整流程', async () => {
      const initialConfig: AppConfig = {
        recordingInterval: 5000,
        ocrEnabled: true,
        aiEnabled: false,
        enableFocusAnalytics: false,
        enableProactiveAssistant: false,
        retentionDays: 30,
        blocklistEnabled: false,
        blocklistMode: 'blocklist',
        privacyModeEnabled: false,
      }

      const updatedConfig: AppConfig = {
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

      mockInvoke
        .mockResolvedValueOnce(initialConfig)
        .mockResolvedValueOnce([])

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state.config).toEqual(initialConfig)
      })

      // 更新配置
      act(() => {
        result.current.dispatch({ type: 'SET_CONFIG', payload: updatedConfig })
      })

      expect(result.current.state.config).toEqual(updatedConfig)
    })
  })

  describe('视图切换', () => {
    it('应该在不同视图之间切换', async () => {
      const config: AppConfig = {
        recordingInterval: 5000,
        ocrEnabled: true,
        aiEnabled: false,
        enableFocusAnalytics: false,
        enableProactiveAssistant: false,
        retentionDays: 30,
        blocklistEnabled: false,
        blocklistMode: 'blocklist',
        privacyModeEnabled: false,
      }

      mockInvoke
        .mockResolvedValueOnce(config)
        .mockResolvedValueOnce([])

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state.currentView).toBe('timeline')
      })

      // 切换到图形视图
      act(() => {
        result.current.dispatch({ type: 'SET_VIEW', payload: 'graph' })
      })

      expect(result.current.state.currentView).toBe('graph')

      // 切换到问答视图
      act(() => {
        result.current.dispatch({ type: 'SET_VIEW', payload: 'qa' })
      })

      expect(result.current.state.currentView).toBe('qa')
    })
  })

  describe('错误处理', () => {
    it('应该在多个操作失败时正确处理错误', async () => {
      const config: AppConfig = {
        recordingInterval: 5000,
        ocrEnabled: true,
        aiEnabled: false,
        enableFocusAnalytics: false,
        enableProactiveAssistant: false,
        retentionDays: 30,
        blocklistEnabled: false,
        blocklistMode: 'blocklist',
        privacyModeEnabled: false,
      }

      mockInvoke
        .mockResolvedValueOnce(config)
        .mockResolvedValueOnce([])
        .mockRejectedValueOnce(new Error('Start failed'))
        .mockRejectedValueOnce(new Error('Load failed'))
        .mockRejectedValueOnce(new Error('Search failed'))

      const { result } = renderHook(() => useApp(), {
        wrapper: AppProvider,
      })

      await waitFor(() => {
        expect(result.current.state).toBeDefined()
      })

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      // 启动录制失败
      await act(async () => {
        await result.current.startRecording()
      })

      expect(consoleErrorSpy).toHaveBeenCalled()

      // 加载活动失败
      await act(async () => {
        await result.current.loadActivities()
      })

      expect(consoleErrorSpy).toHaveBeenCalledTimes(2)

      // 搜索失败
      await act(async () => {
        await result.current.searchActivities({ query: 'test' })
      })

      expect(consoleErrorSpy).toHaveBeenCalledTimes(3)

      consoleErrorSpy.mockRestore()
    })
  })
})

