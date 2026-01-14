import { useEffect, useState, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useApp } from '../contexts/AppContext'
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell, Legend } from 'recharts'
import { Activity, Clock, Monitor, Loader2, ShieldAlert } from 'lucide-react'

interface Stats {
  totalActivities: number
  totalHours: number
  topApp: string
}

interface RecordingStat {
  date: string
  reason: string
  count: number
}

export default function FlowState() {
  const { state } = useApp()
  const [stats, setStats] = useState<Stats>({
    totalActivities: 0,
    totalHours: 0,
    topApp: '未知',
  })
  const [recordingStats, setRecordingStats] = useState<RecordingStat[]>([])
  const [loading, setLoading] = useState(true)

  // 从后端获取统计数据
  useEffect(() => {
    const fetchStats = async () => {
      try {
        const [basicStats, recStats] = await Promise.all([
          invoke<Stats>('get_stats'),
          invoke<RecordingStat[]>('get_recording_stats', { limit: 30 })
        ])
        setStats(basicStats)
        setRecordingStats(recStats)
      } catch (e) {
        console.error('获取统计数据失败:', e)
      } finally {
        setLoading(false)
      }
    }
    fetchStats()
  }, [])

  // Process recording stats for chart
  const skippedStatsData = useMemo(() => {
    // Group by reason
    const reasonCounts: Record<string, number> = {}
    recordingStats.forEach(stat => {
      reasonCounts[stat.reason] = (reasonCounts[stat.reason] || 0) + stat.count
    })
    
    return Object.entries(reasonCounts).map(([name, value]) => ({
      name: name === 'privacy_mode' ? '隐私模式' : 
            name === 'blocklist' ? '黑名单' : 
            name === 'allowlist_miss' ? '非白名单' : name,
      value
    }))
  }, [recordingStats])

  // 从活动数据计算应用使用分布（真实数据）
  const appUsageData = useMemo(() => {
    const appCounts: Record<string, number> = {}
    for (const activity of state.activities) {
      const appName = activity.appName || '未知'
      appCounts[appName] = (appCounts[appName] || 0) + 1
    }
    
    const sorted = Object.entries(appCounts)
      .map(([name, value]) => ({ name, value }))
      .sort((a, b) => b.value - a.value)
    
    // 取前 5 个，其余归为「其他」
    if (sorted.length <= 5) return sorted
    const top5 = sorted.slice(0, 5)
    const othersValue = sorted.slice(5).reduce((sum, item) => sum + item.value, 0)
    if (othersValue > 0) {
      top5.push({ name: '其他', value: othersValue })
    }
    return top5
  }, [state.activities])

  // 从活动数据计算 24 小时活动分布（真实数据）
  const hourlyData = useMemo(() => {
    const hourlyCounts = Array(24).fill(0)
    for (const activity of state.activities) {
      // 后端时间戳是秒级
      const ts = activity.timestamp < 1e12 ? activity.timestamp * 1000 : activity.timestamp
      const hour = new Date(ts).getHours()
      hourlyCounts[hour]++
    }
    return hourlyCounts.map((count, i) => ({
      hour: `${i}:00`,
      activities: count,
    }))
  }, [state.activities])

  const COLORS = ['#2DE2E6', '#9D4EDD', '#02C39A', '#FF3864', '#F77F00', '#D62828']

  return (
    <div className="h-full overflow-y-auto p-6">
      <div className="glass border-b border-glass-border px-6 py-4 mb-6">
        <h2 className="text-lg font-semibold text-neon-green">活动统计</h2>
      </div>

      {/* 统计卡片 */}
      <div className="grid grid-cols-3 gap-4 mb-6">
        <div className="glass p-6 rounded-lg">
          <div className="flex items-center gap-3 mb-2">
            <Activity className="w-6 h-6 text-neon-blue" />
            <span className="text-sm text-gray-400">总活动数</span>
          </div>
          {loading ? (
            <Loader2 className="w-6 h-6 animate-spin text-neon-blue" />
          ) : (
            <div className="text-3xl font-bold text-white">{stats.totalActivities}</div>
          )}
        </div>

        <div className="glass p-6 rounded-lg">
          <div className="flex items-center gap-3 mb-2">
            <Clock className="w-6 h-6 text-neon-purple" />
            <span className="text-sm text-gray-400">累计时长</span>
          </div>
          {loading ? (
            <Loader2 className="w-6 h-6 animate-spin text-neon-purple" />
          ) : (
            <div className="text-3xl font-bold text-white">
              {stats.totalHours < 1 
                ? `${Math.round(stats.totalHours * 60)}m` 
                : `${stats.totalHours.toFixed(1)}h`}
            </div>
          )}
        </div>

        <div className="glass p-6 rounded-lg">
          <div className="flex items-center gap-3 mb-2">
            <Monitor className="w-6 h-6 text-neon-green" />
            <span className="text-sm text-gray-400">最常用应用</span>
          </div>
          {loading ? (
            <Loader2 className="w-6 h-6 animate-spin text-neon-green" />
          ) : (
            <div className="text-xl font-semibold text-white truncate">{stats.topApp}</div>
          )}
        </div>
      </div>

      {/* 应用使用分布 */}
      <div className="glass p-6 rounded-lg mb-6">
        <h3 className="text-md font-semibold text-white mb-4">应用使用分布</h3>
        {appUsageData.length === 0 ? (
          <div className="h-[300px] flex items-center justify-center text-gray-500">
            <div className="text-center">
              <Monitor className="w-12 h-12 mx-auto mb-2 opacity-50" />
              <p>暂无应用使用数据</p>
              <p className="text-sm mt-1">开始录制后数据将显示在这里</p>
            </div>
          </div>
        ) : (
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={appUsageData}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={({ name, percent }) =>
                  `${name} ${(((percent ?? 0) as number) * 100).toFixed(0)}%`
                }
                outerRadius={100}
                fill="#8884d8"
                dataKey="value"
              >
                {appUsageData.map((_, index) => (
                  <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
        )}
      </div>

      {/* 小时活动分布 */}
      <div className="glass p-6 rounded-lg mb-6">
        <h3 className="text-md font-semibold text-white mb-4">24小时活动分布</h3>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart data={hourlyData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#333" />
            <XAxis dataKey="hour" stroke="#666" />
            <YAxis stroke="#666" />
            <Tooltip
              contentStyle={{
                backgroundColor: '#121214',
                border: '1px solid rgba(255, 255, 255, 0.08)',
                borderRadius: '8px',
              }}
            />
            <Bar dataKey="activities" fill="#2DE2E6" radius={[4, 4, 0, 0]} />
          </BarChart>
        </ResponsiveContainer>
      </div>

      {/* 跳过录制统计 */}
      {skippedStatsData.length > 0 && (
        <div className="glass p-6 rounded-lg">
          <div className="flex items-center gap-2 mb-4">
             <ShieldAlert className="w-5 h-5 text-orange-500" />
             <h3 className="text-md font-semibold text-white">隐私保护拦截统计</h3>
          </div>
          <div className="grid grid-cols-2 gap-4">
             <div className="h-[250px]">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={skippedStatsData}
                      cx="50%"
                      cy="50%"
                      innerRadius={60}
                      outerRadius={80}
                      paddingAngle={5}
                      dataKey="value"
                    >
                      {skippedStatsData.map((_, index) => (
                        <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
             </div>
             <div className="flex flex-col justify-center space-y-4">
                {skippedStatsData.map((item, index) => (
                  <div key={item.name} className="flex items-center justify-between p-3 bg-surface/30 rounded-lg border border-glass-border">
                    <div className="flex items-center gap-3">
                      <div className="w-3 h-3 rounded-full" style={{ backgroundColor: COLORS[index % COLORS.length] }} />
                      <span className="text-gray-300">{item.name}</span>
                    </div>
                    <span className="text-xl font-bold text-white">{item.value}</span>
                  </div>
                ))}
             </div>
          </div>
        </div>
      )}
    </div>
  )
}

