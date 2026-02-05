import AgentProposalModal from './AgentProposalModal'

interface AgentModalProps {
  open: boolean
  onClose: () => void
  onSendToQA?: (text: string) => void
}

export default function AgentModal({ open, onClose, onSendToQA }: AgentModalProps) {
  return <AgentProposalModal open={open} onClose={onClose} onSendToQA={onSendToQA} />
}

