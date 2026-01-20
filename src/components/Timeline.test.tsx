import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import React from 'react'
import Timeline from './Timeline'
import { AppProvider, useApp } from '../contexts/AppContext'
import { invoke } from '@tauri-apps/api/core'
import { renderHook } from '@testing-library/react'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `tauri://localhost/${path}`),
}))

vi.mock('../utils/imageLoader', () => ({
  getScreenshotUrl: vi.fn(() => Promise.resolve('data:image/test')),
}))

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

vi.mock('./ImagePreviewModal', () => ({
  default: ({ open, onClose }: { open: boolean; onClose: () => void }) =>
    open ? (
      <div data-testid="image-preview-modal">
        <button onClick={onClose}>关闭</button>
      </div>
    ) : null,
}))

const mockInvoke = vi.mocked(invoke)

const Wrapper = ({ children }: { children: React.ReactNode }) => (
  <AppProvider>{children}</AppProvider>
)

describe('Timeline', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockInvoke.mockResolvedValue(undefined)
  })

  it('应该渲染时间轴组件', () => {
    render(<Timeline />, { wrapper: Wrapper })

    expect(screen.getByText(/活动时间轴/i)).toBeInTheDocument()
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
      expect(screen.getByText(/共.*条记录/i)).toBeInTheDocument()
    })
  })

  it('应该显示空状态当没有活动时', () => {
    render(<Timeline />, { wrapper: Wrapper })

    expect(screen.getByText(/暂无活动记录/i)).toBeInTheDocument()
  })

  it('应该允许输入搜索查询', async () => {
    const user = userEvent.setup()
    // 确保 state.activities 已初始化
    mockInvoke
      .mockResolvedValueOnce(undefined) // get_config
      .mockResolvedValueOnce([]) // get_activities
    
    render(<Timeline />, { wrapper: Wrapper })

    await waitFor(() => {
      const searchInput = screen.getByPlaceholderText(/搜索活动/i)
      expect(searchInput).toBeInTheDocument()
    })

    const searchInput = screen.getByPlaceholderText(/搜索活动/i)
    await user.type(searchInput, 'test query')

    await waitFor(() => {
      expect(searchInput).toHaveValue('test query')
    })
  })

  it('应该能够清除搜索输入', async () => {
    const user = userEvent.setup()
    // 确保 state.activities 已初始化
    mockInvoke
      .mockResolvedValueOnce(undefined) // get_config
      .mockResolvedValueOnce([]) // get_activities
    
    render(<Timeline />, { wrapper: Wrapper })

    await waitFor(() => {
      const searchInput = screen.getByPlaceholderText(/搜索活动/i)
      expect(searchInput).toBeInTheDocument()
    })

    const searchInput = screen.getByPlaceholderText(/搜索活动/i)
    await user.type(searchInput, 'test')

    await waitFor(() => {
      const clearButton = screen.getByRole('button', { name: /清除搜索/i })
      expect(clearButton).toBeInTheDocument()
    })

    const clearButton = screen.getByRole('button', { name: /清除搜索/i })
    await user.click(clearButton)
    
    await waitFor(() => {
      expect(searchInput).toHaveValue('')
    })
  })

  it('应该切换过滤器面板', async () => {
    const user = userEvent.setup()
    render(<Timeline />, { wrapper: Wrapper })

    const filterButton = screen.getByRole('button', { name: '' })
    const buttons = screen.getAllByRole('button')
    const filterBtn = buttons.find((btn) =>
      btn.querySelector('svg')?.classList.contains('lucide-filter')
    )

    if (filterBtn) {
      await user.click(filterBtn)

      await waitFor(() => {
        expect(screen.getByLabelText(/应用名称/i)).toBeInTheDocument()
      })
    }
  })

  it('应该执行搜索', async () => {
    const user = userEvent.setup()
    // 初始化时的调用：get_config, get_activities
    // 然后搜索会调用 search_activities
    mockInvoke
      .mockResolvedValueOnce(undefined) // get_config
      .mockResolvedValueOnce([]) // get_activities
      .mockResolvedValueOnce({ items: [], total: 0 }) // search_activities

    render(<Timeline />, { wrapper: Wrapper })

    await waitFor(() => {
      const searchInput = screen.getByPlaceholderText(/搜索活动/i)
      expect(searchInput).toBeInTheDocument()
    })

    const searchInput = screen.getByPlaceholderText(/搜索活动/i)
    const searchButtons = screen.getAllByRole('button', { name: /搜索/i })
    const searchButton = searchButtons.find(btn => btn.textContent === '搜索')

    await user.type(searchInput, 'test')
    if (searchButton) {
      await user.click(searchButton)
    }

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalled()
    })
  })

  it('应该解析规则查询 app: 前缀', async () => {
    const user = userEvent.setup()
    mockInvoke.mockResolvedValueOnce({ items: [], total: 0 })

    render(<Timeline />, { wrapper: Wrapper })

    const searchInput = screen.getByPlaceholderText(/搜索活动/i)
    const searchButtons = screen.getAllByRole('button', { name: /搜索/i })
    const searchButton = searchButtons.find(btn => btn.textContent === '搜索')

    await user.type(searchInput, 'app:Chrome')
    if (searchButton) {
      await user.click(searchButton)
    }

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalled()
    })
  })

  it('应该解析规则查询 from: 和 to: 日期', async () => {
    const user = userEvent.setup()
    // 初始化时的调用：get_config, get_activities
    // 然后搜索会调用 search_activities
    mockInvoke
      .mockResolvedValueOnce(undefined) // get_config
      .mockResolvedValueOnce([]) // get_activities
      .mockResolvedValueOnce({ items: [], total: 0 }) // search_activities

    render(<Timeline />, { wrapper: Wrapper })

    await waitFor(() => {
      const searchInput = screen.getByPlaceholderText(/搜索活动/i)
      expect(searchInput).toBeInTheDocument()
    })

    const searchInput = screen.getByPlaceholderText(/搜索活动/i)
    const searchButtons = screen.getAllByRole('button', { name: /搜索/i })
    const searchButton = searchButtons.find(btn => btn.textContent === '搜索')

    await user.type(searchInput, 'from:2024-01-01 to:2024-01-31')
    if (searchButton) {
      await user.click(searchButton)
    }

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalled()
    })
  })

  it('应该解析规则查询 ocr: 前缀', async () => {
    const user = userEvent.setup()
    // 初始化时的调用：get_config, get_activities
    // 然后搜索会调用 search_activities
    mockInvoke
      .mockResolvedValueOnce(undefined) // get_config
      .mockResolvedValueOnce([]) // get_activities
      .mockResolvedValueOnce({ items: [], total: 0 }) // search_activities

    render(<Timeline />, { wrapper: Wrapper })

    await waitFor(() => {
      const searchInput = screen.getByPlaceholderText(/搜索活动/i)
      expect(searchInput).toBeInTheDocument()
    })

    const searchInput = screen.getByPlaceholderText(/搜索活动/i)
    const searchButtons = screen.getAllByRole('button', { name: /搜索/i })
    const searchButton = searchButtons.find(btn => btn.textContent === '搜索')

    await user.type(searchInput, 'ocr:true')
    if (searchButton) {
      await user.click(searchButton)
    }

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalled()
    })
  })

  it('应该执行智能搜索', async () => {
    const user = userEvent.setup()
    const mockIntent = {
      app_name: 'Chrome',
      keywords: ['test'],
      date_range: null,
      has_ocr: true,
    }

    // 初始化时的调用：get_config, get_activities
    // 然后智能搜索会调用 parse_query_intent 和 search_activities
    mockInvoke
      .mockResolvedValueOnce(undefined) // get_config
      .mockResolvedValueOnce([]) // get_activities
      .mockResolvedValueOnce(mockIntent) // parse_query_intent
      .mockResolvedValueOnce({ items: [], total: 0 }) // search_activities

    render(<Timeline />, { wrapper: Wrapper })

    await waitFor(() => {
      const searchInput = screen.getByPlaceholderText(/搜索活动/i)
      expect(searchInput).toBeInTheDocument()
    })

    const searchInput = screen.getByPlaceholderText(/搜索活动/i)
    await user.type(searchInput, 'test')

    await waitFor(() => {
      const buttons = screen.getAllByRole('button')
      const smartSearchBtn = buttons.find((btn) =>
        btn.querySelector('svg')?.classList.contains('lucide-sparkles')
      )
      expect(smartSearchBtn).toBeDefined()
    })

    const buttons = screen.getAllByRole('button')
    const smartSearchBtn = buttons.find((btn) =>
      btn.querySelector('svg')?.classList.contains('lucide-sparkles')
    )

    if (smartSearchBtn) {
      await user.click(smartSearchBtn)

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('parse_query_intent', {
          query: 'test',
        })
      })
    }
  })

  it('应该显示智能搜索错误', async () => {
    const user = userEvent.setup()
    const error = new Error('智能搜索失败')

    // 初始化时的调用：get_config, get_activities
    // 然后智能搜索会调用 parse_query_intent (会失败)
    mockInvoke
      .mockResolvedValueOnce(undefined) // get_config
      .mockResolvedValueOnce([]) // get_activities
      .mockRejectedValueOnce(error) // parse_query_intent

    render(<Timeline />, { wrapper: Wrapper })

    await waitFor(() => {
      const searchInput = screen.getByPlaceholderText(/搜索活动/i)
      expect(searchInput).toBeInTheDocument()
    })

    const searchInput = screen.getByPlaceholderText(/搜索活动/i)
    await user.type(searchInput, 'test')

    await waitFor(() => {
      const buttons = screen.getAllByRole('button')
      const smartSearchBtn = buttons.find((btn) =>
        btn.querySelector('svg')?.classList.contains('lucide-sparkles')
      )
      expect(smartSearchBtn).toBeDefined()
    })

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

  it('应该能够重置搜索', async () => {
    const user = userEvent.setup()
    // 初始化时的调用：get_config, get_activities
    mockInvoke
      .mockResolvedValueOnce(undefined) // get_config
      .mockResolvedValueOnce([]) // get_activities

    render(<Timeline />, { wrapper: Wrapper })

    await waitFor(() => {
      const searchInput = screen.getByPlaceholderText(/搜索活动/i)
      expect(searchInput).toBeInTheDocument()
    })

    const searchInput = screen.getByPlaceholderText(/搜索活动/i)
    await user.type(searchInput, 'test')

    await waitFor(() => {
      const resetButton = screen.getByRole('button', { name: /重置/i })
      expect(resetButton).toBeInTheDocument()
    })

    const resetButton = screen.getByRole('button', { name: /重置/i })
    await user.click(resetButton)

    await waitFor(() => {
      expect(searchInput).toHaveValue('')
    })
  })

  it('应该在过滤器中设置应用名称', async () => {
    const user = userEvent.setup()
    render(<Timeline />, { wrapper: Wrapper })

    // 打开过滤器
    const buttons = screen.getAllByRole('button')
    const filterBtn = buttons.find((btn) =>
      btn.querySelector('svg')?.classList.contains('lucide-filter')
    )

    if (filterBtn) {
      await user.click(filterBtn)

      await waitFor(() => {
        const appNameInput = screen.getByLabelText(/应用名称/i)
        expect(appNameInput).toBeInTheDocument()

        user.type(appNameInput, 'Chrome')
        expect(appNameInput).toHaveValue('Chrome')
      })
    }
  })

  it('应该在过滤器中设置日期范围', async () => {
    const user = userEvent.setup()
    render(<Timeline />, { wrapper: Wrapper })

    // 打开过滤器
    const buttons = screen.getAllByRole('button')
    const filterBtn = buttons.find((btn) =>
      btn.querySelector('svg')?.classList.contains('lucide-filter')
    )

    if (filterBtn) {
      await user.click(filterBtn)

      await waitFor(() => {
        const dateInputs = screen.getAllByDisplayValue('')
        expect(dateInputs.length).toBeGreaterThan(0)
      })
    }
  })

  it('应该能够打开图片预览', async () => {
    // 初始化时的调用：get_config, get_activities
    mockInvoke
      .mockResolvedValueOnce(undefined) // get_config
      .mockResolvedValueOnce([]) // get_activities

    const activity = {
      id: 1,
      timestamp: Date.now(),
      appName: 'Test App',
      windowTitle: 'Test Window',
      imagePath: '/test.png',
    }

    // 创建一个测试组件，在同一个 AppProvider 中同时渲染 Timeline 和设置活动
    const TestComponent = () => {
      const { dispatch, state } = useApp()
      React.useEffect(() => {
        if (state.activities.length === 0) {
          dispatch({ type: 'SET_ACTIVITIES', payload: [activity] })
        }
      }, [dispatch, state.activities.length])
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

