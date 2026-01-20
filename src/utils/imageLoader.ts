import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

const urlCache = new Map<string, string>()
const pendingCache = new Map<string, Promise<string>>()

/**
 * 获取截图图片的 URL
 * 优先使用 appimg:// 协议，如果失败则回退到文件路径
 */
export async function getScreenshotUrl(imagePath: string): Promise<string> {
  const cached = urlCache.get(imagePath)
  if (cached) return cached

  const pending = pendingCache.get(imagePath)
  if (pending) return pending

  try {
    const promise = invoke<string>('get_image_path', { filename: imagePath })
      .then((fullPath) => convertFileSrc(fullPath))
      .then((url) => {
        urlCache.set(imagePath, url)
        pendingCache.delete(imagePath)
        return url
      })
      .catch((error) => {
        pendingCache.delete(imagePath)
        console.error('获取图片路径失败:', error)
        return 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTI4IiBoZWlnaHQ9IjgwIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxyZWN0IHdpZHRoPSIxMjgiIGhlaWdodD0iODAiIGZpbGw9IiMxMjEyMTQiLz48dGV4dCB4PSI1MCUiIHk9IjUwJSIgZm9udC1mYW1pbHk9IkFyaWFsIiBmb250LXNpemU9IjE0IiBmaWxsPSIjNjY2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBkeT0iLjNlbSI+5Zu+54mH5pyq5Yqg6L29PC90ZXh0Pjwvc3ZnPg=='
      })

    pendingCache.set(imagePath, promise)
    return await promise
  } catch (error) {
    console.error('获取图片路径失败:', error)
    // 回退方案：返回占位符
    return 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTI4IiBoZWlnaHQ9IjgwIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxyZWN0IHdpZHRoPSIxMjgiIGhlaWdodD0iODAiIGZpbGw9IiMxMjEyMTQiLz48dGV4dCB4PSI1MCUiIHk9IjUwJSIgZm9udC1mYW1pbHk9IkFyaWFsIiBmb250LXNpemU9IjE0IiBmaWxsPSIjNjY2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBkeT0iLjNlbSI+5Zu+54mH5pyq5Yqg6L29PC90ZXh0Pjwvc3ZnPg=='
  }
}

export async function getScreenshotUrls(imagePaths: string[]): Promise<string[]> {
  if (imagePaths.length === 0) return []

  const unique: string[] = []
  const seen = new Set<string>()
  for (const p of imagePaths) {
    if (seen.has(p)) continue
    seen.add(p)
    if (urlCache.has(p)) continue
    unique.push(p)
  }

  if (unique.length > 0) {
    try {
      const fullPaths = await invoke<string[]>('get_image_paths', { filenames: unique })
      for (let i = 0; i < unique.length; i++) {
        const filename = unique[i]
        const fullPath = fullPaths[i]
        if (!fullPath) continue
        urlCache.set(filename, convertFileSrc(fullPath))
      }
    } catch {
      await Promise.all(unique.map((p) => getScreenshotUrl(p)))
    }
  }

  return await Promise.all(imagePaths.map((p) => getScreenshotUrl(p)))
}

