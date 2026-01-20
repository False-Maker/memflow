import AgentProposalModal from './AgentProposalModal'

interface AgentModalProps {
  open: boolean
  onClose: () => void
}

export default function AgentModal({ open, onClose }: AgentModalProps) {
  return <AgentProposalModal open={open} onClose={onClose} />
}

