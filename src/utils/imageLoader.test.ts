import { describe, it, expect, vi, beforeEach } from 'vitest'
import { getScreenshotUrl, getScreenshotUrls } from './imageLoader'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `tauri://localhost/${path}`),
}))

describe('imageLoader', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('getScreenshotUrl', () => {
    it('应该成功获取图片 URL', async () => {
      const mockPath = '/path/to/image.png'
      const mockFullPath = 'C:\\Users\\test\\image.png'
      
      vi.mocked(invoke).mockResolvedValue(mockFullPath)
      vi.mocked(convertFileSrc).mockReturnValue(`tauri://localhost/${mockFullPath}`)

      const url = await getScreenshotUrl(mockPath)

      expect(invoke).toHaveBeenCalledWith('get_image_path', { filename: mockPath })
      expect(convertFileSrc).toHaveBeenCalledWith(mockFullPath)
      expect(url).toBe(`tauri://localhost/${mockFullPath}`)
    })

    it('应该缓存已获取的 URL', async () => {
      const mockPath = '/path/to/image.png'
      const mockFullPath = 'C:\\Users\\test\\image.png'
      
      vi.mocked(invoke).mockResolvedValue(mockFullPath)
      vi.mocked(convertFileSrc).mockReturnValue(`tauri://localhost/${mockFullPath}`)

      const url1 = await getScreenshotUrl(mockPath)
      // 第二次调用应该使用缓存，不会再次调用 invoke
      vi.mocked(invoke).mockClear()
      const url2 = await getScreenshotUrl(mockPath)

      expect(invoke).not.toHaveBeenCalled()
      expect(url1).toBe(url2)
    })

    it('应该处理错误并返回占位符', async () => {
      const mockPath = '/path/to/error-image.png' // 使用不同的路径避免缓存
      
      vi.mocked(invoke).mockRejectedValue(new Error('获取图片失败'))

      const url = await getScreenshotUrl(mockPath)

      expect(url).toBe('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTI4IiBoZWlnaHQ9IjgwIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxyZWN0IHdpZHRoPSIxMjgiIGhlaWdodD0iODAiIGZpbGw9IiMxMjEyMTQiLz48dGV4dCB4PSI1MCUiIHk9IjUwJSIgZm9udC1mYW1pbHk9IkFyaWFsIiBmb250LXNpemU9IjE0IiBmaWxsPSIjNjY2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBkeT0iLjNlbSI+5Zu+54mH5pyq5Yqg6L29PC90ZXh0Pjwvc3ZnPg==')
    })

    it('应该处理并发请求', async () => {
      const mockPath = '/path/to/image.png'
      const mockFullPath = 'C:\\Users\\test\\image.png'
      
      let callCount = 0
      vi.mocked(invoke).mockImplementation(() => {
        callCount++
        return new Promise(resolve => setTimeout(() => resolve(mockFullPath), 50))
      })
      vi.mocked(convertFileSrc).mockReturnValue(`tauri://localhost/${mockFullPath}`)

      const [url1, url2] = await Promise.all([
        getScreenshotUrl(mockPath),
        getScreenshotUrl(mockPath),
      ])

      // 由于并发请求共享同一个 pending promise，应该只调用一次
      expect(callCount).toBeLessThanOrEqual(1)
      expect(url1).toBe(url2)
    })
  })

  describe('getScreenshotUrls', () => {
    it('应该处理空数组', async () => {
      const urls = await getScreenshotUrls([])
      expect(urls).toEqual([])
    })

    it('应该获取多个图片 URL', async () => {
      const paths = ['/path/to/image1.png', '/path/to/image2.png']
      const mockFullPaths = ['C:\\Users\\test\\image1.png', 'C:\\Users\\test\\image2.png']
      
      // getScreenshotUrls 会先尝试批量 API，失败后回退到单个请求
      vi.mocked(invoke)
        .mockRejectedValueOnce(new Error('批量获取失败')) // 批量 API 失败
        .mockResolvedValueOnce(mockFullPaths[0]) // 第一个单独请求
        .mockResolvedValueOnce(mockFullPaths[1]) // 第二个单独请求
      vi.mocked(convertFileSrc).mockImplementation((path: string) => `tauri://localhost/${path}`)

      const urls = await getScreenshotUrls(paths)

      expect(urls).toHaveLength(2)
      expect(urls[0]).toBe(`tauri://localhost/${mockFullPaths[0]}`)
      expect(urls[1]).toBe(`tauri://localhost/${mockFullPaths[1]}`)
    })

    it('应该去重重复的路径', async () => {
      const paths = ['/path/to/unique1.png', '/path/to/unique1.png', '/path/to/unique2.png'] // 使用不同的路径避免缓存
      const mockFullPaths = ['C:\\Users\\test\\unique1.png', 'C:\\Users\\test\\unique2.png']
      
      // 批量 API 成功
      vi.mocked(invoke).mockResolvedValueOnce(mockFullPaths)
      vi.mocked(convertFileSrc).mockImplementation((path: string) => `tauri://localhost/${path}`)

      const urls = await getScreenshotUrls(paths)

      // 检查是否调用了批量 API，去重后的路径
      expect(invoke).toHaveBeenCalledWith('get_image_paths', { 
        filenames: ['/path/to/unique1.png', '/path/to/unique2.png'] 
      })
      expect(urls).toHaveLength(3)
    })

    it('应该使用批量 API 获取路径', async () => {
      const paths = ['/path/to/batch1.png', '/path/to/batch2.png'] // 使用不同的路径避免缓存
      const mockFullPaths = ['C:\\Users\\test\\batch1.png', 'C:\\Users\\test\\batch2.png']
      
      // 批量 API 成功
      vi.mocked(invoke).mockResolvedValueOnce(mockFullPaths)
      vi.mocked(convertFileSrc).mockImplementation((path: string) => `tauri://localhost/${path}`)

      const urls = await getScreenshotUrls(paths)

      expect(invoke).toHaveBeenCalledWith('get_image_paths', { 
        filenames: paths 
      })
      expect(urls).toHaveLength(2)
      expect(urls[0]).toBe(`tauri://localhost/${mockFullPaths[0]}`)
      expect(urls[1]).toBe(`tauri://localhost/${mockFullPaths[1]}`)
    })

    it('应该在批量 API 失败时回退到单个请求', async () => {
      const paths = ['/path/to/image1.png', '/path/to/image2.png']
      const mockFullPath1 = 'C:\\Users\\test\\image1.png'
      const mockFullPath2 = 'C:\\Users\\test\\image2.png'
      
      vi.mocked(invoke)
        .mockRejectedValueOnce(new Error('批量获取失败')) // 批量 API 失败
        .mockResolvedValueOnce(mockFullPath1) // 第一个单独请求
        .mockResolvedValueOnce(mockFullPath2) // 第二个单独请求
      vi.mocked(convertFileSrc).mockImplementation((path: string) => `tauri://localhost/${path}`)

      const urls = await getScreenshotUrls(paths)

      expect(urls).toHaveLength(2)
    })
  })
})
