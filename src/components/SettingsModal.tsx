import { useState, useEffect, useReducer, useCallback } from 'react'
import { X, Check, AlertCircle, Loader2, ChevronDown, Shield, Settings, Bot, Plus, Trash2, Eye, FolderOpen } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { open as openFileDialog } from '@tauri-apps/plugin-dialog'
import { useApp } from '../contexts/AppContext'

// 调试辅助：检查 dialog 插件是否可用
const checkDialogPlugin = async () => {
  try {
    console.log('[调试] 检查 dialog 插件...')
    console.log('[调试] openFileDialog 函数类型:', typeof openFileDialog)
    console.log('[调试] openFileDialog 函数:', openFileDialog)
    return true
  } catch (e) {
    console.error('[调试] dialog 插件检查失败:', e)
    return false
  }
}

// ==================== Type Definitions ====================

type ChatModelProvider = 'openai' | 'anthropic' | 'custom'
type EmbeddingModelProvider = 'openai' | 'custom'

interface ChatModelConfig {
  provider: ChatModelProvider
  modelId: string
  apiKey: string
  baseUrl?: string
  modelName?: string
}

interface EmbeddingModelConfig {
  provider: EmbeddingModelProvider
  modelId: string
  apiKey?: string
  baseUrl?: string
  useSharedKey: boolean
}

interface ModelFormState {
  chat: ChatModelConfig
  embedding: EmbeddingModelConfig
}

interface ApiKeyStatus {
  openai: { saved: boolean; loading: boolean; message: string }
  anthropic: { saved: boolean; loading: boolean; message: string }
  custom: { saved: boolean; loading: boolean; message: string }
  embedding: { saved: boolean; loading: boolean; message: string }
  embeddingCustom: { saved: boolean; loading: boolean; message: string }
}

interface ConnectionTestState {
  chat: { testing: boolean; result: 'idle' | 'success' | 'error'; message: string }
  embedding: { testing: boolean; result: 'idle' | 'success' | 'error'; message: string }
}

// ==================== Constants ====================

const OPENAI_MODELS = [
  { id: 'gpt-4o', name: 'GPT-4o', description: '最强大的多模态模型' },
  { id: 'gpt-4o-mini', name: 'GPT-4o Mini', description: '性价比高，推荐' },
  { id: 'gpt-4-turbo', name: 'GPT-4 Turbo', description: '高性能版本' },
] as const

const ANTHROPIC_MODELS = [
  { id: 'claude-3-5-sonnet-20241022', name: 'Claude 3.5 Sonnet', description: '最新 Sonnet 模型' },
  { id: 'claude-3-opus-20240229', name: 'Claude 3 Opus', description: '最强推理能力' },
  { id: 'claude-3-sonnet-20240229', name: 'Claude 3 Sonnet', description: '平衡性能与速度' },
] as const

const EMBEDDING_MODELS = [
  { id: 'text-embedding-3-small', name: 'Embedding 3 Small', description: '性价比高，推荐' },
  { id: 'text-embedding-3-large', name: 'Embedding 3 Large', description: '更高精度' },
  { id: 'text-embedding-ada-002', name: 'Ada 002', description: '经典模型' },
] as const

// ==================== Form Reducer ====================

type FormAction =
  | { type: 'SET_CHAT_PROVIDER'; payload: ChatModelProvider }
  | { type: 'SET_CHAT_MODEL_ID'; payload: string }
  | { type: 'SET_CHAT_API_KEY'; payload: string }
  | { type: 'SET_CHAT_BASE_URL'; payload: string }
  | { type: 'SET_CHAT_MODEL_NAME'; payload: string }
  | { type: 'SET_EMBEDDING_PROVIDER'; payload: EmbeddingModelProvider }
  | { type: 'SET_EMBEDDING_MODEL_ID'; payload: string }
  | { type: 'SET_EMBEDDING_API_KEY'; payload: string }
  | { type: 'SET_EMBEDDING_BASE_URL'; payload: string }
  | { type: 'SET_EMBEDDING_USE_SHARED_KEY'; payload: boolean }
  | { type: 'RESET_FORM'; payload: ModelFormState }

function getProviderFromModelId(modelId: string): ChatModelProvider {
  if (modelId.startsWith('gpt-') || modelId.startsWith('text-embedding-')) return 'openai'
  if (modelId.startsWith('claude-')) return 'anthropic'
  return 'custom'
}

function getEmbeddingProviderFromModelId(modelId: string): EmbeddingModelProvider {
  if (!modelId) return 'openai'
  // 当前后端仅实现 OpenAI Embeddings，因此列表外的模型名一律视为自定义（用于 UI 回填/保存）
  if (modelId.startsWith('text-embedding-')) return 'openai'
  if (modelId === 'text-embedding-ada-002') return 'openai'
  return 'custom'
}

function formReducer(state: ModelFormState, action: FormAction): ModelFormState {
  switch (action.type) {
    case 'SET_CHAT_PROVIDER': {
      const provider = action.payload
      let defaultModelId = ''
      if (provider === 'openai') defaultModelId = 'gpt-4o-mini'
      else if (provider === 'anthropic') defaultModelId = 'claude-3-5-sonnet-20241022'
      return {
        ...state,
        chat: {
          ...state.chat,
          provider,
          modelId: provider === 'custom' ? '' : defaultModelId,
          modelName: provider === 'custom' ? state.chat.modelName : undefined,
          baseUrl: provider === 'custom' ? state.chat.baseUrl : undefined,
        },
      }
    }
    case 'SET_CHAT_MODEL_ID':
      return { ...state, chat: { ...state.chat, modelId: action.payload } }
    case 'SET_CHAT_API_KEY':
      return { ...state, chat: { ...state.chat, apiKey: action.payload } }
    case 'SET_CHAT_BASE_URL':
      return { ...state, chat: { ...state.chat, baseUrl: action.payload || undefined } }
    case 'SET_CHAT_MODEL_NAME':
      return { ...state, chat: { ...state.chat, modelName: action.payload } }
    case 'SET_EMBEDDING_PROVIDER': {
      const provider = action.payload
      return {
        ...state,
        embedding: {
          ...state.embedding,
          provider,
          modelId: provider === 'openai' ? 'text-embedding-3-small' : '',
          baseUrl: provider === 'custom' ? state.embedding.baseUrl : undefined,
        },
      }
    }
    case 'SET_EMBEDDING_MODEL_ID':
      return { ...state, embedding: { ...state.embedding, modelId: action.payload } }
    case 'SET_EMBEDDING_API_KEY':
      return { ...state, embedding: { ...state.embedding, apiKey: action.payload } }
    case 'SET_EMBEDDING_BASE_URL':
      return { ...state, embedding: { ...state.embedding, baseUrl: action.payload || undefined } }
    case 'SET_EMBEDDING_USE_SHARED_KEY':
      return { ...state, embedding: { ...state.embedding, useSharedKey: action.payload } }
    case 'RESET_FORM':
      return action.payload
    default:
      return state
  }
}

// ==================== Helper Components ====================

interface SelectGroupProps {
  value: string
  onChange: (value: string) => void
  groups: {
    label: string
    options: ReadonlyArray<{ id: string; name: string; description: string }>
  }[]
  customOption?: { label: string; value: string }
  className?: string
}

function GroupedSelect({ value, onChange, groups, customOption, className }: SelectGroupProps) {
  return (
    <div className="relative">
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className={`w-full appearance-none px-4 py-2.5 pr-10 bg-surface border border-glass-border rounded-lg text-white cursor-pointer hover:border-neon-blue/50 transition-colors focus:outline-none focus:ring-2 focus:ring-neon-blue/30 ${className}`}
      >
        {groups.map((group) => (
          <optgroup key={group.label} label={group.label} className="bg-surface">
            {group.options.map((opt) => (
              <option key={opt.id} value={opt.id} className="bg-surface py-2">
                {opt.name} — {opt.description}
              </option>
            ))}
          </optgroup>
        ))}
        {customOption && (
          <optgroup label="自定义" className="bg-surface">
            <option value={customOption.value} className="bg-surface">
              {customOption.label}
            </option>
          </optgroup>
        )}
      </select>
      <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 pointer-events-none" />
    </div>
  )
}

interface InputFieldProps {
  label: string
  value: string
  onChange: (value: string) => void
  type?: 'text' | 'password'
  placeholder?: string
  hint?: string
  status?: 'idle' | 'saved' | 'error'
  statusMessage?: string
  rightElement?: React.ReactNode
}

function InputField({
  label,
  value,
  onChange,
  type = 'text',
  placeholder,
  hint,
  status,
  statusMessage,
  rightElement,
}: InputFieldProps) {
  return (
    <div className="space-y-1.5">
      <label className="block text-sm font-medium text-gray-300">{label}</label>
      <div className="flex gap-2">
        <input
          type={type}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          className="flex-1 px-4 py-2.5 bg-surface border border-glass-border rounded-lg text-white placeholder:text-gray-500 hover:border-neon-blue/50 transition-colors focus:outline-none focus:ring-2 focus:ring-neon-blue/30"
        />
        {rightElement}
      </div>
      {statusMessage && (
        <p
          className={`text-xs flex items-center gap-1 ${
            status === 'saved'
              ? 'text-emerald-400'
              : status === 'error'
              ? 'text-red-400'
              : 'text-gray-400'
          }`}
        >
          {status === 'saved' && <Check className="w-3 h-3" />}
          {status === 'error' && <AlertCircle className="w-3 h-3" />}
          {statusMessage}
        </p>
      )}
      {hint && !statusMessage && <p className="text-xs text-gray-500">{hint}</p>}
    </div>
  )
}

// ==================== Main Component ====================

interface SettingsModalProps {
  open: boolean
  onClose: () => void
}

export default function SettingsModal({ open, onClose }: SettingsModalProps) {
  const { state, dispatch } = useApp()
  const [draftConfig, setDraftConfig] = useState(state.config)
  const [activeTab, setActiveTab] = useState<'general' | 'privacy'>('general')

  // Blocklist state
  const [blocklist, setBlocklist] = useState<string[]>([])
  const [newBlockItem, setNewBlockItem] = useState('')
  const [blocklistLoading, setBlocklistLoading] = useState(false)
  const [blocklistError, setBlocklistError] = useState<string | null>(null)

  // Form state with reducer
  const initialFormState: ModelFormState = {
    chat: {
      provider: getProviderFromModelId(state.config.chatModel || 'gpt-4o-mini'),
      modelId: state.config.chatModel || 'gpt-4o-mini',
      apiKey: '',
      baseUrl: state.config.openaiBaseUrl,
      modelName: undefined,
    },
    embedding: {
      provider: getEmbeddingProviderFromModelId(state.config.embeddingModel || ''),
      modelId: state.config.embeddingModel || 'text-embedding-3-small',
      apiKey: '',
      baseUrl:
        getEmbeddingProviderFromModelId(state.config.embeddingModel || '') === 'custom'
          ? state.config.embeddingBaseUrl
          : undefined,
      useSharedKey: state.config.embeddingUseSharedKey ?? true,
    },
  }

  const [formState, formDispatch] = useReducer(formReducer, initialFormState)

  // API Key status tracking
  const [apiKeyStatus, setApiKeyStatus] = useState<ApiKeyStatus>({
    openai: { saved: false, loading: false, message: '' },
    anthropic: { saved: false, loading: false, message: '' },
    custom: { saved: false, loading: false, message: '' },
    embedding: { saved: false, loading: false, message: '' },
    embeddingCustom: { saved: false, loading: false, message: '' },
  })

  // Connection test state
  const [testState, setTestState] = useState<ConnectionTestState>({
    chat: { testing: false, result: 'idle', message: '' },
    embedding: { testing: false, result: 'idle', message: '' },
  })

  // Check existing API keys on open
  useEffect(() => {
    if (open) {
      checkApiKeys()
      setDraftConfig(state.config)
      loadBlocklist()
      
      // 检查 dialog 插件可用性
      checkDialogPlugin().then((available) => {
        if (!available) {
          console.warn('[黑名单] dialog 插件可能不可用')
          setBlocklistError('文件选择功能可能不可用，请检查应用权限')
        }
      })
      
      // Reset form to current config
      formDispatch({
        type: 'RESET_FORM',
        payload: {
          chat: {
            provider: getProviderFromModelId(state.config.chatModel || 'gpt-4o-mini'),
            modelId: state.config.chatModel || 'gpt-4o-mini',
            apiKey: '',
            baseUrl: state.config.openaiBaseUrl,
            modelName:
              getProviderFromModelId(state.config.chatModel || '') === 'custom'
                ? state.config.chatModel
                : undefined,
          },
          embedding: {
            provider: getEmbeddingProviderFromModelId(state.config.embeddingModel || ''),
            modelId: state.config.embeddingModel || 'text-embedding-3-small',
            apiKey: '',
            baseUrl:
              getEmbeddingProviderFromModelId(state.config.embeddingModel || '') === 'custom'
                ? state.config.embeddingBaseUrl
                : undefined,
            useSharedKey: state.config.embeddingUseSharedKey ?? true,
          },
        },
      })
    }
  }, [open, state.config])

  const loadBlocklist = async () => {
    try {
      setBlocklistLoading(true)
      setBlocklistError(null)
      const list = await invoke<string[]>('get_blocklist')
      setBlocklist(list)
    } catch (e) {
      console.error('加载黑名单失败:', e)
      setBlocklistError(String(e))
    } finally {
      setBlocklistLoading(false)
    }
  }

  const checkApiKeys = async () => {
    // Check OpenAI
    try {
      const key = await invoke<string | null>('get_api_key', { service: 'openai' })
      setApiKeyStatus((prev) => ({
        ...prev,
        openai: { saved: !!key, loading: false, message: key ? 'API Key 已配置' : '' },
      }))
      if (key) {
        formDispatch({ type: 'SET_CHAT_API_KEY', payload: '••••••••••••••••' })
      }
    } catch (e) {
      console.error('检查 OpenAI API Key 失败:', e)
    }

    // Check Embedding (separate key)
    try {
      const key = await invoke<string | null>('get_api_key', { service: 'embedding' })
      setApiKeyStatus((prev) => ({
        ...prev,
        embedding: { saved: !!key, loading: false, message: key ? 'API Key 已配置' : '' },
      }))
      if (key) {
        formDispatch({ type: 'SET_EMBEDDING_API_KEY', payload: '••••••••••••••••' })
      }
    } catch (e) {
      console.error('检查 Embedding API Key 失败:', e)
    }

    // Check Anthropic
    try {
      const key = await invoke<string | null>('get_api_key', { service: 'anthropic' })
      setApiKeyStatus((prev) => ({
        ...prev,
        anthropic: { saved: !!key, loading: false, message: key ? 'API Key 已配置' : '' },
      }))
    } catch (e) {
      console.error('检查 Anthropic API Key 失败:', e)
    }
  }

  // Handle provider change from grouped select
  const handleChatModelChange = useCallback((modelId: string) => {
    if (modelId === 'custom') {
      formDispatch({ type: 'SET_CHAT_PROVIDER', payload: 'custom' })
    } else {
      const provider = getProviderFromModelId(modelId)
      formDispatch({ type: 'SET_CHAT_PROVIDER', payload: provider })
      formDispatch({ type: 'SET_CHAT_MODEL_ID', payload: modelId })
    }
  }, [])

  // Save API Key
  const handleSaveApiKey = async (
    service: 'openai' | 'anthropic' | 'custom' | 'embedding',
    key: string
  ) => {
    if (!key || key === '••••••••••••••••') return

    setApiKeyStatus((prev) => ({
      ...prev,
      [service]: { ...prev[service], loading: true },
    }))

    try {
      const backendService = service === 'custom' ? 'openai' : service
      await invoke('save_api_key', { service: backendService, key })
      setApiKeyStatus((prev) => ({
        ...prev,
        [service]: { saved: true, loading: false, message: 'API Key 保存成功！' },
      }))
      // Mask the key after save
      if (service === 'openai' || service === 'custom') {
        formDispatch({ type: 'SET_CHAT_API_KEY', payload: '••••••••••••••••' })
      }
      if (service === 'embedding') {
        formDispatch({ type: 'SET_EMBEDDING_API_KEY', payload: '••••••••••••••••' })
      }
      setTimeout(() => {
        setApiKeyStatus((prev) => ({
          ...prev,
          [service]: { ...prev[service], message: 'API Key 已配置' },
        }))
      }, 2000)
    } catch (e) {
      setApiKeyStatus((prev) => ({
        ...prev,
        [service]: { saved: false, loading: false, message: `保存失败: ${e}` },
      }))
    }
  }

  // Delete API Key
  const handleDeleteApiKey = async (service: 'openai' | 'anthropic' | 'embedding') => {
    try {
      await invoke('delete_api_key', { service })
      setApiKeyStatus((prev) => ({
        ...prev,
        [service]: { saved: false, loading: false, message: 'API Key 已删除' },
      }))
      if (service === 'openai') formDispatch({ type: 'SET_CHAT_API_KEY', payload: '' })
      if (service === 'embedding') formDispatch({ type: 'SET_EMBEDDING_API_KEY', payload: '' })
      setTimeout(() => {
        setApiKeyStatus((prev) => ({
          ...prev,
          [service]: { ...prev[service], message: '' },
        }))
      }, 2000)
    } catch (e) {
      setApiKeyStatus((prev) => ({
        ...prev,
        [service]: { ...prev[service], message: `删除失败: ${e}` },
      }))
    }
  }

  // Test connection (mock implementation)
  const handleTestConnection = async (type: 'chat' | 'embedding') => {
    setTestState((prev) => ({
      ...prev,
      [type]: { testing: true, result: 'idle', message: '正在测试连接...' },
    }))
    try {
      if (type === 'chat') {
        const provider = formState.chat.provider
        const model =
          provider === 'custom' && formState.chat.modelName ? formState.chat.modelName : formState.chat.modelId

        const apiKey =
          formState.chat.apiKey && formState.chat.apiKey !== '••••••••••••••••'
            ? formState.chat.apiKey
            : undefined

        const baseUrl =
          provider === 'custom'
            ? formState.chat.baseUrl
            : provider === 'anthropic'
            ? state.config.anthropicBaseUrl
            : state.config.openaiBaseUrl

        await invoke('test_chat_connection', {
          params: {
            provider,
            model,
            apiKey,
            baseUrl,
          },
        })

        setTestState((prev) => ({
          ...prev,
          [type]: { testing: false, result: 'success', message: '连接成功！模型响应正常' },
        }))
      } else {
        const provider = formState.embedding.provider
        const model =
          provider === 'custom'
            ? formState.embedding.modelId
            : formState.embedding.modelId || 'text-embedding-3-small'

        const useSharedKey =
          provider === 'openai' ? !!formState.embedding.useSharedKey : false

        const apiKey =
          !useSharedKey &&
          formState.embedding.apiKey &&
          formState.embedding.apiKey !== '••••••••••••••••'
            ? formState.embedding.apiKey
            : undefined

        const baseUrl =
          provider === 'custom' ? formState.embedding.baseUrl : state.config.openaiBaseUrl

        await invoke('test_embedding_connection', {
          params: {
            provider,
            model,
            apiKey,
            baseUrl,
            useSharedKey,
          },
        })

        setTestState((prev) => ({
          ...prev,
          [type]: { testing: false, result: 'success', message: '连接成功！Embedding 响应正常' },
        }))
      }
    } catch (e) {
      const msg = typeof e === 'string' ? e : JSON.stringify(e)
      setTestState((prev) => ({
        ...prev,
        [type]: { testing: false, result: 'error', message: `连接失败：${msg}` },
      }))
    }

    // Clear message after delay
    setTimeout(() => {
      setTestState((prev) => ({
        ...prev,
        [type]: { ...prev[type], result: 'idle', message: '' },
      }))
    }, 3000)
  }

  const handleAddBlockItem = async () => {
    if (!newBlockItem.trim()) return
    try {
      setBlocklistError(null)
      await invoke('add_blocklist_item', { app_name: newBlockItem.trim() })
      setNewBlockItem('')
      await loadBlocklist()
    } catch (e) {
      console.error('添加黑名单失败:', e)
      setBlocklistError(String(e))
    }
  }

  const handleSelectFile = async () => {
    console.log('[黑名单] handleSelectFile 被调用')
    try {
      setBlocklistError(null)
      console.log('[黑名单] 正在打开文件选择对话框...')
      
      const selected = await openFileDialog({
        multiple: false,
        directory: false,
        filters: [{
          name: 'Applications',
          extensions: ['exe', 'lnk', 'app']
        }]
      })
      
      console.log('[黑名单] 文件选择结果:', selected)

      if (selected && typeof selected === 'string') {
        const fileName = selected.split(/[/\\]/).pop()
        console.log('[黑名单] 提取的文件名:', fileName)
        
        if (fileName) {
          try {
            setBlocklistError(null)
            console.log('[黑名单] 正在添加应用到黑名单:', fileName)
            await invoke('add_blocklist_item', { app_name: fileName })
            setNewBlockItem('')
            await loadBlocklist()
            console.log('[黑名单] 成功添加到黑名单')
          } catch (e) {
            console.error('[黑名单] 添加黑名单失败:', e)
            setBlocklistError(`添加失败: ${String(e)}`)
          }
        } else {
          console.warn('[黑名单] 无法从路径提取文件名:', selected)
          setBlocklistError('无法从文件路径提取文件名')
        }
      } else if (selected === null) {
        console.log('[黑名单] 用户取消了文件选择')
        // 用户取消选择，不显示错误
      } else {
        console.warn('[黑名单] 意外的选择结果类型:', typeof selected, selected)
        setBlocklistError('文件选择返回了意外的结果')
      }
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      console.error('[黑名单] 选择文件失败:', e)
      setBlocklistError(`选择文件失败: ${errorMsg}`)
      
      // 检查是否是权限问题
      if (errorMsg.includes('permission') || errorMsg.includes('权限') || errorMsg.includes('denied')) {
        setBlocklistError('文件选择权限被拒绝，请检查应用权限设置')
      }
    }
  }

  const handleRemoveBlockItem = async (item: string) => {
    try {
      setBlocklistError(null)
      await invoke('remove_blocklist_item', { app_name: item })
      await loadBlocklist()
    } catch (e) {
      console.error('移除黑名单失败:', e)
      setBlocklistError(String(e))
    }
  }

  // Save all settings
  const handleSave = async () => {
    try {
      // Determine the actual model ID to save
      let chatModel = formState.chat.modelId
      if (formState.chat.provider === 'custom' && formState.chat.modelName) {
        chatModel = formState.chat.modelName
      }

      const updatedConfig = {
        ...draftConfig,
        chatModel,
        embeddingModel:
          formState.embedding.provider === 'custom'
            ? formState.embedding.modelId
            : formState.embedding.modelId || 'text-embedding-3-small',
        embeddingBaseUrl:
          formState.embedding.provider === 'custom' ? formState.embedding.baseUrl : undefined,
        embeddingUseSharedKey: formState.embedding.useSharedKey,
        openaiBaseUrl:
          formState.chat.provider === 'custom' ? formState.chat.baseUrl : draftConfig.openaiBaseUrl,
      }

      await invoke('update_config', { config: updatedConfig })
      dispatch({ type: 'SET_CONFIG', payload: updatedConfig })
      onClose()
    } catch (e) {
      console.error('保存配置失败:', e)
      alert('保存配置失败: ' + e)
    }
  }

  if (!open) return null

  // Determine which fields to show based on chat provider
  const showOpenAIFields = formState.chat.provider === 'openai'
  const showAnthropicFields = formState.chat.provider === 'anthropic'
  const showCustomFields = formState.chat.provider === 'custom'

  // Can share key only if chat provider is OpenAI and embedding provider is OpenAI
  const canShareKey =
    formState.chat.provider === 'openai' && formState.embedding.provider === 'openai'

  const showEmbeddingKeyField = formState.embedding.provider !== 'openai' || !canShareKey || !formState.embedding.useSharedKey

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="glass w-full max-w-3xl h-[80vh] flex flex-col rounded-2xl border border-glass-border/50 shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-glass-border/50 bg-surface/80 backdrop-blur-md">
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <Settings className="w-5 h-5 text-neon-blue" />
            设置
          </h2>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-white/10 transition-colors"
          >
            <X className="w-5 h-5 text-gray-400" />
          </button>
        </div>

        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar */}
          <div className="w-48 border-r border-glass-border/50 bg-surface/30 p-4 space-y-2">
            <button
              onClick={() => setActiveTab('general')}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-colors ${
                activeTab === 'general'
                  ? 'bg-neon-blue/20 text-neon-blue'
                  : 'text-gray-400 hover:bg-white/5 hover:text-white'
              }`}
            >
              <Bot className="w-4 h-4" />
              模型设置
            </button>
            <button
              onClick={() => setActiveTab('privacy')}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-colors ${
                activeTab === 'privacy'
                  ? 'bg-neon-blue/20 text-neon-blue'
                  : 'text-gray-400 hover:bg-white/5 hover:text-white'
              }`}
            >
              <Shield className="w-4 h-4" />
              隐私与屏蔽
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-6">
            {activeTab === 'general' ? (
              <div className="space-y-8">
                {/* ==================== Chat Model Section ==================== */}
                <section className="space-y-4">
                  <div className="flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-neon-blue to-neon-purple flex items-center justify-center">
                      <span className="text-sm">💬</span>
                    </div>
                    <h3 className="text-lg font-semibold text-white">对话模型</h3>
                  </div>

                  {/* Grouped Model Select */}
                  <div className="space-y-2">
                    <label className="block text-sm font-medium text-gray-300">选择模型</label>
                    <GroupedSelect
                      value={
                        formState.chat.provider === 'custom' ? 'custom' : formState.chat.modelId
                      }
                      onChange={handleChatModelChange}
                      groups={[
                        { label: 'OpenAI', options: [...OPENAI_MODELS] },
                        { label: 'Anthropic', options: [...ANTHROPIC_MODELS] },
                      ]}
                      customOption={{ label: '自定义模型（OpenAI 兼容）', value: 'custom' }}
                    />
                  </div>

                  {/* Dynamic Fields based on Provider */}
                  <div className="space-y-4 pt-2">
                    {/* OpenAI Fields */}
                    {showOpenAIFields && (
                      <div className="p-4 rounded-xl bg-surface/50 border border-glass-border/30 space-y-4">
                        <div className="flex items-center gap-2 text-sm text-emerald-400">
                          <div className="w-2 h-2 rounded-full bg-emerald-400"></div>
                          <span>OpenAI API</span>
                        </div>
                        <InputField
                          label="OpenAI API Key"
                          value={formState.chat.apiKey}
                          onChange={(v) => {
                            formDispatch({ type: 'SET_CHAT_API_KEY', payload: v })
                            setApiKeyStatus((prev) => ({
                              ...prev,
                              openai: { ...prev.openai, message: '' },
                            }))
                          }}
                          type="password"
                          placeholder="sk-..."
                          hint="获取 Key: platform.openai.com/api-keys"
                          status={apiKeyStatus.openai.saved ? 'saved' : 'idle'}
                          statusMessage={apiKeyStatus.openai.message}
                          rightElement={
                            <div className="flex gap-2">
                              <button
                                onClick={() => handleSaveApiKey('openai', formState.chat.apiKey)}
                                disabled={
                                  !formState.chat.apiKey ||
                                  formState.chat.apiKey === '••••••••••••••••' ||
                                  apiKeyStatus.openai.loading
                                }
                                className="px-4 py-2 rounded-lg bg-emerald-500 text-white hover:bg-emerald-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                              >
                                {apiKeyStatus.openai.loading && (
                                  <Loader2 className="w-4 h-4 animate-spin" />
                                )}
                                保存
                              </button>
                              {apiKeyStatus.openai.saved && (
                                <button
                                  onClick={() => handleDeleteApiKey('openai')}
                                  className="px-4 py-2 rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors"
                                >
                                  删除
                                </button>
                              )}
                            </div>
                          }
                        />
                      </div>
                    )}

                    {/* Anthropic Fields */}
                    {showAnthropicFields && (
                      <div className="p-4 rounded-xl bg-surface/50 border border-glass-border/30 space-y-4">
                        <div className="flex items-center gap-2 text-sm text-amber-400">
                          <div className="w-2 h-2 rounded-full bg-amber-400"></div>
                          <span>Anthropic API</span>
                        </div>
                        <InputField
                          label="Anthropic API Key"
                          value={formState.chat.apiKey}
                          onChange={(v) => {
                            formDispatch({ type: 'SET_CHAT_API_KEY', payload: v })
                            setApiKeyStatus((prev) => ({
                              ...prev,
                              anthropic: { ...prev.anthropic, message: '' },
                            }))
                          }}
                          type="password"
                          placeholder="sk-ant-..."
                          hint="获取 Key: console.anthropic.com/settings/keys"
                          status={apiKeyStatus.anthropic.saved ? 'saved' : 'idle'}
                          statusMessage={apiKeyStatus.anthropic.message}
                          rightElement={
                            <div className="flex gap-2">
                              <button
                                onClick={() => handleSaveApiKey('anthropic', formState.chat.apiKey)}
                                disabled={
                                  !formState.chat.apiKey ||
                                  formState.chat.apiKey === '••••••••••••••••' ||
                                  apiKeyStatus.anthropic.loading
                                }
                                className="px-4 py-2 rounded-lg bg-amber-500 text-white hover:bg-amber-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                              >
                                {apiKeyStatus.anthropic.loading && (
                                  <Loader2 className="w-4 h-4 animate-spin" />
                                )}
                                保存
                              </button>
                              {apiKeyStatus.anthropic.saved && (
                                <button
                                  onClick={() => handleDeleteApiKey('anthropic')}
                                  className="px-4 py-2 rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors"
                                >
                                  删除
                                </button>
                              )}
                            </div>
                          }
                        />
                      </div>
                    )}

                    {/* Custom Model Fields */}
                    {showCustomFields && (
                      <div className="p-4 rounded-xl bg-surface/50 border border-glass-border/30 space-y-4">
                        <div className="flex items-center gap-2 text-sm text-violet-400">
                          <div className="w-2 h-2 rounded-full bg-violet-400"></div>
                          <span>自定义模型（OpenAI 兼容）</span>
                        </div>
                        <InputField
                          label="模型名称"
                          value={formState.chat.modelName || ''}
                          onChange={(v) => formDispatch({ type: 'SET_CHAT_MODEL_NAME', payload: v })}
                          placeholder="例如: deepseek-chat, llama-3-70b"
                        />
                        <InputField
                          label="Base URL"
                          value={formState.chat.baseUrl || ''}
                          onChange={(v) => formDispatch({ type: 'SET_CHAT_BASE_URL', payload: v })}
                          placeholder="https://api.openai.com/v1"
                          hint="OpenAI 兼容端点（可填基础 URL 或完整地址）"
                        />
                        <InputField
                          label="API Key"
                          value={formState.chat.apiKey}
                          onChange={(v) => formDispatch({ type: 'SET_CHAT_API_KEY', payload: v })}
                          type="password"
                          placeholder="your-api-key"
                          rightElement={
                            <button
                              onClick={() => handleSaveApiKey('custom', formState.chat.apiKey)}
                              disabled={
                                !formState.chat.apiKey ||
                                formState.chat.apiKey === '••••••••••••••••' ||
                                apiKeyStatus.custom.loading
                              }
                              className="px-4 py-2 rounded-lg bg-violet-500 text-white hover:bg-violet-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                            >
                              {apiKeyStatus.custom.loading && (
                                <Loader2 className="w-4 h-4 animate-spin" />
                              )}
                              保存
                            </button>
                          }
                        />
                      </div>
                    )}

                    {/* Test Connection Button */}
                    <div className="flex items-center gap-4">
                      <button
                        onClick={() => handleTestConnection('chat')}
                        disabled={testState.chat.testing}
                        className="px-4 py-2 rounded-lg border border-glass-border text-gray-300 hover:bg-white/5 hover:text-white transition-colors disabled:opacity-50 flex items-center gap-2"
                      >
                        {testState.chat.testing ? (
                          <Loader2 className="w-4 h-4 animate-spin" />
                        ) : (
                          <Check className="w-4 h-4" />
                        )}
                        测试连接
                      </button>
                      {testState.chat.message && (
                        <span
                          className={`text-sm flex items-center gap-1 ${
                            testState.chat.result === 'success'
                              ? 'text-emerald-400'
                              : testState.chat.result === 'error'
                              ? 'text-red-400'
                              : 'text-gray-400'
                          }`}
                        >
                          {testState.chat.result === 'success' && <Check className="w-4 h-4" />}
                          {testState.chat.result === 'error' && <AlertCircle className="w-4 h-4" />}
                          {testState.chat.message}
                        </span>
                      )}
                    </div>
                  </div>
                </section>

                <div className="h-px bg-glass-border/50" />

                {/* ==================== Embedding Model Section ==================== */}
                <section className="space-y-4">
                  <div className="flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-neon-purple to-pink-500 flex items-center justify-center">
                      <span className="text-sm">🔍</span>
                    </div>
                    <h3 className="text-lg font-semibold text-white">Embedding 模型</h3>
                  </div>

                  <div className="space-y-2">
                    <label className="block text-sm font-medium text-gray-300">选择模型</label>
                    <GroupedSelect
                      value={formState.embedding.provider === 'custom' ? 'custom' : formState.embedding.modelId}
                      onChange={(val) => {
                        if (val === 'custom') {
                          formDispatch({ type: 'SET_EMBEDDING_PROVIDER', payload: 'custom' })
                        } else {
                          const provider = getEmbeddingProviderFromModelId(val)
                          formDispatch({ type: 'SET_EMBEDDING_PROVIDER', payload: provider })
                          formDispatch({ type: 'SET_EMBEDDING_MODEL_ID', payload: val })
                        }
                      }}
                      groups={[{ label: 'OpenAI', options: [...EMBEDDING_MODELS] }]}
                      customOption={{ label: '自定义模型', value: 'custom' }}
                    />
                  </div>

                  {/* OpenAI Shared Key Option */}
                  {canShareKey && (
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() =>
                          formDispatch({
                            type: 'SET_EMBEDDING_USE_SHARED_KEY',
                            payload: !formState.embedding.useSharedKey,
                          })
                        }
                        className={`w-10 h-6 rounded-full transition-colors relative ${
                          formState.embedding.useSharedKey ? 'bg-neon-blue' : 'bg-gray-600'
                        }`}
                      >
                        <div
                          className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
                            formState.embedding.useSharedKey ? 'translate-x-4' : 'translate-x-0'
                          }`}
                        />
                      </button>
                      <span className="text-sm text-gray-300">
                        使用对话模型的 API Key
                        {formState.embedding.useSharedKey && <span className="text-gray-500 ml-2">(已启用)</span>}
                      </span>
                    </div>
                  )}

                  {/* Embedding Custom Fields / Key */}
                  <div className="space-y-4 pt-2">
                    {formState.embedding.provider === 'custom' && (
                      <>
                        <InputField
                          label="模型 ID"
                          value={formState.embedding.modelId}
                          onChange={(v) => formDispatch({ type: 'SET_EMBEDDING_MODEL_ID', payload: v })}
                          placeholder="例如: text-embedding-ada-002"
                        />
                        <InputField
                          label="Base URL"
                          value={formState.embedding.baseUrl || ''}
                          onChange={(v) => formDispatch({ type: 'SET_EMBEDDING_BASE_URL', payload: v })}
                          placeholder="https://api.openai.com/v1"
                        />
                      </>
                    )}

                    {showEmbeddingKeyField && (
                      <InputField
                        label="Embedding API Key"
                        value={formState.embedding.apiKey || ''}
                        onChange={(v) => {
                          formDispatch({ type: 'SET_EMBEDDING_API_KEY', payload: v })
                          setApiKeyStatus((prev) => ({
                            ...prev,
                            embedding: { ...prev.embedding, message: '' },
                          }))
                        }}
                        type="password"
                        placeholder="sk-..."
                        status={apiKeyStatus.embedding.saved ? 'saved' : 'idle'}
                        statusMessage={apiKeyStatus.embedding.message}
                        rightElement={
                          <div className="flex gap-2">
                            <button
                              onClick={() => handleSaveApiKey('embedding', formState.embedding.apiKey || '')}
                              disabled={
                                !formState.embedding.apiKey ||
                                formState.embedding.apiKey === '••••••••••••••••' ||
                                apiKeyStatus.embedding.loading
                              }
                              className="px-4 py-2 rounded-lg bg-neon-blue text-black font-medium hover:bg-neon-blue/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                            >
                              {apiKeyStatus.embedding.loading && (
                                <Loader2 className="w-4 h-4 animate-spin" />
                              )}
                              保存
                            </button>
                            {apiKeyStatus.embedding.saved && (
                              <button
                                onClick={() => handleDeleteApiKey('embedding')}
                                className="px-4 py-2 rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors"
                              >
                                删除
                              </button>
                            )}
                          </div>
                        }
                      />
                    )}

                    {/* Test Embedding Connection */}
                    <div className="flex items-center gap-4">
                      <button
                        onClick={() => handleTestConnection('embedding')}
                        disabled={testState.embedding.testing}
                        className="px-4 py-2 rounded-lg border border-glass-border text-gray-300 hover:bg-white/5 hover:text-white transition-colors disabled:opacity-50 flex items-center gap-2"
                      >
                        {testState.embedding.testing ? (
                          <Loader2 className="w-4 h-4 animate-spin" />
                        ) : (
                          <Check className="w-4 h-4" />
                        )}
                        测试 Embedding
                      </button>
                      {testState.embedding.message && (
                        <span
                          className={`text-sm flex items-center gap-1 ${
                            testState.embedding.result === 'success'
                              ? 'text-emerald-400'
                              : testState.embedding.result === 'error'
                              ? 'text-red-400'
                              : 'text-gray-400'
                          }`}
                        >
                          {testState.embedding.result === 'success' && <Check className="w-4 h-4" />}
                          {testState.embedding.result === 'error' && <AlertCircle className="w-4 h-4" />}
                          {testState.embedding.message}
                        </span>
                      )}
                    </div>
                  </div>
                </section>
              </div>
            ) : (
              // ==================== Privacy & Blocklist Tab ====================
              <div className="space-y-8 animate-in fade-in slide-in-from-right-4 duration-300">
                {/* OCR Privacy Settings */}
                <section className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-8 h-8 rounded-lg bg-emerald-500/20 text-emerald-500 flex items-center justify-center">
                        <Eye className="w-5 h-5" />
                      </div>
                      <div>
                        <h3 className="text-lg font-semibold text-white">OCR 隐私脱敏</h3>
                        <p className="text-sm text-gray-400">识别并隐藏图片中的敏感信息</p>
                      </div>
                    </div>
                    <button
                      onClick={() =>
                        setDraftConfig((prev) => ({
                          ...prev,
                          ocrRedactionEnabled: !prev.ocrRedactionEnabled,
                        }))
                      }
                      className={`w-12 h-6 rounded-full transition-colors relative ${
                        draftConfig.ocrRedactionEnabled ? 'bg-emerald-500' : 'bg-gray-600'
                      }`}
                    >
                      <div
                        className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
                          draftConfig.ocrRedactionEnabled ? 'translate-x-6' : 'translate-x-0'
                        }`}
                      />
                    </button>
                  </div>

                  {draftConfig.ocrRedactionEnabled && (
                    <div className="p-4 rounded-xl bg-surface/50 border border-glass-border/30 space-y-4">
                      <label className="block text-sm font-medium text-gray-300">脱敏级别</label>
                      <div className="grid grid-cols-2 gap-3">
                        <button
                          onClick={() =>
                            setDraftConfig((prev) => ({
                              ...prev,
                              ocrRedactionLevel: 'basic',
                            }))
                          }
                          className={`p-3 rounded-lg border text-left transition-colors ${
                            draftConfig.ocrRedactionLevel === 'basic'
                              ? 'bg-emerald-500/20 border-emerald-500'
                              : 'border-glass-border hover:bg-surface/80'
                          }`}
                        >
                          <div className="font-medium text-white mb-1">基础模式</div>
                          <div className="text-xs text-gray-400">
                            仅脱敏手机号、身份证、银行卡、邮箱
                          </div>
                        </button>
                        <button
                          onClick={() =>
                            setDraftConfig((prev) => ({
                              ...prev,
                              ocrRedactionLevel: 'strict',
                            }))
                          }
                          className={`p-3 rounded-lg border text-left transition-colors ${
                            draftConfig.ocrRedactionLevel === 'strict'
                              ? 'bg-emerald-500/20 border-emerald-500'
                              : 'border-glass-border hover:bg-surface/80'
                          }`}
                        >
                          <div className="font-medium text-white mb-1">严格模式</div>
                          <div className="text-xs text-gray-400">
                            脱敏 IP、MAC、金额及所有长数字序列
                          </div>
                        </button>
                      </div>
                    </div>
                  )}
                </section>

                <div className="h-px bg-glass-border/50" />

                {/* Privacy Mode */}
                <section className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-8 h-8 rounded-lg bg-amber-500/20 text-amber-500 flex items-center justify-center">
                        <Shield className="w-5 h-5" />
                      </div>
                      <div>
                        <h3 className="text-lg font-semibold text-white">隐私模式</h3>
                        <p className="text-sm text-gray-400">暂停录制并隐藏敏感内容</p>
                      </div>
                    </div>
                    <button
                      onClick={() =>
                        setDraftConfig((prev) => ({
                          ...prev,
                          privacyModeEnabled: !prev.privacyModeEnabled,
                          privacyModeUntil: !prev.privacyModeEnabled
                            ? Date.now() + 3600 * 1000 // Default 1h
                            : undefined,
                        }))
                      }
                      className={`w-12 h-6 rounded-full transition-colors relative ${
                        draftConfig.privacyModeEnabled ? 'bg-amber-500' : 'bg-gray-600'
                      }`}
                    >
                      <div
                        className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
                          draftConfig.privacyModeEnabled ? 'translate-x-6' : 'translate-x-0'
                        }`}
                      />
                    </button>
                  </div>

                  {draftConfig.privacyModeEnabled && (
                    <div className="p-4 rounded-xl bg-surface/50 border border-glass-border/30 space-y-4">
                      <label className="block text-sm font-medium text-gray-300">自动关闭时间</label>
                      <div className="flex gap-2">
                        {[
                          { label: '1 小时', val: 3600 * 1000 },
                          { label: '4 小时', val: 4 * 3600 * 1000 },
                          { label: '24 小时', val: 24 * 3600 * 1000 },
                        ].map((opt) => (
                          <button
                            key={opt.label}
                            onClick={() =>
                              setDraftConfig((prev) => ({
                                ...prev,
                                privacyModeUntil: Date.now() + opt.val,
                              }))
                            }
                            className={`px-3 py-1.5 rounded-lg text-sm border transition-colors ${
                              draftConfig.privacyModeUntil &&
                              draftConfig.privacyModeUntil - Date.now() <= opt.val &&
                              draftConfig.privacyModeUntil - Date.now() > opt.val - 3600 * 1000 // Rough check
                                ? 'bg-amber-500/20 border-amber-500 text-amber-500'
                                : 'border-glass-border hover:bg-surface/80 text-gray-300'
                            }`}
                          >
                            {opt.label}
                          </button>
                        ))}
                      </div>
                    </div>
                  )}
                </section>

                <div className="h-px bg-glass-border/50" />

                {/* Blocklist */}
                <section className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-8 h-8 rounded-lg bg-red-500/20 text-red-500 flex items-center justify-center">
                        <AlertCircle className="w-5 h-5" />
                      </div>
                      <div>
                        <h3 className="text-lg font-semibold text-white">应用黑名单</h3>
                        <p className="text-sm text-gray-400">禁止录制特定应用的活动</p>
                      </div>
                    </div>
                    <button
                      onClick={() =>
                        setDraftConfig((prev) => ({
                          ...prev,
                          blocklistEnabled: !prev.blocklistEnabled,
                        }))
                      }
                      className={`w-12 h-6 rounded-full transition-colors relative ${
                        draftConfig.blocklistEnabled ? 'bg-red-500' : 'bg-gray-600'
                      }`}
                    >
                      <div
                        className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
                          draftConfig.blocklistEnabled ? 'translate-x-6' : 'translate-x-0'
                        }`}
                      />
                    </button>
                  </div>

                  <div className="p-4 rounded-xl bg-surface/50 border border-glass-border/30 space-y-4">
                    <div className="flex gap-2">
                      <input
                        type="text"
                        value={newBlockItem}
                        onChange={(e) => setNewBlockItem(e.target.value)}
                        placeholder="输入应用名称 (如: chrome / chrome.exe)"
                        className="flex-1 px-4 py-2 bg-surface border border-glass-border rounded-lg text-white placeholder:text-gray-500 focus:outline-none focus:ring-2 focus:ring-red-500/30"
                        onKeyDown={(e) => e.key === 'Enter' && handleAddBlockItem()}
                      />
                      <button
                        onClick={(e) => {
                          e.preventDefault()
                          e.stopPropagation()
                          console.log('[黑名单] 按钮被点击, newBlockItem:', newBlockItem)
                          if (newBlockItem.trim()) {
                            handleAddBlockItem()
                          } else {
                            handleSelectFile()
                          }
                        }}
                        className="px-4 py-2 rounded-lg bg-surface border border-glass-border hover:bg-white/10 active:bg-white/20 active:scale-95 transition-all duration-100"
                        title={newBlockItem.trim() ? "添加" : "选择文件"}
                        type="button"
                      >
                        {newBlockItem.trim() ? (
                          <Plus className="w-5 h-5 text-gray-300" />
                        ) : (
                          <FolderOpen className="w-5 h-5 text-gray-300" />
                        )}
                      </button>
                    </div>
                    {blocklistError && (
                      <div className="text-xs text-red-400 break-words">{blocklistError}</div>
                    )}

                    <div className="max-h-60 overflow-y-auto space-y-2 pr-2 custom-scrollbar">
                      {blocklistLoading ? (
                        <div className="flex justify-center py-4">
                          <Loader2 className="w-5 h-5 animate-spin text-gray-500" />
                        </div>
                      ) : blocklist.length === 0 ? (
                        <p className="text-sm text-gray-500 text-center py-4">暂无黑名单应用</p>
                      ) : (
                        blocklist.map((item) => (
                          <div
                            key={item}
                            className="flex items-center justify-between px-3 py-2 rounded-lg bg-surface border border-glass-border/50 group"
                          >
                            <span className="text-sm text-gray-300">{item}</span>
                            <button
                              onClick={() => handleRemoveBlockItem(item)}
                              className="p-1.5 rounded-md hover:bg-red-500/20 text-gray-500 hover:text-red-400 transition-colors opacity-0 group-hover:opacity-100"
                            >
                              <Trash2 className="w-4 h-4" />
                            </button>
                          </div>
                        ))
                      )}
                    </div>
                  </div>
                </section>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-glass-border/50 bg-surface/80 backdrop-blur-md">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-lg hover:bg-white/10 text-gray-300 transition-colors"
          >
            取消
          </button>
          <button
            onClick={handleSave}
            className="px-6 py-2 rounded-lg bg-neon-blue text-black font-semibold hover:bg-neon-blue/90 transition-colors shadow-lg shadow-neon-blue/20"
          >
            保存配置
          </button>
        </div>
      </div>
    </div>
  )
}
