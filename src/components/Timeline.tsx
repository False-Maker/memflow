import { useEffect, useState } from 'react'
import { useApp } from '../contexts/AppContext'
import { Virtuoso } from 'react-virtuoso'
import { Clock, Monitor, FileText, Search, Filter, X, Sparkles } from 'lucide-react'
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
    if (!trimmedQuery) {
      setSmartSearchError('请输入描述，例如“查找昨天看过的 React 文档”')
      setTimeout(() => setSmartSearchError(null), 3000)
      return
    }

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

        switch (intent.date_range) {
          case 'today':
            break;
          case 'yesterday':
            start.setDate(today.getDate() - 1)
            end.setDate(today.getDate() - 1)
            break;
          case 'this_week': {
            const day = today.getDay() || 7;
            if (day !== 1) start.setHours(-24 * (day - 1));
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

      <div className="border-b border-zinc-800 px-4 py-2 flex items-center gap-2 bg-void">
        {/* 标题 */}
        <h2 className="text-xs font-bold text-zinc-400 uppercase tracking-widest font-mono flex items-center gap-2 mr-2">
          <Clock className="w-3.5 h-3.5 text-neon-cyan" />
          LOG
        </h2>

        {/* 紧凑搜索框 */}
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-zinc-500" />
          <input
            type="text"
            placeholder="搜索活动记录..."
            className="w-full bg-void border border-white/10 rounded-sm pl-8 pr-8 py-1.5 text-xs font-mono text-zinc-300 placeholder:text-zinc-600 focus:outline-none focus:border-neon-cyan focus:ring-1 focus:ring-neon-cyan/50 transition-all backdrop-blur-sm"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
          />
          {query && (
            <button
              onClick={() => setQuery('')}
              className="absolute right-2 top-1/2 -translate-y-1/2"
              aria-label="Clear Search"
            >
              <X className="w-3 h-3 text-zinc-500 hover:text-zinc-300" />
            </button>
          )}
        </div>

        {/* 功能按钮组 */}
        <div className="flex items-center gap-1">
          {/* 筛选按钮 */}
          <button
            onClick={() => setShowFilters(!showFilters)}
className={`p-1.5 rounded-sm border transition-all ${
              showFilters || appName || hasOcr || startDate || endDate
                ? 'bg-neon-cyan/20 border-neon-cyan text-neon-cyan'
                : 'border-white/10 hover:border-neon-cyan/50 text-zinc-500 hover:text-neon-cyan'
            }`}
            title="筛选"
          >
            <Filter className="w-3.5 h-3.5" />
          </button>

          {/* AI 智能搜索 */}
          <button
            onClick={handleSmartSearch}
            disabled={isParsingIntent}
className={`p-1.5 rounded-sm border border-white/10 transition-all ${
              isParsingIntent
                ? 'bg-neon-cyan/20 text-neon-cyan animate-pulse cursor-wait'
                : 'text-zinc-500 hover:text-neon-cyan hover:border-neon-cyan'
            }`}
            title="AI 智能搜索"
          >
            <Sparkles className="w-3.5 h-3.5" />
          </button>

          {/* 搜索按钮 */}
          <button
            onClick={handleSearch}
            className="px-3 py-1.5 bg-zinc-100 text-black text-[10px] font-bold uppercase tracking-wider rounded-sm hover:bg-white transition-colors"
          >
            搜索
          </button>

          {/* 重置 */}
          {(query || appName || hasOcr || startDate || endDate) && (
            <button
              onClick={clearSearch}
              className="px-2 py-1.5 text-zinc-500 text-[10px] font-mono uppercase hover:text-zinc-300 transition-colors"
            >
              重置
            </button>
          )}
        </div>

        {/* 计数 */}
        <div className="text-[10px] text-zinc-600 font-mono hidden sm:block">
          {state.activities?.length ?? 0} 条
        </div>
      </div>

      {/* AI 搜索提示 */}
      {(smartSearchNotice || smartSearchError) && (
        <div
className={`mx-4 mt-2 text-[10px] px-3 py-2 rounded-lg border ${
            smartSearchError
              ? 'border-red-500/30 bg-red-500/10 text-red-400'
              : 'border-neon-cyan/30 bg-neon-cyan/10 text-neon-cyan'
          }`}
        >
          {smartSearchError ?? smartSearchNotice}
        </div>
      )}

      {/* 折叠的筛选面板 */}
      {showFilters && (
        <div className="px-4 py-3 bg-zinc-900/30 border-b border-zinc-800 animate-in slide-in-from-top-2 duration-200">
          <div className="grid grid-cols-4 gap-3 text-xs">
            {/* 应用名称 */}
            <div className="space-y-1">
              <label className="text-[10px] text-zinc-500 font-mono uppercase">应用</label>
              <input
                type="text"
                placeholder="Chrome"
                className="w-full bg-void border border-white/10 rounded px-2 py-1.5 text-zinc-300 placeholder:text-zinc-600 focus:border-neon-cyan focus:ring-1 focus:ring-neon-cyan/50 transition-all backdrop-blur-sm"
                value={appName}
                onChange={(e) => setAppName(e.target.value)}
              />
            </div>

            {/* 开始日期 */}
            <div className="space-y-1">
              <label className="text-[10px] text-zinc-500 font-mono uppercase">从</label>
              <input
                type="date"
                className="w-full bg-void border border-white/10 rounded px-2 py-1.5 text-zinc-300 [color-scheme:dark] focus:border-neon-cyan focus:ring-1 focus:ring-neon-cyan/50 transition-all backdrop-blur-sm"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
              />
            </div>

            {/* 结束日期 */}
            <div className="space-y-1">
              <label className="text-[10px] text-zinc-500 font-mono uppercase">至</label>
              <input
                type="date"
                className="w-full bg-void border border-white/10 rounded px-2 py-1.5 text-zinc-300 [color-scheme:dark] focus:border-neon-cyan focus:ring-1 focus:ring-neon-cyan/50 transition-all backdrop-blur-sm"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
              />
            </div>

            {/* OCR 筛选 */}
            <div className="space-y-1">
              <label className="text-[10px] text-zinc-500 font-mono uppercase">选项</label>
              <label className="flex items-center gap-2 cursor-pointer h-full pt-1.5">
                <input
                  type="checkbox"
                  className="rounded border-white/10 bg-void accent-neon-cyan"
                  checked={hasOcr}
                  onChange={(e) => setHasOcr(e.target.checked)}
                />
                <span className="text-zinc-400">含 OCR</span>
              </label>
            </div>
          </div>
        </div>
      )}

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
<div className="glass mx-6 mb-3 p-4 border border-white/10 bg-void hover:bg-neon-cyan/5 transition-all rounded-sm group relative hover:shadow-[0_0_15px_rgba(0,240,255,0.3)]">
                    <div className="absolute left-0 top-0 bottom-0 w-1 bg-transparent group-hover:bg-neon-cyan transition-colors"></div>
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
                          <Monitor className="w-4 h-4 text-zinc-500 flex-shrink-0" />
                          <span className="font-bold text-zinc-200 truncate font-mono text-sm uppercase tracking-wide">
                            {activity.appName}
                          </span>
                          <span className="text-xs text-zinc-600 font-mono">
                            {formatTime(activity.timestamp)}
                          </span>
                        </div>

                        <div className="text-xs text-zinc-400 mb-2 truncate font-sans pl-6 border-lborder-zinc-800">
                          {activity.windowTitle}
                        </div>

                        {activity.ocrText && (
                          <div className="flex items-start gap-2 mt-2 pl-6">
                            <FileText className="w-3 h-3 text-zinc-600 flex-shrink-0 mt-0.5" />
                            <p className="text-xs text-zinc-500 line-clamp-2 font-mono">
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
      <div className="w-32 h-20 bg-zinc-900 rounded-sm border border-zinc-800 flex items-center justify-center">
        <div className="w-4 h-4 border-2 border-neon-cyan border-t-transparent rounded-full animate-spin" />
      </div>
    )
  }

  return (
    <img
      src={imageUrl}
      alt="Screenshot"
      className="w-32 h-20 object-cover rounded-sm border border-zinc-800 cursor-pointer hover:opacity-80 transition-opacity"
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

