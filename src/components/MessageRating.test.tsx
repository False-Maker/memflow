import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import MessageRating from './MessageRating'
import { invoke } from '@tauri-apps/api/core'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('MessageRating', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('应该在没有 messageId 时不渲染', () => {
    const { container } = render(<MessageRating messageId={undefined} />)
    expect(container.firstChild).toBeNull()
  })

  it('应该渲染评价按钮', () => {
    render(<MessageRating messageId={1} />)
    
    expect(screen.getByText('有帮助')).toBeInTheDocument()
    expect(screen.getByText('无帮助')).toBeInTheDocument()
  })

  it('应该显示当前评价状态', () => {
    render(<MessageRating messageId={1} currentRating={1} />)
    
    const thumbsUpButton = screen.getByText('有帮助').closest('button')
    expect(thumbsUpButton).toHaveClass('bg-green-500/20', 'text-green-400')
  })

  it('应该能够点击评价按钮', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined)
    
    render(<MessageRating messageId={1} />)
    
    const thumbsUpButton = screen.getByText('有帮助').closest('button')
    fireEvent.click(thumbsUpButton!)
    
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('rate_message', {
        messageId: 1,
        rating: 1,
        comment: null,
      })
    })
  })

  it('应该能够切换评价', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined)
    
    const onRatingChange = vi.fn()
    render(<MessageRating messageId={1} currentRating={1} onRatingChange={onRatingChange} />)
    
    const thumbsUpButton = screen.getByText('有帮助').closest('button')
    fireEvent.click(thumbsUpButton!)
    
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('rate_message', {
        messageId: 1,
        rating: 0, // 点击已选中的评价，应该取消评价
        comment: null,
      })
      expect(onRatingChange).toHaveBeenCalledWith(null)
    })
  })

  it('应该能够从无评价切换到有帮助', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined)
    
    const onRatingChange = vi.fn()
    render(<MessageRating messageId={1} onRatingChange={onRatingChange} />)
    
    const thumbsUpButton = screen.getByText('有帮助').closest('button')
    fireEvent.click(thumbsUpButton!)
    
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('rate_message', {
        messageId: 1,
        rating: 1,
        comment: null,
      })
      expect(onRatingChange).toHaveBeenCalledWith(1)
    })
  })

  it('应该能够从无评价切换到无帮助', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined)
    
    const onRatingChange = vi.fn()
    render(<MessageRating messageId={1} onRatingChange={onRatingChange} />)
    
    const thumbsDownButton = screen.getByText('无帮助').closest('button')
    fireEvent.click(thumbsDownButton!)
    
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('rate_message', {
        messageId: 1,
        rating: -1,
        comment: null,
      })
      expect(onRatingChange).toHaveBeenCalledWith(-1)
    })
  })

  it('应该在加载时禁用按钮', async () => {
    vi.mocked(invoke).mockImplementation(() => 
      new Promise(resolve => setTimeout(() => resolve(undefined), 100))
    )
    
    render(<MessageRating messageId={1} />)
    
    const thumbsUpButton = screen.getByText('有帮助').closest('button')
    fireEvent.click(thumbsUpButton!)
    
    // 按钮应该在加载时被禁用
    await waitFor(() => {
      expect(thumbsUpButton).toHaveClass('opacity-50', 'cursor-not-allowed')
    })
  })

  it('应该处理评价失败的错误', async () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    vi.mocked(invoke).mockRejectedValue(new Error('评价失败'))
    
    render(<MessageRating messageId={1} />)
    
    const thumbsUpButton = screen.getByText('有帮助').closest('button')
    fireEvent.click(thumbsUpButton!)
    
    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalledWith('评价失败:', expect.any(Error))
    })
    
    consoleErrorSpy.mockRestore()
  })
})

