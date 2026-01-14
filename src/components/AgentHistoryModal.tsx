import { X } from 'lucide-react'

interface AgentHistoryModalProps {
  open: boolean
  onClose: () => void
}

export default function AgentHistoryModal({ open, onClose }: AgentHistoryModalProps) {
  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="glass w-full max-w-2xl rounded-lg p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-white">代理历史</h2>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-surface transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
        <p className="text-gray-400">功能开发中...</p>
      </div>
    </div>
  )
}

