import { useState } from 'react'
import { ThumbsUp, ThumbsDown } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'

interface MessageRatingProps {
  messageId: number | undefined  // 数据库中的消息 ID
  currentRating?: 1 | -1 | null
  onRatingChange?: (rating: 1 | -1 | null) => void
}

export default function MessageRating({ messageId, currentRating, onRatingChange }: MessageRatingProps) {
  const [rating, setRating] = useState<1 | -1 | null>(currentRating ?? null)
  const [loading, setLoading] = useState(false)

  const handleRate = async (newRating: 1 | -1) => {
    if (!messageId || loading) return
    
    setLoading(true)
    try {
      // 如果点击已选中的评价，则取消评价
      const finalRating = rating === newRating ? null : newRating
      
      if (finalRating === null) {
        // 取消评价：这里暂时保持原评价，后端暂未实现删除评价
        // 实际上我们用 rate_message 覆盖即可
      }
      
      await invoke('rate_message', {
        messageId,
        rating: finalRating ?? 0,  // 0 表示无评价
        comment: null,
      })
      
      setRating(finalRating)
      onRatingChange?.(finalRating)
    } catch (e) {
      console.error('评价失败:', e)
    } finally {
      setLoading(false)
    }
  }

  // 如果没有消息 ID，不显示评价按钮
  if (!messageId) return null

  return (
    <div className="flex items-center gap-1 pt-2 border-t border-glass-border/30 mt-2">
      <button
        onClick={() => handleRate(1)}
        disabled={loading}
        className={`flex items-center gap-1 px-2 py-1 rounded text-xs transition-all ${
          rating === 1
            ? 'bg-green-500/20 text-green-400'
            : 'text-gray-500 hover:text-green-400 hover:bg-green-500/10'
        } ${loading ? 'opacity-50 cursor-not-allowed' : ''}`}
        title="有帮助"
      >
        <ThumbsUp className="w-3.5 h-3.5" />
        <span>有帮助</span>
      </button>
      
      <button
        onClick={() => handleRate(-1)}
        disabled={loading}
        className={`flex items-center gap-1 px-2 py-1 rounded text-xs transition-all ${
          rating === -1
            ? 'bg-red-500/20 text-red-400'
            : 'text-gray-500 hover:text-red-400 hover:bg-red-500/10'
        } ${loading ? 'opacity-50 cursor-not-allowed' : ''}`}
        title="无帮助"
      >
        <ThumbsDown className="w-3.5 h-3.5" />
        <span>无帮助</span>
      </button>
    </div>
  )
}














