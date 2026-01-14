import { useEffect, useState } from 'react'
import { useApp } from '../contexts/AppContext'
import { Virtuoso } from 'react-virtuoso'
import { Clock, Monitor, FileText, Search, Filter, X, Calendar } from 'lucide-react'
import { getScreenshotUrl } from '../utils/imageLoader'
import ImagePreviewModal from './ImagePreviewModal'
import { ActivityLog } from '../contexts/AppContext'

export default function Timeline() {
  const { state, loadActivities, searchActivities } = useApp()
  const [previewActivity, setPreviewActivity] = useState<ActivityLog | null>(null)
  const [query, setQuery] = useState('')
  const [showFilters, setShowFilters] = useState(false)
  const [appName, setAppName] = useState('')
  const [hasOcr, setHasOcr] = useState(false)
  const [startDate, setStartDate] = useState('')
  const [endDate, setEndDate] = useState('')

  const parseDateParts = (value: string) => {
    const trimmed = value.trim()
    if (!trimmed) return null
    const parts = trimmed.includes('-') ? trimmed.split('-') : trimmed.split('/')
    if (parts.length !== 3) return null
    const year = Number(parts[0])
    const month = Number(parts[1])
    const day = Number(parts[2])
    if (!Number.isFinite(year) || !Number.isFinite(month) || !Number.isFinite(day)) return null
    if (month < 1 || month > 12) return null
    if (day < 1 || day > 31) return null
    return { year, month, day }
  }

  const toShanghaiEpochSeconds = (
    year: number,
    month: number,
    day: number,
    hours: number,
    minutes: number,
    seconds: number,
    milliseconds: number
  ) => {
    const utcMs = Date.UTC(year, month - 1, day, hours, minutes, seconds, milliseconds)
    const shanghaiOffsetMs = 8 * 60 * 60 * 1000
    return Math.floor((utcMs - shanghaiOffsetMs) / 1000)
  }

  useEffect(() => {
    loadActivities()
  }, [])

  const handleSearch = () => {
    // Convert dates to timestamps
    let fromTs: number | undefined
    let toTs: number | undefined

    if (startDate) {
      const d = parseDateParts(startDate)
      if (d) {
        fromTs = toShanghaiEpochSeconds(d.year, d.month, d.day, 0, 0, 0, 0)
      }
    }
    if (endDate) {
      const d = parseDateParts(endDate)
      if (d) {
        toTs = toShanghaiEpochSeconds(d.year, d.month, d.day, 23, 59, 59, 999)
      }
    }

    if (fromTs !== undefined && toTs !== undefined && fromTs > toTs) {
      const tmp = fromTs
      fromTs = toTs
      toTs = tmp
    }

    searchActivities({
      query: query || undefined,
      appName: appName || undefined,
      hasOcr: hasOcr || undefined,
      fromTs,
      toTs,
    })
  }

  const clearSearch = () => {
    setQuery('')
    setAppName('')
    setHasOcr(false)
    setStartDate('')
    setEndDate('')
    loadActivities()
  }

  // 将时间戳转换为毫秒（后端返回的是秒级时间戳）
  const toMs = (timestamp: number) => {
    // 如果时间戳小于 10^12，说明是秒级时间戳，需要乘以 1000
    return timestamp < 1e12 ? timestamp * 1000 : timestamp
  }

  const formatTime = (timestamp: number) => {
    const date = new Date(toMs(timestamp))
    return date.toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      timeZone: 'Asia/Shanghai',
    })
  }

  const formatDate = (timestamp: number) => {
    const date = new Date(toMs(timestamp))
    const today = new Date()
    const yesterday = new Date(today)
    yesterday.setDate(yesterday.getDate() - 1)

    if (date.toDateString() === today.toDateString()) {
      return '今天'
    } else if (date.toDateString() === yesterday.toDateString()) {
      return '昨天'
    } else {
      return date.toLocaleDateString('zh-CN', {
        month: 'short',
        day: 'numeric',
        timeZone: 'Asia/Shanghai',
      })
    }
  }

  const handleImageClick = (activity: ActivityLog) => {
    setPreviewActivity(activity)
  }

  return (
    <div className="h-full flex flex-col">
      <ImagePreviewModal
        open={previewActivity !== null}
        activity={previewActivity}
        onClose={() => setPreviewActivity(null)}
      />

      <div className="glass border-b border-glass-border px-6 py-4 space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-neon-blue flex items-center gap-2">
            <Clock className="w-5 h-5" />
            活动时间轴
          </h2>
          <div className="text-sm text-gray-400">
            共 {state.activities.length} 条记录
          </div>
        </div>

        <div className="flex gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
            <input
              type="text"
              placeholder="搜索活动..."
              className="w-full bg-surface/50 border border-glass-border rounded-lg pl-9 pr-4 py-2 text-sm focus:outline-none focus:border-neon-blue transition-colors text-white"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            />
            {query && (
              <button
                onClick={() => setQuery('')}
                className="absolute right-3 top-1/2 -translate-y-1/2"
              >
                <X className="w-4 h-4 text-gray-500 hover:text-white" />
              </button>
            )}
          </div>
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={`p-2 rounded-lg border border-glass-border transition-colors ${
              showFilters
                ? 'bg-neon-blue/20 border-neon-blue text-neon-blue'
                : 'hover:bg-surface/50 text-gray-400'
            }`}
          >
            <Filter className="w-5 h-5" />
          </button>
          <button
            onClick={handleSearch}
            className="px-4 py-2 bg-neon-blue text-black font-semibold rounded-lg hover:bg-neon-blue/90 transition-colors"
          >
            搜索
          </button>
          {(query || appName || hasOcr || startDate || endDate) && (
             <button
                onClick={clearSearch}
                className="px-3 py-2 border border-glass-border text-gray-400 rounded-lg hover:bg-surface/50 hover:text-white transition-colors text-sm"
             >
                重置
             </button>
          )}
        </div>

        {showFilters && (
          <div className="grid grid-cols-2 gap-4 p-4 bg-surface/30 rounded-lg animate-in slide-in-from-top-2">
            <div className="space-y-1">
              <label className="text-xs text-gray-400">应用名称</label>
              <input
                type="text"
                placeholder="例如: Chrome"
                className="w-full bg-surface/50 border border-glass-border rounded px-3 py-1.5 text-sm text-white"
                value={appName}
                onChange={(e) => setAppName(e.target.value)}
              />
            </div>
            
            <div className="space-y-1">
              <label className="text-xs text-gray-400 flex items-center gap-1">
                <Calendar className="w-3 h-3" /> 日期范围
              </label>
              <div className="flex items-center gap-2">
                <input
                  type="date"
                  className="w-full bg-surface/50 border border-glass-border rounded px-2 py-1.5 text-sm text-white [color-scheme:dark]"
                  value={startDate}
                  onChange={(e) => setStartDate(e.target.value)}
                />
                <span className="text-gray-500">-</span>
                <input
                  type="date"
                  className="w-full bg-surface/50 border border-glass-border rounded px-2 py-1.5 text-sm text-white [color-scheme:dark]"
                  value={endDate}
                  onChange={(e) => setEndDate(e.target.value)}
                />
              </div>
            </div>

            <div className="flex items-end col-span-2">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  className="rounded border-glass-border bg-surface/50 text-neon-blue focus:ring-neon-blue"
                  checked={hasOcr}
                  onChange={(e) => setHasOcr(e.target.checked)}
                />
                <span className="text-sm text-gray-300">仅显示含 OCR 文本</span>
              </label>
            </div>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-hidden">
        {state.activities.length === 0 ? (
          <div className="flex items-center justify-center h-full text-gray-500">
            <div className="text-center">
              <Monitor className="w-16 h-16 mx-auto mb-4 opacity-50" />
              <p>暂无活动记录</p>
              <p className="text-sm mt-2">开始录制后，活动将显示在这里</p>
            </div>
          </div>
        ) : (
          <Virtuoso
            data={state.activities}
            itemContent={(index, activity) => {
              const prevActivity = index > 0 ? state.activities[index - 1] : null
              const showDateSeparator =
                !prevActivity ||
                formatDate(prevActivity.timestamp) !== formatDate(activity.timestamp)

              return (
                <div key={activity.id}>
                  {showDateSeparator && (
                    <div className="px-6 py-3 text-xs font-semibold text-gray-500 uppercase tracking-wider">
                      {formatDate(activity.timestamp)}
                    </div>
                  )}
                  <div className="glass mx-6 mb-3 p-4 rounded-lg hover:bg-surface/50 transition-all">
                    <div className="flex items-start gap-4">
                      {/* 截图缩略图 */}
                      <div className="flex-shrink-0">
                        <ScreenshotImage
                          imagePath={activity.imagePath}
                          onClick={() => handleImageClick(activity)}
                        />
                      </div>

                      {/* 活动信息 */}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-2">
                          <Monitor className="w-4 h-4 text-neon-blue flex-shrink-0" />
                          <span className="font-medium text-white truncate">
                            {activity.appName}
                          </span>
                          <span className="text-xs text-gray-500">
                            {formatTime(activity.timestamp)}
                          </span>
                        </div>

                        <div className="text-sm text-gray-400 mb-2 truncate">
                          {activity.windowTitle}
                        </div>

                        {activity.ocrText && (
                          <div className="flex items-start gap-2 mt-2">
                            <FileText className="w-4 h-4 text-neon-green flex-shrink-0 mt-0.5" />
                            <p className="text-sm text-gray-300 line-clamp-2">
                              {activity.ocrText}
                            </p>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              )
            }}
            style={{ height: '100%' }}
          />
        )}
      </div>
    </div>
  )
}

// 截图图片组件，处理图片加载
function ScreenshotImage({
  imagePath,
  onClick,
}: {
  imagePath: string
  onClick?: () => void
}) {
  const [imageUrl, setImageUrl] = useState<string>('')
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    getScreenshotUrl(imagePath).then((url) => {
      setImageUrl(url)
      setLoading(false)
    })
  }, [imagePath])

  if (loading) {
    return (
      <div className="w-32 h-20 bg-surface rounded border border-glass-border flex items-center justify-center">
        <div className="w-4 h-4 border-2 border-neon-blue border-t-transparent rounded-full animate-spin" />
      </div>
    )
  }

  return (
    <img
      src={imageUrl}
      alt="Screenshot"
      className="w-32 h-20 object-cover rounded border border-glass-border cursor-pointer hover:opacity-80 transition-opacity"
      loading="lazy"
      onClick={onClick}
      onError={(e) => {
        // 加载失败时显示占位符
        e.currentTarget.src =
          'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTI4IiBoZWlnaHQ9IjgwIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxyZWN0IHdpZHRoPSIxMjgiIGhlaWdodD0iODAiIGZpbGw9IiMxMjEyMTQiLz48dGV4dCB4PSI1MCUiIHk9IjUwJSIgZm9udC1mYW1pbHk9IkFyaWFsIiBmb250LXNpemU9IjE0IiBmaWxsPSIjNjY2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBkeT0iLjNlbSI+5Zu+54mH5pyq5Yqg6L29PC90ZXh0Pjwvc3ZnPg=='
      }}
    />
  )
}

