import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

/**
 * 获取截图图片的 URL
 * 优先使用 appimg:// 协议，如果失败则回退到文件路径
 */
export async function getScreenshotUrl(imagePath: string): Promise<string> {
  try {
    // 尝试使用 appimg:// 协议
    // 注意：在 Tauri 2.0 中，可能需要通过 convertFileSrc 转换
    const fullPath = await invoke<string>('get_image_path', { filename: imagePath })
    
    // 使用 Tauri 的 convertFileSrc 将文件路径转换为可访问的 URL
    return convertFileSrc(fullPath)
  } catch (error) {
    console.error('获取图片路径失败:', error)
    // 回退方案：返回占位符
    return 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTI4IiBoZWlnaHQ9IjgwIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxyZWN0IHdpZHRoPSIxMjgiIGhlaWdodD0iODAiIGZpbGw9IiMxMjEyMTQiLz48dGV4dCB4PSI1MCUiIHk9IjUwJSIgZm9udC1mYW1pbHk9IkFyaWFsIiBmb250LXNpemU9IjE0IiBmaWxsPSIjNjY2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBkeT0iLjNlbSI+5Zu+54mH5pyq5Yqg6L29PC90ZXh0Pjwvc3ZnPg=='
  }
}

