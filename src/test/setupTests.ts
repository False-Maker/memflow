import { expect, afterEach, vi } from 'vitest'
import { cleanup } from '@testing-library/react'
import * as matchers from '@testing-library/jest-dom/matchers'
import '@testing-library/jest-dom/vitest'

// 扩展 Vitest 的 expect 以包含 jest-dom 匹配器
expect.extend(matchers)

// 每个测试后清理
afterEach(() => {
  cleanup()
})

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => {
  const mockTransformCallback = vi.fn((callback) => callback)
  return {
    invoke: vi.fn(),
    convertFileSrc: vi.fn((path: string) => `tauri://localhost/${path}`),
    transformCallback: mockTransformCallback,
  }
})

vi.mock('@tauri-apps/api/event', () => {
  const unlistenFn = vi.fn(() => {})
  return {
    listen: vi.fn(() => Promise.resolve(unlistenFn)),
  }
})

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
}))

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

