import { useEffect, useState } from 'react'
import { X, ZoomIn, ZoomOut, RotateCw } from 'lucide-react'
import { ActivityLog } from '../contexts/AppContext'
import { getScreenshotUrl } from '../utils/imageLoader'

interface ImagePreviewModalProps {
  open: boolean
  activity: ActivityLog | null
  onClose: () => void
}

export default function ImagePreviewModal({
  open,
  activity,
  onClose,
}: ImagePreviewModalProps) {
  const [imageUrl, setImageUrl] = useState<string>('')
  const [loading, setLoading] = useState(true)
  const [scale, setScale] = useState(1)
  const [rotation, setRotation] = useState(0)
  const [position, setPosition] = useState({ x: 0, y: 0 })
  const [isDragging, setIsDragging] = useState(false)
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 })

  useEffect(() => {
    if (open && activity) {
      setLoading(true)
      getScreenshotUrl(activity.imagePath).then((url) => {
        setImageUrl(url)
        setLoading(false)
      })
      // 重置状态
      setScale(1)
      setRotation(0)
      setPosition({ x: 0, y: 0 })
    }
  }, [open, activity])

  // ESC 键关闭
  useEffect(() => {
    if (!open) return

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [open, onClose])

  // 鼠标滚轮缩放
  const handleWheel = (e: React.WheelEvent) => {
    e.preventDefault()
    const delta = e.deltaY > 0 ? -0.1 : 0.1
    setScale((prev) => Math.max(0.5, Math.min(3, prev + delta)))
  }

  // 鼠标拖拽
  const handleMouseDown = (e: React.MouseEvent) => {
    if (scale > 1) {
      setIsDragging(true)
      setDragStart({
        x: e.clientX - position.x,
        y: e.clientY - position.y,
      })
    }
  }

  const handleMouseMove = (e: React.MouseEvent) => {
    if (isDragging && scale > 1) {
      setPosition({
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y,
      })
    }
  }

  const handleMouseUp = () => {
    setIsDragging(false)
  }

  // 将时间戳转换为毫秒（后端返回的是秒级时间戳）
  const toMs = (ts: number) => (ts < 1e12 ? ts * 1000 : ts)

  const formatTime = (timestamp: number) => {
    const date = new Date(toMs(timestamp))
    return date.toLocaleString('zh-CN', {
      timeZone: 'Asia/Shanghai',
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    })
  }

  if (!open || !activity) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm"
      onClick={(e) => {
        // 点击背景关闭
        if (e.target === e.currentTarget) {
          onClose()
        }
      }}
    >
      <div className="relative w-full h-full flex flex-col">
        {/* 顶部工具栏 */}
        <div className="glass border-b border-glass-border px-6 py-4 flex items-center justify-between">
          <div className="flex-1 min-w-0">
            <h3 className="text-lg font-semibold text-white truncate">
              {activity.appName}
            </h3>
            <p className="text-sm text-gray-400 truncate">{activity.windowTitle}</p>
            <p className="text-xs text-gray-500 mt-1">{formatTime(activity.timestamp)}</p>
          </div>

          <div className="flex items-center gap-2 ml-4">
            {/* 缩放控制 */}
            <div className="flex items-center gap-1 bg-surface/50 rounded-lg p-1">
              <button
                onClick={() => setScale((prev) => Math.max(0.5, prev - 0.1))}
                className="p-2 rounded hover:bg-surface transition-colors"
                title="缩小"
              >
                <ZoomOut className="w-4 h-4 text-gray-300" />
              </button>
              <span className="text-xs text-gray-400 px-2 min-w-[3rem] text-center">
                {Math.round(scale * 100)}%
              </span>
              <button
                onClick={() => setScale((prev) => Math.min(3, prev + 0.1))}
                className="p-2 rounded hover:bg-surface transition-colors"
                title="放大"
              >
                <ZoomIn className="w-4 h-4 text-gray-300" />
              </button>
            </div>

            {/* 旋转 */}
            <button
              onClick={() => setRotation((prev) => (prev + 90) % 360)}
              className="p-2 rounded-lg hover:bg-surface transition-colors"
              title="旋转"
            >
              <RotateCw className="w-4 h-4 text-gray-300" />
            </button>

            {/* 关闭 */}
            <button
              onClick={onClose}
              className="p-2 rounded-lg hover:bg-surface transition-colors"
              title="关闭 (ESC)"
            >
              <X className="w-5 h-5 text-gray-300" />
            </button>
          </div>
        </div>

        {/* 图片预览区域 */}
        <div
          className="flex-1 overflow-hidden relative"
          onWheel={handleWheel}
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
          style={{ cursor: scale > 1 ? (isDragging ? 'grabbing' : 'grab') : 'default' }}
        >
          {loading ? (
            <div className="flex items-center justify-center h-full">
              <div className="w-8 h-8 border-2 border-neon-blue border-t-transparent rounded-full animate-spin" />
            </div>
          ) : (
            <div className="w-full h-full flex items-center justify-center">
              <img
                src={imageUrl}
                alt="Screenshot preview"
                className="max-w-full max-h-full object-contain select-none"
                style={{
                  transform: `scale(${scale}) rotate(${rotation}deg) translate(${position.x / scale}px, ${position.y / scale}px)`,
                  transition: isDragging ? 'none' : 'transform 0.1s ease-out',
                }}
                draggable={false}
                onError={(e) => {
                  // 加载失败时显示占位符
                  e.currentTarget.src =
                    'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iODAwIiBoZWlnaHQ9IjYwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB3aWR0aD0iODAwIiBoZWlnaHQ9IjYwMCIgZmlsbD0iIzEyMTIxNCIvPjx0ZXh0IHg9IjUwJSIgeT0iNTAlIiBmb250LWZhbWlseT0iQXJpYWwiIGZvbnQtc2l6ZT0iMjQiIGZpbGw9IiM2NjYiIHRleHQtYW5jaG9yPSJtaWRkbGUiIGR5PSIuM2VtIj7lm77niYfmnKrlirDovb08L3RleHQ+PC9zdmc+'
                }}
              />
            </div>
          )}
        </div>

        {/* 底部信息栏 */}
        {activity.ocrText && (
          <div className="glass border-t border-glass-border px-6 py-3 max-h-32 overflow-y-auto">
            <p className="text-sm text-gray-300 leading-relaxed">{activity.ocrText}</p>
          </div>
        )}

        {/* 提示信息 */}
        {scale > 1 && (
          <div className="absolute bottom-4 left-1/2 transform -translate-x-1/2 glass px-4 py-2 rounded-lg text-xs text-gray-400">
            拖拽移动 | 滚轮缩放 | ESC 关闭
          </div>
        )}
      </div>
    </div>
  )
}

