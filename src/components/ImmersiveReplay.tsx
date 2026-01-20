import { useEffect, useMemo, useRef, useState } from 'react'
import { useApp } from '../contexts/AppContext'
import { getScreenshotUrls, getScreenshotUrl } from '../utils/imageLoader'
import { Pause, Play, SkipBack, SkipForward } from 'lucide-react'
import { Virtuoso, VirtuosoHandle } from 'react-virtuoso'
import { ActivityLog } from '../contexts/AppContext'

type ReplaySpeed = 1 | 2 | 5

export default function ImmersiveReplay() {
  const { state } = useApp()
  const [isPlaying, setIsPlaying] = useState(false)
  const [speed, setSpeed] = useState<ReplaySpeed>(1)
  const [currentIndex, setCurrentIndex] = useState(0)
  const [currentUrl, setCurrentUrl] = useState<string>('')
  
  // Ref for filmstrip auto-scrolling
  const filmstripRef = useRef<VirtuosoHandle>(null)

  const activities = useMemo(() => {
    const sorted = [...state.activities].sort((a, b) => a.timestamp - b.timestamp)
    return sorted.filter((a) => a.imagePath)
  }, [state.activities])

  const activitiesRef = useRef(activities)
  useEffect(() => {
    activitiesRef.current = activities
  }, [activities])

  useEffect(() => {
    if (activities.length === 0) {
      setIsPlaying(false)
      setCurrentIndex(0)
      setCurrentUrl('')
      return
    }
    if (currentIndex > activities.length - 1) {
      setCurrentIndex(activities.length - 1)
    }
    // Auto-scroll filmstrip to keep current frame in view
    filmstripRef.current?.scrollToIndex({ index: currentIndex, align: 'center', behavior: 'auto' })
  }, [activities.length, currentIndex])

  const isPlayingRef = useRef(isPlaying)
  useEffect(() => {
    isPlayingRef.current = isPlaying
  }, [isPlaying])

  const speedRef = useRef(speed)
  useEffect(() => {
    speedRef.current = speed
  }, [speed])

  const rafRef = useRef<number | null>(null)
  const lastTsRef = useRef<number | null>(null)
  const accRef = useRef(0)

  useEffect(() => {
    if (!isPlaying) {
      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current)
        rafRef.current = null
      }
      lastTsRef.current = null
      accRef.current = 0
      return
    }

    const baseFps = 12
    const tick = (now: number) => {
      if (!isPlayingRef.current) return

      const last = lastTsRef.current ?? now
      const delta = now - last
      lastTsRef.current = now
      accRef.current += delta

      const frameMs = 1000 / (baseFps * speedRef.current)
      while (accRef.current >= frameMs) {
        accRef.current -= frameMs
        setCurrentIndex((prev) => {
          const maxIndex = activitiesRef.current.length - 1
          if (maxIndex < 0) return 0
          if (prev >= maxIndex) {
            setTimeout(() => setIsPlaying(false), 0)
            return maxIndex
          }
          return prev + 1
        })
      }

      rafRef.current = requestAnimationFrame(tick)
    }

    rafRef.current = requestAnimationFrame(tick)
    return () => {
      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current)
        rafRef.current = null
      }
    }
  }, [isPlaying])

  useEffect(() => {
    const activity = activities[currentIndex]
    if (!activity) {
      setCurrentUrl('')
      return
    }

    const radius = 10
    const start = Math.max(0, currentIndex - radius)
    const end = Math.min(activities.length - 1, currentIndex + radius)
    const imagePaths = activities.slice(start, end + 1).map((a) => a.imagePath)

    let cancelled = false
    ;(async () => {
      const urls = await getScreenshotUrls(imagePaths)
      if (cancelled) return

      const targetIndex = currentIndex - start
      const url = urls[targetIndex] ?? ''
      setCurrentUrl(url)
    })()

    return () => {
      cancelled = true
    }
  }, [activities, currentIndex])

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

  const current = activities[currentIndex]
  const total = activities.length

  const seek = (idx: number) => {
    if (total === 0) return
    const next = Math.max(0, Math.min(total - 1, idx))
    setCurrentIndex(next)
  }

  const togglePlay = () => {
    if (total === 0) return
    if (currentIndex >= total - 1) {
      setCurrentIndex(0)
      setIsPlaying(true)
      return
    }
    setIsPlaying((p) => !p)
  }

  // Keyboard controls
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if typing in an input
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return

      switch (e.key) {
        case ' ':
        case 'k':
          e.preventDefault()
          togglePlay()
          break
        case 'ArrowLeft':
          e.preventDefault()
          seek(currentIndex - (e.shiftKey ? 10 : 1))
          break
        case 'ArrowRight':
          e.preventDefault()
          seek(currentIndex + (e.shiftKey ? 10 : 1))
          break
        case 'Home':
          e.preventDefault()
          seek(0)
          break
        case 'End':
          e.preventDefault()
          seek(total - 1)
          break
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [currentIndex, total, isPlaying])

  // Detect time jumps
  const timeJump = useMemo(() => {
    if (currentIndex === 0) return null
    const prev = activities[currentIndex - 1]
    const curr = activities[currentIndex]
    if (!prev || !curr) return null
    
    const diff = curr.timestamp - prev.timestamp
    // Consider > 5 minutes as a significant jump
    const threshold = 5 * 60
    
    if (diff > threshold) {
      const minutes = Math.floor(diff / 60)
      const hours = Math.floor(minutes / 60)
      return hours > 0 
        ? `${hours}小时 ${minutes % 60}分钟` 
        : `${minutes}分钟`
    }
    return null
  }, [currentIndex, activities])

  if (total === 0) {
    return (
      <div className="h-full flex items-center justify-center bg-void text-gray-500">
        <div className="text-center">
          <div className="text-lg font-semibold text-gray-300">沉浸式时光机</div>
          <div className="mt-2 text-sm">暂无可回放的截图</div>
        </div>
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col bg-void">
      <div className="px-4 py-3 border-b border-glass-border flex items-center justify-between">
        <div className="min-w-0">
          <div className="text-sm font-semibold text-white">沉浸式时光机</div>
          <div className="text-xs text-gray-400 truncate">
            {current ? `${current.appName} · ${formatTime(current.timestamp)}` : ''}
          </div>
        </div>
        <div className="text-xs text-gray-500 tabular-nums">
          {Math.min(currentIndex + 1, total)}/{total}
        </div>
      </div>

      <div className="flex-1 min-h-0 flex flex-col relative group/player">
        {/* Main Viewer */}
        <div className="flex-1 min-h-0 flex items-center justify-center p-4 relative bg-black/40">
          {/* Time Jump Indicator */}
          {timeJump && (
             <div className="absolute top-8 bg-black/60 backdrop-blur-md text-white px-4 py-2 rounded-full border border-glass-border animate-in fade-in slide-in-from-top-4 flex items-center gap-2 shadow-lg z-10">
               <div className="w-2 h-2 rounded-full bg-yellow-400 animate-pulse" />
               <span className="text-sm font-medium">已跳过 {timeJump} 静默期</span>
             </div>
          )}
          
          {currentUrl ? (
            <img
              src={currentUrl}
              alt={current?.windowTitle ?? 'replay'}
              className="max-w-full max-h-full object-contain rounded-lg shadow-2xl"
              draggable={false}
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center">
              <div className="w-8 h-8 border-2 border-neon-blue border-t-transparent rounded-full animate-spin" />
            </div>
          )}
        </div>

        {/* Bottom Controls Area */}
        <div className="bg-surface/90 backdrop-blur border-t border-glass-border">
            
            {/* Real-time Density Timeline */}
            <div className="px-4 pt-2">
                <TimeDensityCanvas 
                    activities={activities} 
                    currentIndex={currentIndex} 
                    onSeek={seek} 
                />
            </div>

            {/* Controls Row */}
            <div className="px-4 py-2 flex items-center justify-between gap-4">
                <div className="flex items-center gap-2">
                    <button
                        onClick={() => seek(0)}
                        className="p-1.5 rounded-lg text-gray-300 hover:text-white hover:bg-white/10 transition-all hidden sm:block"
                        title="Home"
                    >
                        <SkipBack className="w-4 h-4" />
                    </button>
                    <button
                        onClick={() => seek(currentIndex - 1)}
                        className="p-1.5 rounded-lg text-gray-300 hover:text-white hover:bg-white/10 transition-all"
                        title="Previous"
                    >
                        <SkipBack className="w-4 h-4 rotate-180" />
                    </button>
                    <button
                        onClick={togglePlay}
                        className="p-2 rounded-full bg-neon-blue text-black hover:bg-neon-blue/90 transition-all mx-1"
                        title="Space"
                    >
                        {isPlaying ? <Pause className="w-4 h-4 fill-current" /> : <Play className="w-4 h-4 fill-current pl-0.5" />}
                    </button>
                    <button
                        onClick={() => seek(currentIndex + 1)}
                        className="p-1.5 rounded-lg text-gray-300 hover:text-white hover:bg-white/10 transition-all"
                        title="Next"
                    >
                        <SkipForward className="w-4 h-4" />
                    </button>
                    <button
                        onClick={() => seek(total - 1)}
                        className="p-1.5 rounded-lg text-gray-300 hover:text-white hover:bg-white/10 transition-all hidden sm:block"
                        title="End"
                    >
                        <SkipForward className="w-4 h-4 rotate-180" />
                    </button>
                </div>

                <div className="flex items-center gap-1 bg-black/20 rounded-lg p-1">
                    {[1, 2, 5].map((s) => (
                        <button
                        key={s}
                        onClick={() => setSpeed(s as ReplaySpeed)}
                        className={`px-2 py-0.5 rounded text-xs font-medium transition-all ${
                            speed === s
                            ? 'bg-neon-blue/20 text-neon-blue shadow-sm'
                            : 'text-gray-400 hover:text-white hover:bg-white/5'
                        }`}
                        >
                        {s}x
                        </button>
                    ))}
                </div>
            </div>

            {/* Filmstrip - Virtualized Horizontal List */}
            <div className="h-20 border-t border-glass-border/50 bg-black/20 relative">
                 <Virtuoso
                    ref={filmstripRef}
                    horizontalDirection
                    data={activities}
                    initialTopMostItemIndex={Math.max(0, currentIndex - 5)}
                    itemContent={(index, activity) => (
                        <div className="py-2 px-1 h-full flex items-center justify-center">
                            <Thumbnail 
                                activity={activity} 
                                active={index === currentIndex} 
                                onClick={() => seek(index)} 
                            />
                        </div>
                    )}
                    style={{ height: '100%' }}
                    className="no-scrollbar"
                />
                 {/* Center Marker Line */}
                 <div className="absolute top-0 bottom-0 left-1/2 w-px bg-neon-blue/50 pointer-events-none z-10 hidden" />
            </div>
        </div>
      </div>
    </div>
  )
}

// -----------------------------------------------------------------------------
// Sub-components
// -----------------------------------------------------------------------------

function Thumbnail({ activity, active, onClick }: { activity: ActivityLog, active: boolean, onClick: () => void }) {
  const [url, setUrl] = useState('')
  const [loaded, setLoaded] = useState(false)
  
  useEffect(() => {
    let mounted = true
    getScreenshotUrl(activity.imagePath).then(u => {
        if (mounted) {
            setUrl(u)
            setLoaded(true)
        }
    })
    return () => { mounted = false }
  }, [activity.imagePath])

  return (
    <div 
      className={`relative h-14 aspect-video flex-shrink-0 cursor-pointer rounded-md overflow-hidden transition-all duration-200 group ${
          active 
            ? 'ring-2 ring-neon-blue ring-offset-1 ring-offset-black scale-105 z-10' 
            : 'opacity-50 hover:opacity-100 hover:scale-105 grayscale hover:grayscale-0'
      }`}
      onClick={onClick}
    >
       {loaded ? (
           <img src={url} className="w-full h-full object-cover" loading="lazy" />
       ) : (
           <div className="w-full h-full bg-surface/50 animate-pulse" />
       )}
       {/* App Icon / Overlay */}
       <div className="absolute inset-0 bg-gradient-to-t from-black/80 to-transparent opacity-0 group-hover:opacity-100 transition-opacity flex items-end p-1">
            <span className="text-[10px] text-white truncate w-full">{activity.appName}</span>
       </div>
    </div>
  )
}

function TimeDensityCanvas({ activities, currentIndex, onSeek }: { activities: ActivityLog[], currentIndex: number, onSeek: (index: number) => void }) {
    const canvasRef = useRef<HTMLCanvasElement>(null)
    const containerRef = useRef<HTMLDivElement>(null)
    
    // Draw heatmap
    useEffect(() => {
        const canvas = canvasRef.current
        const container = containerRef.current
        if (!canvas || !container || activities.length === 0) return
        
        const ctx = canvas.getContext('2d')
        if (!ctx) return
        
        // Resize canvas to match container
        const dpr = window.devicePixelRatio || 1
        const rect = container.getBoundingClientRect()
        canvas.width = rect.width * dpr
        canvas.height = rect.height * dpr
        canvas.style.width = `${rect.width}px`
        canvas.style.height = `${rect.height}px`
        ctx.scale(dpr, dpr)
        
        const width = rect.width
        const height = rect.height
        
        ctx.clearRect(0, 0, width, height)
        
        const startTs = activities[0].timestamp
        const endTs = activities[activities.length - 1].timestamp
        const duration = endTs - startTs || 1
        
        // 1. Draw Background Track
        ctx.fillStyle = 'rgba(255, 255, 255, 0.05)'
        ctx.fillRect(0, height / 2 - 2, width, 4)
        
        // 2. Draw Activity Density (Vertical lines)
        ctx.fillStyle = 'rgba(0, 243, 255, 0.4)' // Neon Blue
        
        // Optimization: bucketize to prevent drawing thousands of overlapping lines
        // 1 pixel = 1 bucket
        const buckets = new Float32Array(Math.ceil(width))
        activities.forEach(a => {
            const x = Math.floor(((a.timestamp - startTs) / duration) * width)
            if (x >= 0 && x < buckets.length) {
                buckets[x] += 0.2 // opacity accumulator
            }
        })
        
        for (let x = 0; x < buckets.length; x++) {
            const density = buckets[x]
            if (density > 0) {
                // Clamp opacity between 0.2 and 1
                const alpha = Math.min(1, Math.max(0.2, density))
                ctx.fillStyle = `rgba(0, 243, 255, ${alpha})`
                // Draw a bar, taller if denser
                const h = Math.min(height, 8 + density * 4) 
                ctx.fillRect(x, (height - h) / 2, 1, h)
            }
        }
        
        // 3. Draw Current Position Indicator
        const currentA = activities[currentIndex]
        if (currentA) {
             const x = ((currentA.timestamp - startTs) / duration) * width
             
             // Glow
             ctx.shadowBlur = 8
             ctx.shadowColor = 'white'
             ctx.fillStyle = '#ffffff'
             
             // Cursor line
             ctx.fillRect(x - 1, 4, 2, height - 8)
             
             // Top Triangle
             ctx.beginPath()
             ctx.moveTo(x, 0)
             ctx.lineTo(x - 4, 6)
             ctx.lineTo(x + 4, 6)
             ctx.fill()
             
             ctx.shadowBlur = 0
        }
        
    }, [activities, currentIndex])

    const handleClick = (e: React.MouseEvent) => {
        if (activities.length === 0) return
        const rect = e.currentTarget.getBoundingClientRect()
        const x = e.clientX - rect.left
        const ratio = Math.max(0, Math.min(1, x / rect.width))
        
        const startTs = activities[0].timestamp
        const endTs = activities[activities.length - 1].timestamp
        const targetTs = startTs + ratio * (endTs - startTs)
        
        // Find nearest
        let nearestIdx = 0
        let minDiff = Infinity
        
        // Linear scan is fast enough for <10k items
        for(let i=0; i<activities.length; i++) {
            const diff = Math.abs(activities[i].timestamp - targetTs)
            if (diff < minDiff) {
                minDiff = diff
                nearestIdx = i
            } else if (diff > minDiff) {
                // Since sorted, we can break early once diff starts increasing
                // But timestamps might have small jitter, so let's be safe or just scan all
                // Break optimization:
                break 
            }
        }
        onSeek(nearestIdx)
    }

    return (
        <div ref={containerRef} className="h-10 w-full cursor-pointer relative group" onClick={handleClick}>
            <canvas ref={canvasRef} className="absolute inset-0" />
            {/* Hover effect could be added here */}
        </div>
    )
}
