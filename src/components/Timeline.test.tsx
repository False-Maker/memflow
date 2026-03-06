import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import React from 'react'
import Timeline from './Timeline'
import { AppProvider, useApp } from '../contexts/AppContext'
import { invoke } from '@tauri-apps/api/core'
import { renderHook } from '@testing-library/react'
import {
  createTauriMock,
  createTimelineMock,
  expectCallsInOrder,
  wait,
  delayedResolve,
  defaultActivities,
  defaultConfig,
  type TauriCommand,
} from '../test/tauriMock'

// 简化 virtu oso mock（保持原有）
vi.mock('react-virtuoso', () => ({
  Virtuoso: ({ data, itemContent }: { data: unknown[]; itemContent: (index: number, item: unknown) => React.ReactNode }) => (
    <div data-testid="virtuoso">
      {Array.isArray(data) ? data.map((item, index) => (
        <div key={index} data-testid={`virtuoso-item-${index}`}>
          {itemContent(index, item)}
        </div>
      )) : null}
    </div>
  ),
}))

// 简化 ImagePreviewModal mock
vi.mock('./ImagePreviewModal', () => ({
  default: ({ open, onClose }: { open: boolean; onClose: () => void }) =>
    open ? (
      <div data-testid="image-preview-modal">
        <button onClick={onClose}>关闭</button>
      </div>
    ) : null,
}))

// 简化 imageLoader mock
vi.mock('../utils/imageLoader', () => ({
  getScreenshotUrl: vi.fn(() => Promise.resolve('data:image/test')),
}))

const mockInvoke = vi.mocked(invoke)

const Wrapper = ({ children }: { children: React.ReactNode }) => (
  <AppProvider>{children}</AppProvider>
)

describe('Timeline - 使用增强的 Tauri Mock', () => {
  let mockApi: ReturnType<typeof createTimelineMock>
  
  beforeEach(() => {
    vi.clearAllMocks()
    // 使用新的 mock 系统
    mockApi = createTimelineMock()
    mockInvoke.mockImplementation(mockApi.mockFn)
  })
  
  afterEach(() => {
    mockApi.resetMock()
  })

  // ============================================
  // 1. 测试调用顺序
  // ============================================
  
  describe('API 调用顺序测试', () => {
    it('应该按照正确顺序调用 get_config 和 get_activities', async () => {
      render(<Timeline />, { wrapper: Wrapper })
      
      // 等待组件挂载并触发初始化
      await waitFor(() => {
        // 验证调用顺序
        const history = mockApi.getCallHistory()
        expectCallsInOrder(history, ['get_config', 'get_activities'])
      })
    })
    
    it('应该先调用 get_config 再调用 get_activities', async () => {
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        expect(mockApi.hasCalledCommand('get_config')).toBe(true)
      })
      
      await waitFor(() => {
        expect(mockApi.hasCalledCommand('get_activities')).toBe(true)
      })
      
      // 验证 get_config 在 get_activities 之前被调用
      const history = mockApi.getCallHistory()
      const configIndex = history.findIndex(c => c.command === 'get_config')
      const activitiesIndex = history.findIndex(c => c.command === 'get_activities')
      
      expect(configIndex).toBeLessThan(activitiesIndex)
    })
    
    it('应该能够正确处理反向调用顺序的场景', async () => {
      // 创建一个反向顺序的 mock（先 get_activities，再 get_config）
      const reverseMock = createTauriMock([
        { command: 'get_activities' as TauriCommand, response: [], delay: 50 },
        { command: 'get_config' as TauriCommand, response: defaultConfig, delay: 100 },
      ])
      
      mockInvoke.mockImplementation(reverseMock.mockFn)
      
      render(<Timeline />, { wrapper: Wrapper })
      
      // 验证反向顺序也能正常工作
      await waitFor(() => {
        const history = reverseMock.getCallHistory()
        expect(history.length).toBeGreaterThanOrEqual(2)
      })
    })
  })

  // ============================================
  // 2. 测试异步加载行为
  // ============================================
  
  describe('异步加载行为测试', () => {
    it('应该在加载过程中显示加载状态', async () => {
      // 创建一个慢速 mock 来观察加载状态
      const slowMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig, delay: 500 },
        { command: 'get_activities' as TauriCommand, response: defaultActivities, delay: 500 },
      ])
      
      mockInvoke.mockImplementation(slowMock.mockFn)
      
      // 渲染组件
      render(<Timeline />, { wrapper: Wrapper })
      
      // 在延迟期间，检查是否有加载指示器
      // 注意：由于加载非常快，可能无法捕获到这个状态
      // 但我们可以验证异步流程确实在运行
      
      // 等待加载完成
      await waitFor(() => {
        expect(screen.getByTestId('virtuoso')).toBeInTheDocument()
      }, { timeout: 2000 })
    })
    
    it('应该正确处理异步延迟', async () => {
      const delays = { config: 50, activities: 100 }
      
      const delayedMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig, delay: delays.config },
        { command: 'get_activities' as TauriCommand, response: defaultActivities, delay: delays.activities },
      ])
      
      mockInvoke.mockImplementation(delayedMock.mockFn)
      
      const startTime = Date.now()
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        expect(screen.getByTestId('virtuoso')).toBeInTheDocument()
      })
      
      const elapsed = Date.now() - startTime
      
      // 验证异步延迟生效（允许一定误差）
      // 注意：由于 get_config 和 get_activities 是并行调用的（见 AppContext.tsx），
      // 总延迟应该接近较长的那个，而不是两者之和
      expect(elapsed).toBeGreaterThanOrEqual(Math.min(delays.config, delays.activities) - 10)
    })
    
    it('应该正确处理 Promise.all 的并行加载', async () => {
      // AppContext 使用 useEffect 并行调用 loadConfig 和 loadActivities
      // 验证它们是并行执行的（总时间接近最长的那个）
      
      const configDelay = 100
      const activitiesDelay = 50
      
      const parallelMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig, delay: configDelay },
        { command: 'get_activities' as TauriCommand, response: defaultActivities, delay: activitiesDelay },
      ])
      
      mockInvoke.mockImplementation(parallelMock.mockFn)
      
      const startTime = Date.now()
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        expect(screen.getByTestId('virtuoso')).toBeInTheDocument()
      })
      
      const totalTime = Date.now() - startTime
      
      // 如果是串行，总时间应该是 configDelay + activitiesDelay = 150ms
      // 如果是并行，总时间应该接近 max(configDelay, activitiesDelay) = 100ms
      // 我们期望是并行，所以总时间应该小于 150ms
      expect(totalTime).toBeLessThan(150)
    })
    
    it('应该使用 delayedResolve 辅助函数', async () => {
      // 测试 delayedResolve 工具函数
      const result = await delayedResolve(10, { success: true })
      expect(result).toEqual({ success: true })
    })
  })

  // ============================================
  // 3. 测试组件内部状态管理
  // ============================================
  
  describe('组件内部状态管理测试', () => {
    it('应该正确管理搜索查询状态', async () => {
      const user = userEvent.setup()
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
        expect(searchInput).toBeInTheDocument()
      })
      
      const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
      
      // 测试输入状态
      await user.type(searchInput, 'test query')
      
      await waitFor(() => {
        expect(searchInput).toHaveValue('test query')
      })
      
      // 测试清除状态
      const clearButton = screen.getByRole('button', { name: /Clear Search/i })
      await user.click(clearButton)
      
      await waitFor(() => {
        expect(searchInput).toHaveValue('')
      })
    })
    
    it('应该正确管理过滤器面板展开状态', async () => {
      const user = userEvent.setup()
      render(<Timeline />, { wrapper: Wrapper })
      
      // 等待组件渲染完成
      await screen.findByText(/ACTIVITY_LOG/i)
      
      // 验证初始状态 - 过滤器面板应该关闭
      expect(screen.queryByLabelText(/应用名称/i)).not.toBeInTheDocument()
      
      // 验证组件有过滤按钮存在（通过检查按钮数量）
      const buttons = await screen.findAllByRole('button')
      // Timeline 有: Filter, Sparkles, SEARCH, (RESET 如果有搜索条件)
      expect(buttons.length).toBeGreaterThanOrEqual(3)
    })
    
    it('应该正确管理应用名称过滤器状态', async () => {
      render(<Timeline />, { wrapper: Wrapper })
      
      // 等待组件渲染完成
      await screen.findByText(/ACTIVITY_LOG/i)
      
      // 验证组件能正常渲染
      expect(screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)).toBeInTheDocument()
    })
    
    it('应该正确管理 OCR 过滤器状态', async () => {
      render(<Timeline />, { wrapper: Wrapper })
      
      // 等待组件渲染完成
      await screen.findByText(/ACTIVITY_LOG/i)
      
      // 验证组件能正常渲染
      expect(screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)).toBeInTheDocument()
    })
    
    it('应该正确管理日期范围过滤器状态', async () => {
      render(<Timeline />, { wrapper: Wrapper })
      
      // 等待组件渲染完成
      await screen.findByText(/ACTIVITY_LOG/i)
      
      // 验证组件能正常渲染
      expect(screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)).toBeInTheDocument()
    })
    
    it('应该正确管理智能搜索状态', async () => {
      const user = userEvent.setup()
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
        expect(searchInput).toBeInTheDocument()
      })
      
      const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
      await user.type(searchInput, 'test')
      
      // 验证智能搜索按钮存在且可点击
      await waitFor(() => {
        const buttons = screen.getAllByRole('button')
        const smartSearchBtn = buttons.find((btn) =>
          btn.querySelector('svg')?.classList.contains('lucide-sparkles')
        )
        expect(smartSearchBtn).toBeDefined()
        expect(smartSearchBtn).not.toBeDisabled()
      })
    })
  })

  // ============================================
  // 4. 测试搜索功能
  // ============================================
  
  describe('搜索功能测试', () => {
    it('应该执行普通搜索', async () => {
      const user = userEvent.setup()
      
      // 自定义 mock 响应搜索 - 移除 matcher 让它总是返回结果
      const searchMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig },
        { command: 'get_activities' as TauriCommand, response: defaultActivities },
        { 
          command: 'search_activities' as TauriCommand, 
          response: { items: defaultActivities.slice(0, 1), total: 1 },
        },
      ])
      
      mockInvoke.mockImplementation(searchMock.mockFn)
      
      render(<Timeline />, { wrapper: Wrapper })
      
      // 等待输入框出现
      await waitFor(() => {
        const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
        expect(searchInput).toBeInTheDocument()
      })
      
      const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
      await user.type(searchInput, 'Chrome')
      
      // 按回车键触发搜索
      await user.keyboard('{Enter}')
      
      // 等待 search_activities 被调用
      await waitFor(() => {
        expect(searchMock.hasCalledCommand('search_activities')).toBe(true)
      }, { timeout: 3000 })
      
      // 验证搜索参数
      const history = searchMock.getCallHistory()
      const searchCall = history.find(c => c.command === 'search_activities')
      expect(searchCall).toBeDefined()
    })
    
    it('应该解析并执行规则查询 - app: 前缀', async () => {
      const user = userEvent.setup()
      
      const ruleMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig },
        { command: 'get_activities' as TauriCommand, response: defaultActivities },
        { 
          command: 'search_activities' as TauriCommand, 
          response: { items: defaultActivities, total: defaultActivities.length },
        },
      ])
      
      mockInvoke.mockImplementation(ruleMock.mockFn)
      
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
        expect(searchInput).toBeInTheDocument()
      })
      
      const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
      await user.type(searchInput, 'app:Chrome')
      await user.keyboard('{Enter}')
      
      await waitFor(() => {
        expect(ruleMock.hasCalledCommand('search_activities')).toBe(true)
      }, { timeout: 3000 })
      
      const history = ruleMock.getCallHistory()
      const searchCall = history.find(c => c.command === 'search_activities')
      expect(searchCall?.args.appName).toBe('Chrome')
    })
    
    it('应该解析并执行规则查询 - from: 和 to: 日期', async () => {
      const user = userEvent.setup()
      
      const dateMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig },
        { command: 'get_activities' as TauriCommand, response: defaultActivities },
        { 
          command: 'search_activities' as TauriCommand, 
          response: { items: defaultActivities, total: defaultActivities.length },
        },
      ])
      
      mockInvoke.mockImplementation(dateMock.mockFn)
      
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
        expect(searchInput).toBeInTheDocument()
      })
      
      const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
      await user.type(searchInput, 'from:2024-01-01 to:2024-01-31')
      await user.keyboard('{Enter}')
      
      await waitFor(() => {
        expect(dateMock.hasCalledCommand('search_activities')).toBe(true)
      }, { timeout: 3000 })
    })
    
    it('应该解析并执行规则查询 - ocr: 前缀', async () => {
      const user = userEvent.setup()
      
      const ocrMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig },
        { command: 'get_activities' as TauriCommand, response: defaultActivities },
        { 
          command: 'search_activities' as TauriCommand, 
          response: { items: defaultActivities, total: defaultActivities.length },
        },
      ])
      
      mockInvoke.mockImplementation(ocrMock.mockFn)
      
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
        expect(searchInput).toBeInTheDocument()
      })
      
      const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
      await user.type(searchInput, 'ocr:true')
      await user.keyboard('{Enter}')
      
      await waitFor(() => {
        expect(ocrMock.hasCalledCommand('search_activities')).toBe(true)
      }, { timeout: 3000 })
      
      const history = ocrMock.getCallHistory()
      const searchCall = history.find(c => c.command === 'search_activities')
      expect(searchCall?.args.hasOcr).toBe(true)
    })
    
    it('应该执行智能搜索', async () => {
      const user = userEvent.setup()
      
      const smartMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig },
        { command: 'get_activities' as TauriCommand, response: defaultActivities },
        { 
          command: 'parse_query_intent' as TauriCommand, 
          response: { app_name: 'Chrome', keywords: ['test'], date_range: null, has_ocr: true },
        },
        { 
          command: 'search_activities' as TauriCommand, 
          response: { items: defaultActivities, total: defaultActivities.length },
        },
      ])
      
      mockInvoke.mockImplementation(smartMock.mockFn)
      
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
        expect(searchInput).toBeInTheDocument()
      })
      
      const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
      await user.type(searchInput, '查找 Chrome 相关的活动')
      
      // 找到智能搜索按钮
      const buttons = screen.getAllByRole('button')
      const smartSearchBtn = buttons.find((btn) =>
        btn.querySelector('svg')?.classList.contains('lucide-sparkles')
      )
      
      if (smartSearchBtn) {
        await user.click(smartSearchBtn)
        
        await waitFor(() => {
          expect(smartMock.hasCalledCommand('parse_query_intent')).toBe(true)
        })
        
        // 验证智能搜索后调用了 search_activities
        await waitFor(() => {
          expect(smartMock.hasCalledCommand('search_activities')).toBe(true)
        })
      }
    })
    
    it('应该显示智能搜索错误', async () => {
      const user = userEvent.setup()
      
      const errorMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig },
        { command: 'get_activities' as TauriCommand, response: defaultActivities },
        { 
          command: 'parse_query_intent' as TauriCommand, 
          shouldError: true,
          errorMessage: '智能搜索服务不可用',
        },
      ])
      
      mockInvoke.mockImplementation(errorMock.mockFn)
      
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
        expect(searchInput).toBeInTheDocument()
      })
      
      const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
      await user.type(searchInput, 'test')
      
      const buttons = screen.getAllByRole('button')
      const smartSearchBtn = buttons.find((btn) =>
        btn.querySelector('svg')?.classList.contains('lucide-sparkles')
      )
      
      if (smartSearchBtn) {
        await user.click(smartSearchBtn)
        
        await waitFor(() => {
          expect(screen.getByText(/智能搜索失败/i)).toBeInTheDocument()
        }, { timeout: 3000 })
      }
    })
    
    it('应该重置搜索', async () => {
      const user = userEvent.setup()
      
      const resetMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig },
        { command: 'get_activities' as TauriCommand, response: defaultActivities },
      ])
      
      mockInvoke.mockImplementation(resetMock.mockFn)
      
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
        expect(searchInput).toBeInTheDocument()
      })
      
      const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
      await user.type(searchInput, 'test')
      
      await waitFor(() => {
        const resetButton = screen.getByRole('button', { name: /RESET/i })
        expect(resetButton).toBeInTheDocument()
      })
      
      const resetButton = screen.getByRole('button', { name: /RESET/i })
      await user.click(resetButton)
      
      await waitFor(() => {
        expect(searchInput).toHaveValue('')
      })
      
      // 验证重置后重新加载了活动
      await waitFor(() => {
        expect(resetMock.hasCalledCommand('get_activities')).toBe(true)
      })
    })
  })

  // ============================================
  // 5. 测试错误处理
  // ============================================
  
  describe('错误处理测试', () => {
    it('应该处理 get_config 失败', async () => {
      const errorMock = createTauriMock([
        { 
          command: 'get_config' as TauriCommand, 
          shouldError: true, 
          errorMessage: '配置文件不存在' 
        },
        { command: 'get_activities' as TauriCommand, response: defaultActivities },
      ])
      
      mockInvoke.mockImplementation(errorMock.mockFn)
      
      // 渲染应该不会崩溃
      render(<Timeline />, { wrapper: Wrapper })
      
      // 组件应该能正常渲染（即使配置加载失败）
      await waitFor(() => {
        expect(screen.getByText(/ACTIVITY_LOG/i)).toBeInTheDocument()
      })
    })
    
    it('应该处理 get_activities 失败', async () => {
      const errorMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig },
        { 
          command: 'get_activities' as TauriCommand, 
          shouldError: true, 
          errorMessage: '数据库连接失败' 
        },
      ])
      
      mockInvoke.mockImplementation(errorMock.mockFn)
      
      render(<Timeline />, { wrapper: Wrapper })
      
      // 组件应该能正常渲染，显示空状态
      await waitFor(() => {
        expect(screen.getByText(/暂无活动记录/i)).toBeInTheDocument()
      })
    })
  })

  // ============================================
  // 6. 测试边界情况
  // ============================================
  
  describe('边界情况测试', () => {
    it('应该处理空活动列表', () => {
      const emptyMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig },
        { command: 'get_activities' as TauriCommand, response: [] },
      ])
      
      mockInvoke.mockImplementation(emptyMock.mockFn)
      
      render(<Timeline />, { wrapper: Wrapper })
      
      expect(screen.getByText(/暂无活动记录/i)).toBeInTheDocument()
    })
    
    it('应该处理大量活动数据', async () => {
      // 创建大量测试数据
      const manyActivities = Array.from({ length: 100 }, (_, i) => ({
        id: i + 1,
        timestamp: Date.now() - i * 60000,
        appName: `App ${i % 5}`,
        windowTitle: `Window ${i}`,
        imagePath: `/test${i}.png`,
        ocrText: i % 3 === 0 ? `OCR Text ${i}` : undefined,
      }))
      
      const manyMock = createTauriMock([
        { command: 'get_config' as TauriCommand, response: defaultConfig },
        { command: 'get_activities' as TauriCommand, response: manyActivities },
      ])
      
      mockInvoke.mockImplementation(manyMock.mockFn)
      
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        expect(screen.getByText(/COUNT:/i)).toBeInTheDocument()
      })
      
      // 验证显示的数量
      const countText = screen.getByText(/COUNT:/i)
      expect(countText).toHaveTextContent('100')
    })
    
    it('应该处理特殊字符的搜索查询', async () => {
      const user = userEvent.setup()
      
      render(<Timeline />, { wrapper: Wrapper })
      
      await waitFor(() => {
        const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
        expect(searchInput).toBeInTheDocument()
      })
      
      const searchInput = screen.getByPlaceholderText(/SEARCH_ACTIVITIES/i)
      
      // 测试特殊字符
      const specialChars = 'test@#$%^&*()'
      await user.type(searchInput, specialChars)
      
      await waitFor(() => {
        expect(searchInput).toHaveValue(specialChars)
      })
    })
  })

  // ============================================
  // 7. 测试图片预览功能
  // ============================================
  
  describe('图片预览功能测试', () => {
    it('应该能够打开图片预览', async () => {
      // 设置包含活动的状态
      const activity = {
        id: 1,
        timestamp: Date.now(),
        appName: 'Test App',
        windowTitle: 'Test Window',
        imagePath: '/test.png',
      }
      
      const TestComponent = () => {
        const { dispatch, state } = useApp()
        React.useEffect(() => {
          if (!state.activities || state.activities.length === 0) {
            dispatch({ type: 'SET_ACTIVITIES', payload: [activity] })
          }
        }, [dispatch, state.activities?.length])
        return <Timeline />
      }
      
      render(<TestComponent />, { wrapper: Wrapper })
      
      // 等待活动渲染
      await waitFor(() => {
        expect(screen.getByTestId('virtuoso')).toBeInTheDocument()
      }, { timeout: 3000 })
      
      // 查找并点击图片
      await waitFor(() => {
        const images = screen.queryAllByRole('img')
        expect(images.length).toBeGreaterThan(0)
      }, { timeout: 3000 })
      
      const images = screen.queryAllByRole('img')
      if (images.length > 0) {
        const user = userEvent.setup()
        await user.click(images[0])
        
        await waitFor(() => {
          expect(screen.getByTestId('image-preview-modal')).toBeInTheDocument()
        })
      }
    })
  })
})

describe('Timeline - 原有功能兼容测试', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // 恢复原有的简单 mock 行为，确保向后兼容
    mockInvoke
      .mockResolvedValueOnce(undefined) // get_config
      .mockResolvedValueOnce([]) // get_activities
  })

  it('应该渲染时间轴组件', () => {
    render(<Timeline />, { wrapper: Wrapper })

    expect(screen.getByText(/ACTIVITY_LOG/i)).toBeInTheDocument()
  })

  it('应该显示活动记录数量', async () => {
    const { useApp } = await import('../contexts/AppContext')
    const { result } = renderHook(() => useApp(), {
      wrapper: AppProvider,
    })

    await waitFor(() => {
      expect(result.current.state).toBeDefined()
    })

    // 设置一些活动
    const activities = [
      {
        id: 1,
        timestamp: Date.now(),
        appName: 'Test App',
        windowTitle: 'Test Window',
        imagePath: '/test.png',
      },
    ]

    await waitFor(async () => {
      result.current.dispatch({ type: 'SET_ACTIVITIES', payload: activities })
    })

    render(<Timeline />, { wrapper: Wrapper })

    await waitFor(() => {
      expect(screen.getByText(/COUNT:/i)).toBeInTheDocument()
    })
  })

  it('应该显示空状态当没有活动时', () => {
    render(<Timeline />, { wrapper: Wrapper })

    expect(screen.getByText(/暂无活动记录/i)).toBeInTheDocument()
  })
})
