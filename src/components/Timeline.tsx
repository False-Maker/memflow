import { useEffect, useState } from 'react'
import { useApp } from '../contexts/AppContext'
import { Virtuoso } from 'react-virtuoso'
import { Clock, Monitor, FileText, Search, Filter, X, Calendar, Sparkles } from 'lucide-react'
import { getScreenshotUrl } from '../utils/imageLoader'
import ImagePreviewModal from './ImagePreviewModal'
import { ActivityLog } from '../contexts/AppContext'
import { invoke } from '@tauri-apps/api/core'

export default function Timeline() {
  const { state, loadActivities, searchActivities } = useApp()
  const [previewActivity, setPreviewActivity] = useState<ActivityLog | null>(null)
  const [query, setQuery] = useState('')
  const [showFilters, setShowFilters] = useState(false)
  const [appName, setAppName] = useState('')
  const [hasOcr, setHasOcr] = useState(false)
  const [startDate, setStartDate] = useState('')
  const [endDate, setEndDate] = useState('')
  const [isParsingIntent, setIsParsingIntent] = useState(false)
  const [smartSearchNotice, setSmartSearchNotice] = useState<string | null>(null)
  const [smartSearchError, setSmartSearchError] = useState<string | null>(null)

  const redactSensitive = (text: string) =>
    text
      .replace(/Incorrect API key provided:\s*([^\s".\r\n]+)/g, 'Incorrect API key provided: [REDACTED]')
      .replace(/Bearer\s+([^\s"'\r\n]+)/g, 'Bearer [REDACTED]')
      .replace(/sk-[A-Za-z0-9_-]+/g, 'sk-[REDACTED]')

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

  const parseRuleQuery = (rawQuery: string) => {
    const parts = rawQuery
      .split(/\s+/)
      .map((p) => p.trim())
      .filter(Boolean)

    let extractedAppName: string | undefined
    let extractedHasOcr: boolean | undefined
    let extractedStartDate: string | undefined
    let extractedEndDate: string | undefined
    const remaining: string[] = []

    const normalizeDate = (value: string) => {
      const d = parseDateParts(value)
      if (!d) return undefined
      const y = String(d.year).padStart(4, '0')
      const m = String(d.month).padStart(2, '0')
      const day = String(d.day).padStart(2, '0')
      return `${y}-${m}-${day}`
    }

    for (const token of parts) {
      const appMatch = token.match(/^app:(.+)$/i)
      if (appMatch) {
        const value = appMatch[1].trim()
        if (value) extractedAppName = value
        continue
      }

      const fromMatch = token.match(/^from:(.+)$/i)
      if (fromMatch) {
        const normalized = normalizeDate(fromMatch[1].trim())
        if (normalized) extractedStartDate = normalized
        continue
      }

      const toMatch = token.match(/^to:(.+)$/i)
      if (toMatch) {
        const normalized = normalizeDate(toMatch[1].trim())
        if (normalized) extractedEndDate = normalized
        continue
      }

      const ocrMatch = token.match(/^ocr:(true|false)$/i)
      if (ocrMatch) {
        extractedHasOcr = ocrMatch[1].toLowerCase() === 'true'
        continue
      }

      if (/^has:ocr$/i.test(token)) {
        extractedHasOcr = true
        continue
      }

      remaining.push(token)
    }

    return {
      queryText: remaining.join(' ').trim(),
      appName: extractedAppName,
      hasOcr: extractedHasOcr,
      startDate: extractedStartDate,
      endDate: extractedEndDate,
    }
  }

  // 监听 state.lastSearchParams 的变化，同步到本地 state
  useEffect(() => {
    if (state.lastSearchParams) {
      // 只有当本地状态与全局状态不一致时才更新，避免死循环（虽然这里是单向同步）
      // 这里主要关注 fromTs/toTs 转换回 startDate/endDate 的逻辑
      // 注意：startDate/endDate 格式是 yyyy-MM-dd
      
      const p = state.lastSearchParams
      if (p.query !== undefined && p.query !== query) setQuery(p.query)
      if (p.appName !== undefined && p.appName !== appName) setAppName(p.appName)
      if (p.hasOcr !== undefined && p.hasOcr !== hasOcr) setHasOcr(p.hasOcr)
      
      if (p.fromTs) {
        const d = new Date(toMs(p.fromTs))
        // 简单处理：转为 YYYY-MM-DD
        // 注意时区问题，这里使用本地时间（因为 Date 对象默认就是本地时间）
        // 但 input type="date" 需要 yyyy-MM-dd
        const y = d.getFullYear()
        const m = String(d.getMonth() + 1).padStart(2, '0')
        const day = String(d.getDate()).padStart(2, '0')
        const s = `${y}-${m}-${day}`
        if (s !== startDate) setStartDate(s)
      } else if (startDate) {
          // 如果全局清空了，本地也清空
          setStartDate('')
      }
      
      if (p.toTs) {
        const d = new Date(toMs(p.toTs))
        const y = d.getFullYear()
        const m = String(d.getMonth() + 1).padStart(2, '0')
        const day = String(d.getDate()).padStart(2, '0')
        const s = `${y}-${m}-${day}`
        if (s !== endDate) setEndDate(s)
      } else if (endDate) {
          setEndDate('')
      }
      
      // 如果有任何过滤条件，展开过滤面板
      if (p.appName || p.fromTs || p.toTs || p.hasOcr) {
        setShowFilters(true)
      }
    }
  }, [state.lastSearchParams])

  const handleSmartSearch = async () => {
    const trimmedQuery = query.trim()
    if (!trimmedQuery) return
    
    setIsParsingIntent(true)
    setSmartSearchError(null)
    setSmartSearchNotice('正在解析智能搜索意图...')
    try {
      const intent = await invoke<{
          app_name?: string | null, 
          keywords?: string[], 
          date_range?: string | null, 
          has_ocr?: boolean | null
      }>('parse_query_intent', { query })
      
      console.log("Intent parsed:", intent)

      let newAppName = appName
      let newHasOcr = hasOcr
      let newStartDate = startDate
      let newEndDate = endDate
      const keywordQuery =
        intent.keywords && intent.keywords.length > 0 ? intent.keywords.join(' ') : null

      if (intent.app_name) newAppName = intent.app_name
      if (intent.has_ocr !== null && intent.has_ocr !== undefined) newHasOcr = intent.has_ocr
      
      if (intent.date_range) {
         const today = new Date()
         const start = new Date(today)
         const end = new Date(today)
         
         switch(intent.date_range) {
             case 'today':
                 break;
             case 'yesterday':
                 start.setDate(today.getDate() - 1)
                 end.setDate(today.getDate() - 1)
                 break;
             case 'this_week': {
                 const day = today.getDay() || 7; 
                 if(day !== 1) start.setHours(-24 * (day - 1)); 
                 break;
             }
             case 'last_week': {
                 const currentDay = today.getDay() || 7;
                 start.setDate(today.getDate() - currentDay - 6);
                 end.setDate(today.getDate() - currentDay);
                 break;
             }
             case 'this_month':
                 start.setDate(1)
                 break;
         }
         
         const fmt = (d: Date) => {
             const y = d.getFullYear()
             const m = String(d.getMonth() + 1).padStart(2, '0')
             const day = String(d.getDate()).padStart(2, '0')
             return `${y}-${m}-${day}`
         }
         
         newStartDate = fmt(start)
         newEndDate = fmt(end)
      }

      setAppName(newAppName)
      setHasOcr(newHasOcr)
      setStartDate(newStartDate)
      setEndDate(newEndDate)
      if (keywordQuery !== null) setQuery(keywordQuery)
      setShowFilters(Boolean(newAppName || newHasOcr || newStartDate || newEndDate || keywordQuery !== null))
      
      // Execute search
      let fromTs: number | undefined
      let toTs: number | undefined

      if (newStartDate) {
        const d = parseDateParts(newStartDate)
        if (d) fromTs = toShanghaiEpochSeconds(d.year, d.month, d.day, 0, 0, 0, 0)
      }
      if (newEndDate) {
        const d = parseDateParts(newEndDate)
        if (d) toTs = toShanghaiEpochSeconds(d.year, d.month, d.day, 23, 59, 59, 999)
      }

      if (fromTs !== undefined && toTs !== undefined && fromTs > toTs) {
          const tmp = fromTs
          fromTs = toTs
          toTs = tmp
      }
      
      const appliedQuery = keywordQuery ?? trimmedQuery
      const searchQuery = keywordQuery || undefined

      setSmartSearchNotice(`已应用智能搜索：${appliedQuery}`)

      await searchActivities({
        query: searchQuery,
        appName: newAppName || undefined,
        hasOcr: newHasOcr || undefined,
        fromTs,
        toTs,
        orderBy: searchQuery ? 'rank' : 'time',
      })

    } catch (e) {
      console.error("Smart search failed", e)
      setSmartSearchError(`智能搜索失败：${redactSensitive(String(e))}`)
      setSmartSearchNotice(null)
    } finally {
      setIsParsingIntent(false)
    }
  }

  const handleSearch = () => {
    const parsed = parseRuleQuery(query)

    const effectiveQuery = parsed.queryText
    const effectiveAppName = parsed.appName ?? appName
    const effectiveHasOcr = parsed.hasOcr ?? hasOcr
    const effectiveStartDate = parsed.startDate ?? startDate
    const effectiveEndDate = parsed.endDate ?? endDate

    if (parsed.appName !== undefined) setAppName(effectiveAppName)
    if (parsed.hasOcr !== undefined) setHasOcr(effectiveHasOcr)
    if (parsed.startDate !== undefined) setStartDate(effectiveStartDate)
    if (parsed.endDate !== undefined) setEndDate(effectiveEndDate)
    if (parsed.queryText !== query) setQuery(effectiveQuery)

    if (effectiveAppName || effectiveHasOcr || effectiveStartDate || effectiveEndDate) {
      setShowFilters(true)
    }

    // Convert dates to timestamps
    let fromTs: number | undefined
    let toTs: number | undefined

    if (effectiveStartDate) {
      const d = parseDateParts(effectiveStartDate)
      if (d) {
        fromTs = toShanghaiEpochSeconds(d.year, d.month, d.day, 0, 0, 0, 0)
      }
    }
    if (effectiveEndDate) {
      const d = parseDateParts(effectiveEndDate)
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
      query: effectiveQuery || undefined,
      appName: effectiveAppName || undefined,
      hasOcr: effectiveHasOcr || undefined,
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
            共 {state.activities?.length ?? 0} 条记录
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
                aria-label="清除搜索"
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
            onClick={handleSmartSearch}
            disabled={isParsingIntent || !query.trim()}
            className={`p-2 rounded-lg border border-glass-border transition-colors ${
              isParsingIntent
                ? 'bg-neon-purple/20 border-neon-purple text-neon-purple animate-pulse'
                : 'hover:bg-surface/50 text-gray-400 hover:text-neon-purple'
            }`}
            title="AI 智能搜索 (例如: '查找上周看过的 PDF')"
          >
            <Sparkles className="w-5 h-5" />
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

        {(smartSearchNotice || smartSearchError) && (
          <div
            className={`text-xs px-3 py-2 rounded-lg border ${
              smartSearchError
                ? 'border-neon-red/30 bg-neon-red/10 text-neon-red'
                : 'border-neon-purple/30 bg-neon-purple/10 text-neon-purple'
            }`}
          >
            {smartSearchError ?? smartSearchNotice}
          </div>
        )}

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
        {!state.activities || state.activities.length === 0 ? (
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

