import { useEffect, useRef, useState, useMemo } from 'react'
import ForceGraph2D from 'react-force-graph-2d'
import { invoke } from '@tauri-apps/api/core'
import { RefreshCw } from 'lucide-react'

interface GraphNode {
  id: string
  name: string
  group: string
  size: number
  x?: number
  y?: number
}

interface GraphEdge {
  source: string
  target: string
  value: number
}

// 后端返回的数据格式
interface BackendGraphData {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

// react-force-graph-2d 需要的数据格式
interface ForceGraphData {
  nodes: GraphNode[]
  links: GraphEdge[]
}

export default function KnowledgeGraph() {
  const graphRef = useRef<any>()
  const containerRef = useRef<HTMLDivElement>(null)
  const [rawGraphData, setRawGraphData] = useState<BackendGraphData>({ nodes: [], edges: [] })
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [lastAction, setLastAction] = useState<'load' | 'rebuild'>('load')
  const [notice, setNotice] = useState<string | null>(null)
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 })

  // 将后端数据转换为 force-graph 需要的格式
  const graphData: ForceGraphData = useMemo(() => ({
    nodes: rawGraphData.nodes,
    links: rawGraphData.edges || [],
  }), [rawGraphData])

  // 更新容器尺寸
  useEffect(() => {
    const updateDimensions = () => {
      if (containerRef.current) {
        const rect = containerRef.current.getBoundingClientRect()
        // 确保尺寸有效
        if (rect.width > 0 && rect.height > 0) {
          setDimensions({
            width: Math.floor(rect.width),
            height: Math.floor(rect.height),
          })
        }
      }
    }

    // 延迟执行以确保 DOM 已完成布局
    const timer = setTimeout(updateDimensions, 100)
    window.addEventListener('resize', updateDimensions)
    
    // 使用 ResizeObserver 监听容器尺寸变化
    const resizeObserver = new ResizeObserver(() => {
      // 防抖处理
      requestAnimationFrame(updateDimensions)
    })
    if (containerRef.current) {
      resizeObserver.observe(containerRef.current)
    }

    return () => {
      clearTimeout(timer)
      window.removeEventListener('resize', updateDimensions)
      resizeObserver.disconnect()
    }
  }, []) // 容器挂载后开始监听

  const loadGraph = async () => {
    try {
      setLastAction('load')
      setLoading(true)
      setError(null)
      setNotice(null)
      const data = await invoke<BackendGraphData>('get_graph_data')
      setRawGraphData(data || { nodes: [], edges: [] })
    } catch (error) {
      console.error('加载图谱失败:', error)
      setError(error instanceof Error ? error.message : String(error))
    } finally {
      setLoading(false)
    }
  }

  const rebuildGraph = async () => {
    try {
      setLastAction('rebuild')
      setLoading(true)
      setError(null)
      setNotice(null)
      const data = await invoke<BackendGraphData>('rebuild_graph')
      const safeData = data || { nodes: [], edges: [] }
      setRawGraphData(safeData)
      setNotice(`图谱已重建：${safeData.nodes.length} 个节点，${(safeData.edges || []).length} 条边`)
    } catch (error) {
      console.error('重建图谱失败:', error)
      setError(error instanceof Error ? error.message : String(error))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadGraph()
  }, [])

  // 当图谱数据变化时，延迟后自动缩放适应
  useEffect(() => {
    if (graphRef.current && graphData.nodes.length > 0) {
      // 等待图谱渲染完成后缩放
      const timer = setTimeout(() => {
        graphRef.current?.zoomToFit(400, 50)
      }, 500)
      return () => clearTimeout(timer)
    }
  }, [graphData])

  // 渲染内容区域
  const renderContent = () => {
    if (loading) {
      return (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center">
            <RefreshCw className="w-8 h-8 mx-auto mb-4 text-neon-purple animate-spin" />
            <p className="text-gray-400">
              {lastAction === 'rebuild' ? '正在重建知识图谱...' : '加载知识图谱...'}
            </p>
          </div>
        </div>
      )
    }

    if (error) {
      return (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center">
            <p className="text-red-400 mb-4">错误: {error}</p>
            <button
              onClick={lastAction === 'rebuild' ? rebuildGraph : loadGraph}
              className="px-4 py-2 rounded-lg bg-neon-purple/20 text-neon-purple hover:bg-neon-purple/30 transition-colors"
            >
              重试
            </button>
          </div>
        </div>
      )
    }

    if (graphData.nodes.length === 0) {
      return (
        <div className="absolute inset-0 flex items-center justify-center text-gray-500">
          <div className="text-center">
            <p>暂无图谱数据</p>
            <p className="text-sm mt-2">开始录制后，图谱将自动生成</p>
          </div>
        </div>
      )
    }

    if (dimensions.width > 0 && dimensions.height > 0) {
      return (
        <ForceGraph2D
          ref={graphRef}
          graphData={graphData}
          nodeLabel={(node: any) => node.name}
          nodeColor={(node: any) => {
            const colors: Record<string, string> = {
              app: '#2DE2E6',
              doc: '#9D4EDD',
              time: '#02C39A',
            }
            return colors[node.group] || '#666'
          }}
          nodeVal={(node: any) => Math.sqrt(node.size || 1) * 5}
          linkColor={() => 'rgba(255, 255, 255, 0.2)'}
          linkWidth={(link: any) => Math.sqrt(link.value || 1)}
          backgroundColor="#0a0a0a"
          width={dimensions.width}
          height={dimensions.height}
          minZoom={0.1}
          maxZoom={10}
          onEngineStop={() => {
            if (graphRef.current) {
              graphRef.current.zoomToFit(400, 50)
            }
          }}
        />
      )
    }

    return null
  }

  return (
    <div className="h-full flex flex-col">
      <div className="glass border-b border-glass-border px-6 py-4 flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-neon-purple">
            知识图谱
          </h2>
          <p className="text-sm text-gray-400 mt-1">
            可视化活动关联关系 ({graphData.nodes.length} 个节点, {graphData.links.length} 条边)
          </p>
          <p className="text-xs text-gray-500 mt-2">
            使用方法：先开始录制并产生活动，再点“重建图谱”；开启 OCR 时会出现更多关键词节点。
          </p>
        </div>
        <button
          onClick={rebuildGraph}
          disabled={loading}
          className="px-4 py-2 rounded-lg bg-neon-purple/20 text-neon-purple hover:bg-neon-purple/30 transition-colors flex items-center gap-2 disabled:opacity-50"
        >
          <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          重建图谱
        </button>
      </div>

      {notice && (
        <div className="px-6 py-3 border-b border-glass-border text-sm text-neon-purple bg-neon-purple/5">
          {notice}
        </div>
      )}

      <div 
        ref={containerRef} 
        className="flex-1 relative bg-[#0a0a0a]" 
        style={{ minHeight: 0, overflow: 'hidden' }}
      >
        {renderContent()}
      </div>
    </div>
  )
}
