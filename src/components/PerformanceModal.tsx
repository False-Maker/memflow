import { useState, useEffect } from 'react'
import { X, Trash2, RefreshCw } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'

interface PerformanceModalProps {
  open: boolean
  onClose: () => void
}

interface PerformanceMetrics {
  memory_usage_mb: number
  cpu_usage_percent: number
  disk_usage_mb: number
  screenshot_count: number
  activities_count: number
}

export default function PerformanceModal({ open, onClose }: PerformanceModalProps) {
  const [metrics, setMetrics] = useState<PerformanceMetrics | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (open) {
      loadMetrics()
      const interval = setInterval(loadMetrics, 5000)
      return () => clearInterval(interval)
    }
  }, [open])

  const loadMetrics = async () => {
    try {
      const data = await invoke<PerformanceMetrics>('get_performance_metrics')
      setMetrics(data)
    } catch (error) {
      console.error('获取性能指标失败:', error)
    }
  }

  const handleGC = async () => {
    try {
      setLoading(true)
      await invoke('trigger_gc')
      await loadMetrics()
      alert('垃圾回收完成')
    } catch (error) {
      console.error('垃圾回收失败:', error)
      alert('垃圾回收失败: ' + error)
    } finally {
      setLoading(false)
    }
  }

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="glass w-full max-w-2xl rounded-lg p-6 max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-white">性能监控</h2>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-surface transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {metrics ? (
          <div className="space-y-6">
            <div className="grid grid-cols-2 gap-4">
              <div className="glass p-4 rounded-lg">
                <div className="text-sm text-gray-400 mb-1">内存使用</div>
                <div className="text-2xl font-bold text-neon-blue">
                  {metrics.memory_usage_mb.toFixed(2)} MB
                </div>
              </div>
              <div className="glass p-4 rounded-lg">
                <div className="text-sm text-gray-400 mb-1">CPU 使用</div>
                <div className="text-2xl font-bold text-neon-green">
                  {metrics.cpu_usage_percent.toFixed(1)}%
                </div>
              </div>
              <div className="glass p-4 rounded-lg">
                <div className="text-sm text-gray-400 mb-1">磁盘使用</div>
                <div className="text-2xl font-bold text-neon-purple">
                  {metrics.disk_usage_mb.toFixed(2)} MB
                </div>
              </div>
              <div className="glass p-4 rounded-lg">
                <div className="text-sm text-gray-400 mb-1">截图数量</div>
                <div className="text-2xl font-bold text-white">
                  {metrics.screenshot_count}
                </div>
              </div>
            </div>

            <div className="glass p-4 rounded-lg">
              <div className="text-sm text-gray-400 mb-1">活动记录数</div>
              <div className="text-2xl font-bold text-white">
                {metrics.activities_count}
              </div>
            </div>

            <div className="flex gap-3">
              <button
                onClick={loadMetrics}
                className="flex-1 px-4 py-2 rounded-lg bg-neon-blue/20 text-neon-blue hover:bg-neon-blue/30 transition-colors flex items-center justify-center gap-2"
              >
                <RefreshCw className="w-4 h-4" />
                刷新
              </button>
              <button
                onClick={handleGC}
                disabled={loading}
                className="flex-1 px-4 py-2 rounded-lg bg-neon-red/20 text-neon-red hover:bg-neon-red/30 transition-colors flex items-center justify-center gap-2 disabled:opacity-50"
              >
                <Trash2 className="w-4 h-4" />
                {loading ? '清理中...' : '垃圾回收'}
              </button>
            </div>
          </div>
        ) : (
          <div className="text-center py-8 text-gray-400">
            <RefreshCw className="w-8 h-8 mx-auto mb-4 animate-spin" />
            <p>加载中...</p>
          </div>
        )}
      </div>
    </div>
  )
}

