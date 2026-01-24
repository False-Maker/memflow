import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import QnA from './QnA'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `tauri://localhost/${path}`),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

vi.mock('../utils/imageLoader', () => ({
  getScreenshotUrl: vi.fn(() => Promise.resolve('data:image/test')),
}))

const mockInvoke = vi.mocked(invoke)
const mockListen = vi.mocked(listen)

const Wrapper = ({ children }: { children: React.ReactNode }) => children

describe('QnA', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockListen.mockResolvedValue(() => {})
  })

  it('应该渲染欢迎消息', () => {
    mockInvoke.mockResolvedValue(undefined)

    render(<QnA />, { wrapper: Wrapper })

    expect(screen.getByText(/你可以在这里提问/i)).toBeInTheDocument()
  })

  it('应该允许用户输入问题', async () => {
    const user = userEvent.setup()
    mockInvoke.mockResolvedValue(undefined)

    render(<QnA />, { wrapper: Wrapper })

    const textarea = screen.getByPlaceholderText(/输入你的问题/i)
    await user.type(textarea, '这是一个测试问题')

    expect(textarea).toHaveValue('这是一个测试问题')
  })

  it('应该发送消息并显示回复', async () => {
    const user = userEvent.setup()
    mockListen.mockImplementationOnce(async (eventName: string, handler: (event: { payload: unknown }) => void) => {
      if (eventName === 'ai-chat-chunk') {
        handler({ payload: '这是AI回复' })
      }
      return () => {}
    })
    mockInvoke
      .mockResolvedValueOnce(1) // create_chat_session
      .mockResolvedValueOnce(2) // save_chat_message (user)
      .mockResolvedValueOnce(undefined) // ai_chat_stream
      .mockResolvedValueOnce(3) // save_chat_message (assistant)

    render(<QnA />, { wrapper: Wrapper })

    const textarea = screen.getByPlaceholderText(/输入你的问题/i)
    const sendButton = screen.getByRole('button', { name: /发送/i })

    await user.type(textarea, '测试问题')
    await user.click(sendButton)

    await waitFor(() => {
      expect(screen.getByText('测试问题')).toBeInTheDocument()
    }, { timeout: 3000 })

    await waitFor(() => {
      expect(screen.getByText('这是AI回复')).toBeInTheDocument()
    }, { timeout: 3000 })

    expect(mockInvoke).toHaveBeenCalledWith('create_chat_session', {
      title: '测试问题',
    })
    expect(mockInvoke).toHaveBeenCalledWith('save_chat_message', {
      sessionId: 1,
      role: 'user',
      content: '测试问题',
      contextIds: null,
    })
    expect(mockInvoke).toHaveBeenCalledWith('ai_chat_stream', { query: '测试问题' })
  })

  it('应该在发送后清空输入框', async () => {
    const user = userEvent.setup()
    mockListen.mockImplementationOnce(async (eventName: string, handler: (event: { payload: unknown }) => void) => {
      if (eventName === 'ai-chat-chunk') {
        handler({ payload: '回复' })
      }
      return () => {}
    })
    mockInvoke
      .mockResolvedValueOnce(1)
      .mockResolvedValueOnce(2)
      .mockResolvedValueOnce(undefined) // ai_chat_stream
      .mockResolvedValueOnce(3)

    render(<QnA />, { wrapper: Wrapper })

    const textarea = screen.getByPlaceholderText(/输入你的问题/i)
    const sendButton = screen.getByRole('button', { name: /发送/i })

    await user.type(textarea, '测试问题')
    await user.click(sendButton)

    await waitFor(() => {
      expect(textarea).toHaveValue('')
    })
  })

  it('应该禁用发送按钮当输入为空时', () => {
    mockInvoke.mockResolvedValue(undefined)

    render(<QnA />, { wrapper: Wrapper })

    const sendButton = screen.getByRole('button', { name: /发送/i })
    expect(sendButton).toBeDisabled()
  })

  it('应该允许使用 Enter 键发送消息', async () => {
    const user = userEvent.setup()
    mockListen.mockImplementationOnce(async (eventName: string, handler: (event: { payload: unknown }) => void) => {
      if (eventName === 'ai-chat-chunk') {
        handler({ payload: '回复' })
      }
      return () => {}
    })
    mockInvoke
      .mockResolvedValueOnce(1)
      .mockResolvedValueOnce(2)
      .mockResolvedValueOnce(undefined) // ai_chat_stream
      .mockResolvedValueOnce(3)

    render(<QnA />, { wrapper: Wrapper })

    const textarea = screen.getByPlaceholderText(/输入你的问题/i)

    await user.type(textarea, '测试问题{Enter}')

    await waitFor(() => {
      expect(screen.getByText('测试问题')).toBeInTheDocument()
    })
  })

  it('应该允许使用 Shift+Enter 换行', async () => {
    const user = userEvent.setup()
    mockInvoke.mockResolvedValue(undefined)

    render(<QnA />, { wrapper: Wrapper })

    const textarea = screen.getByPlaceholderText(/输入你的问题/i)

    await user.type(textarea, '第一行{Shift>}{Enter}{/Shift}第二行')

    expect(textarea).toHaveValue('第一行\n第二行')
  })

  it('应该在发送失败时显示错误消息', async () => {
    const user = userEvent.setup()
    const error = new Error('发送失败')
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    mockInvoke
      .mockResolvedValueOnce(1) // create_chat_session
      .mockResolvedValueOnce(2) // save_chat_message (user)
      .mockRejectedValueOnce(error) // ai_chat_stream

    render(<QnA />, { wrapper: Wrapper })

    const textarea = screen.getByPlaceholderText(/输入你的问题/i)
    const sendButton = screen.getByRole('button', { name: /发送/i })

    await user.type(textarea, '测试问题')
    await user.click(sendButton)

    await waitFor(() => {
      expect(screen.getByText(/^发送失败$/)).toBeInTheDocument()
    })
    consoleErrorSpy.mockRestore()
  })

  it('应该显示开始新对话按钮', () => {
    mockInvoke.mockResolvedValue(undefined)

    render(<QnA />, { wrapper: Wrapper })

    expect(screen.getByRole('button', { name: /新对话/i })).toBeInTheDocument()
  })

  it('应该能够开始新对话', async () => {
    const user = userEvent.setup()
    mockListen.mockImplementationOnce(async (eventName: string, handler: (event: { payload: unknown }) => void) => {
      if (eventName === 'ai-chat-chunk') {
        handler({ payload: '回复' })
      }
      return () => {}
    })
    mockInvoke
      .mockResolvedValueOnce(1)
      .mockResolvedValueOnce(2)
      .mockResolvedValueOnce(undefined) // ai_chat_stream
      .mockResolvedValueOnce(3)

    render(<QnA />, { wrapper: Wrapper })

    // 发送一条消息
    const textarea = screen.getByPlaceholderText(/输入你的问题/i)
    await user.type(textarea, '测试问题')
    await user.click(screen.getByRole('button', { name: /发送/i }))

    await waitFor(() => {
      expect(screen.getByText('测试问题')).toBeInTheDocument()
    })

    // 开始新对话
    const newConversationButton = screen.getByRole('button', { name: /新对话/i })
    await user.click(newConversationButton)

    await waitFor(() => {
      expect(screen.getByText(/你可以在这里提问/i)).toBeInTheDocument()
      expect(screen.queryByText('测试问题')).not.toBeInTheDocument()
    })
  })

  it('应该加载历史消息当传入 initialSessionId', async () => {
    const mockMessages = [
      {
        id: 1,
        role: 'user',
        content: '历史问题',
        createdAt: Date.now(),
        rating: null,
      },
      {
        id: 2,
        role: 'assistant',
        content: '历史回答',
        createdAt: Date.now(),
        rating: 1,
      },
    ]

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_chat_messages') return mockMessages
      return undefined
    })

    render(<QnA initialSessionId={123} />, { wrapper: Wrapper })

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('get_chat_messages', {
        sessionId: 123,
      })
    })

    await waitFor(() => {
      expect(screen.getByText('历史问题')).toBeInTheDocument()
      expect(screen.getByText('历史回答')).toBeInTheDocument()
    })
  })

  it('应该在加载历史消息时显示加载指示器', async () => {
    let resolveMessages: (value: unknown[]) => void
    const messagesPromise = new Promise<unknown[]>((resolve) => {
      resolveMessages = resolve
    })

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_chat_messages') return messagesPromise
      return undefined
    })

    render(<QnA initialSessionId={123} />, { wrapper: Wrapper })

    // 应该显示加载指示器
    await waitFor(() => {
      expect(screen.getByRole('status', { hidden: true })).toBeInTheDocument()
    })

    // 解析消息后应该隐藏加载指示器
    resolveMessages!([])
    await waitFor(() => {
      expect(screen.queryByRole('status', { hidden: true })).not.toBeInTheDocument()
    })
  })

  it('应该应用 draft 属性到输入框', async () => {
    mockInvoke.mockResolvedValue(undefined)

    render(<QnA draft="预设文本" />, { wrapper: Wrapper })

    await waitFor(() => {
      const textarea = screen.getByPlaceholderText(/输入你的问题/i)
      expect(textarea).toHaveValue('预设文本')
    })
  })

  it('应该调用 onSessionCreated 回调', async () => {
    const user = userEvent.setup()
    const onSessionCreated = vi.fn()

    mockListen.mockImplementationOnce(async (eventName: string, handler: (event: { payload: unknown }) => void) => {
      if (eventName === 'ai-chat-chunk') {
        handler({ payload: '回复' })
      }
      return () => {}
    })
    mockInvoke
      .mockResolvedValueOnce(456) // create_chat_session
      .mockResolvedValueOnce(1) // save_chat_message (user)
      .mockResolvedValueOnce(undefined) // ai_chat_stream
      .mockResolvedValueOnce(2) // save_chat_message (assistant)

    render(<QnA onSessionCreated={onSessionCreated} />, { wrapper: Wrapper })

    const textarea = screen.getByPlaceholderText(/输入你的问题/i)
    const sendButton = screen.getByRole('button', { name: /发送/i })

    await user.type(textarea, '测试')
    await user.click(sendButton)

    await waitFor(() => {
      expect(onSessionCreated).toHaveBeenCalledWith(456)
    }, { timeout: 3000 })
  })
})

