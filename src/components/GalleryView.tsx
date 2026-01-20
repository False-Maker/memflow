import { useState, useEffect, useMemo, forwardRef } from 'react'
import { useApp } from '../contexts/AppContext'
import { VirtuosoGrid } from 'react-virtuoso'
import { getScreenshotUrl } from '../utils/imageLoader'
import { ActivityLog } from '../contexts/AppContext'
import { Monitor, Clock, FileText, LayoutGrid, Layers } from 'lucide-react'
import ImagePreviewModal from './ImagePreviewModal'

export default function GalleryView() {
  const { state } = useApp()
  const [previewActivity, setPreviewActivity] = useState<ActivityLog | null>(null)
  const [selectedApp, setSelectedApp] = useState<string | null>(null)

  // Calculate app statistics for the sidebar
  const appStats = useMemo(() => {
    const stats = new Map<string, number>()
    state.activities.forEach(a => {
      const name = a.appName || 'Unknown'
      stats.set(name, (stats.get(name) || 0) + 1)
    })
    return Array.from(stats.entries())
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count)
  }, [state.activities])

  // Filter activities based on selection
  const filteredActivities = useMemo(() => {
    if (!selectedApp) return state.activities
    return state.activities.filter(a => (a.appName || 'Unknown') === selectedApp)
  }, [state.activities, selectedApp])

  return (
    <div className="h-full flex bg-void overflow-hidden">
      <ImagePreviewModal
        open={previewActivity !== null}
        activity={previewActivity}
        onClose={() => setPreviewActivity(null)}
      />

      {/* Sidebar - App Visual Index */}
      <div className="w-64 flex-shrink-0 border-r border-glass-border bg-surface/30 flex flex-col">
        <div className="p-4 border-b border-glass-border">
          <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-wider flex items-center gap-2">
            <Layers className="w-4 h-4" />
            应用索引
          </h2>
        </div>
        
        <div className="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
          <button
            onClick={() => setSelectedApp(null)}
            className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-sm transition-colors ${
              selectedApp === null
                ? 'bg-neon-blue/20 text-neon-blue border border-neon-blue/50'
                : 'text-gray-300 hover:bg-surface/50 hover:text-white border border-transparent'
            }`}
          >
            <div className="flex items-center gap-2">
              <LayoutGrid className="w-4 h-4" />
              <span>全部应用</span>
            </div>
            <span className="text-xs opacity-60">{state.activities.length}</span>
          </button>

          <div className="my-2 border-t border-glass-border/50" />

          {appStats.map((app) => (
            <button
              key={app.name}
              onClick={() => setSelectedApp(app.name)}
              className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-sm transition-colors ${
                selectedApp === app.name
                  ? 'bg-neon-blue/20 text-neon-blue border border-neon-blue/50'
                  : 'text-gray-300 hover:bg-surface/50 hover:text-white border border-transparent'
              }`}
            >
              <div className="flex items-center gap-2 truncate">
                <Monitor className="w-4 h-4 flex-shrink-0 opacity-70" />
                <span className="truncate" title={app.name}>{app.name}</span>
              </div>
              <span className="text-xs opacity-60 flex-shrink-0 bg-surface/50 px-1.5 py-0.5 rounded">
                {app.count}
              </span>
            </button>
          ))}
        </div>
      </div>

      {/* Main Content - Grid */}
      <div className="flex-1 overflow-hidden p-4 flex flex-col">
        <div className="mb-4 flex items-center justify-between">
            <h3 className="text-lg font-medium text-white flex items-center gap-2">
                {selectedApp ? (
                    <>
                        <Monitor className="w-5 h-5 text-neon-blue" />
                        {selectedApp}
                    </>
                ) : (
                    <>
                        <LayoutGrid className="w-5 h-5 text-neon-blue" />
                        全部视图
                    </>
                )}
            </h3>
            <span className="text-sm text-gray-400">
                {filteredActivities.length} 张截图
            </span>
        </div>

        <div className="flex-1 border border-glass-border/30 rounded-xl bg-surface/10 overflow-hidden">
            {filteredActivities.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-500">
                <div className="text-center">
                <Monitor className="w-16 h-16 mx-auto mb-4 opacity-50" />
                <p>该分类下暂无活动记录</p>
                </div>
            </div>
            ) : (
            <VirtuosoGrid
                key={selectedApp || 'all'} // Force remount on app switch to reset scroll
                data={filteredActivities}
                totalCount={filteredActivities.length}
                overscan={200}
                components={{
                List: forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(({ style, children, ...props }, ref) => (
                    <div
                    ref={ref}
                    {...props}
                    style={{
                        display: 'grid',
                        gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))',
                        gap: '1rem',
                        padding: '1rem',
                        ...style,
                    }}
                    >
                    {children}
                    </div>
                )),
                Item: ({ children, ...props }) => (
                    <div {...props} style={{ padding: 0 }}>
                    {children}
                    </div>
                )
                }}
                itemContent={(index) => {
                const activity = filteredActivities[index]
                return (
                    <GalleryItem 
                    activity={activity} 
                    onClick={() => setPreviewActivity(activity)} 
                    />
                )
                }}
                style={{ height: '100%' }}
            />
            )}
        </div>
      </div>
    </div>
  )
}

function GalleryItem({ activity, onClick }: { activity: ActivityLog; onClick: () => void }) {
  const [imageUrl, setImageUrl] = useState<string>('')
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let mounted = true
    getScreenshotUrl(activity.imagePath).then((url) => {
      if (mounted) {
        setImageUrl(url)
        setLoading(false)
      }
    })
    return () => {
      mounted = false
    }
  }, [activity.imagePath])

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp < 1e12 ? timestamp * 1000 : timestamp)
    return date.toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp < 1e12 ? timestamp * 1000 : timestamp)
    return date.toLocaleDateString('zh-CN', {
        month: 'short',
        day: 'numeric',
    })
  }

  return (
    <div 
      className="group relative aspect-video bg-surface rounded-lg border border-glass-border overflow-hidden cursor-pointer hover:border-neon-blue hover:shadow-[0_0_15px_rgba(0,243,255,0.3)] transition-all duration-300"
      onClick={onClick}
    >
      {loading ? (
        <div className="w-full h-full flex items-center justify-center bg-surface">
          <div className="w-6 h-6 border-2 border-neon-blue border-t-transparent rounded-full animate-spin" />
        </div>
      ) : (
        <img
          src={imageUrl}
          alt={activity.windowTitle}
          className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
          loading="lazy"
          onError={(e) => {
            e.currentTarget.src =
              'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTI4IiBoZWlnaHQ9IjgwIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxyZWN0IHdpZHRoPSIxMjgiIGhlaWdodD0iODAiIGZpbGw9IiMxMjEyMTQiLz48dGV4dCB4PSI1MCUiIHk9IjUwJSIgZm9udC1mYW1pbHk9IkFyaWFsIiBmb250LXNpemU9IjE0IiBmaWxsPSIjNjY2IiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBkeT0iLjNlbSI+5Zu+54mH5pyq5Yqg6L29PC90ZXh0Pjwvc3ZnPg=='
          }}
        />
      )}
      
      {/* 悬停显示元数据 - 增强版 */}
      <div className="absolute inset-0 bg-gradient-to-t from-black/95 via-black/50 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex flex-col justify-end p-4">
        <div className="transform translate-y-4 group-hover:translate-y-0 transition-transform duration-300">
          <div className="flex items-center justify-between text-xs text-neon-blue mb-1.5">
            <span className="flex items-center gap-1.5 bg-neon-blue/10 px-2 py-0.5 rounded-full border border-neon-blue/20">
              <Clock className="w-3 h-3" />
              {formatDate(activity.timestamp)} {formatTime(activity.timestamp)}
            </span>
          </div>
          <div className="text-sm font-bold text-white truncate mb-1 drop-shadow-md">
            {activity.appName}
          </div>
          <div className="text-xs text-gray-300 line-clamp-2 leading-relaxed">
            {activity.windowTitle}
          </div>
          {activity.ocrText && (
             <div className="mt-2 flex items-center gap-1.5 text-[10px] text-neon-green/80 bg-neon-green/10 px-2 py-1 rounded w-fit border border-neon-green/20">
                <FileText className="w-3 h-3" />
                <span>OCR 已识别</span>
             </div>
          )}
        </div>
      </div>
    </div>
  )
}