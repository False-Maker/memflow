import { useState, useEffect, useReducer, useCallback } from 'react'
import { X, Check, AlertCircle, Loader2, ChevronDown, Shield, Settings, Bot, Plus, Trash2, Eye, FolderOpen, Gauge, Sparkles, Download, HardDrive, Database, Trash, Power } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { open as openFileDialog, save as saveFileDialog } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
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
        className={`w-full appearance-none px-4 py-2.5 pr-10 bg-surface border border-glass-border rounded-lg text-white cursor-pointer hover:border-neon-cyan/50 transition-colors focus:outline-none focus:ring-2 focus:ring-neon-cyan/30 ${className}`}
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
          className="flex-1 px-4 py-2.5 bg-surface border border-glass-border rounded-lg text-white placeholder:text-gray-500 hover:border-neon-cyan/50 transition-colors focus:outline-none focus:ring-2 focus:ring-neon-cyan/30"
        />
        {rightElement}
      </div>
      {statusMessage && (
        <p
          className={`text-xs flex items-center gap-1 ${status === 'saved'
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
  const [activeTab, setActiveTab] = useState<'general' | 'privacy' | 'storage'>('general')

  // Blocklist state
  const [blocklist, setBlocklist] = useState<string[]>([])
  const [newBlockItem, setNewBlockItem] = useState('')
  const [blocklistLoading, setBlocklistLoading] = useState(false)
  const [blocklistError, setBlocklistError] = useState<string | null>(null)

  // P5.5-3: Storage stats
  const [storageStats, setStorageStats] = useState<{
    screenshotsCount: number
    screenshotsSizeMb: number
    activitiesCount: number
    databaseSizeMb: number
    totalSizeMb: number
    maxStorageGb: number
    usagePercent: number
    nextGcTime: string | null
  } | null>(null)
  const [storageLoading, setStorageLoading] = useState(false)
  const [storageError, setStorageError] = useState<string | null>(null)

  // P5.5-4: Export state
  const [exporting, setExporting] = useState(false)
  const [exportError, setExportError] = useState<string | null>(null)

  // 自启动状态
  const [autostartEnabled, setAutostartEnabled] = useState(false)
  const [autostartLoading, setAutostartLoading] = useState(false)

  // 数据目录状态
  const [dataDirectory, setDataDirectory] = useState<string | null>(null)

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

      // 加载自启动状态
      loadAutostartStatus()

      // 加载数据目录
      loadDataDirectory()

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

  // P5.5-3: 当切换到存储 tab 时加载存储统计
  useEffect(() => {
    if (activeTab === 'storage') {
      loadStorageStats()
    }
  }, [activeTab])

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

  // 加载自启动状态
  const loadAutostartStatus = async () => {
    try {
      setAutostartLoading(true)
      const info = await invoke<{ enabled: boolean; appName: string }>('get_autostart_status')
      setAutostartEnabled(info.enabled)
    } catch (e) {
      console.error('加载自启动状态失败:', e)
    } finally {
      setAutostartLoading(false)
    }
  }

  // P5.5-3: 加载存储统计
  const loadStorageStats = async () => {
    try {
      setStorageLoading(true)
      setStorageError(null)
      const stats = await invoke<{
        screenshotsCount: number
        screenshotsSizeMb: number
        activitiesCount: number
        databaseSizeMb: number
        totalSizeMb: number
        maxStorageGb: number
        usagePercent: number
        nextGcTime: string | null
      }>('get_storage_stats')
      setStorageStats(stats)
    } catch (e) {
      console.error('加载存储统计失败:', e)
      setStorageError(String(e))
    } finally {
      setStorageLoading(false)
    }
  }

  // P5.5-4: 导出 JSON 数据
  const handleExportJson = async () => {
    try {
      setExporting(true)
      setExportError(null)
      
      const data = await invoke<string>('export_data_json', { limit: 1000 })
      
      const filePath = await saveFileDialog({
        defaultPath: `memflow_export_${new Date().toISOString().split('T')[0]}.json`,
        filters: [{ name: 'JSON', extensions: ['json'] }]
      })
      
      if (filePath) {
        await writeTextFile(filePath, data)
        alert('导出成功！')
      }
    } catch (e) {
      console.error('导出 JSON 失败:', e)
      setExportError(String(e))
    } finally {
      setExporting(false)
    }
  }

  // P5.5-4: 导出 Markdown 数据
  const handleExportMarkdown = async () => {
    try {
      setExporting(true)
      setExportError(null)
      
      const data = await invoke<string>('export_data_markdown', { limit: 1000 })
      
      const filePath = await saveFileDialog({
        defaultPath: `memflow_export_${new Date().toISOString().split('T')[0]}.md`,
        filters: [{ name: 'Markdown', extensions: ['md'] }]
      })
      
      if (filePath) {
        await writeTextFile(filePath, data)
        alert('导出成功！')
      }
    } catch (e) {
      console.error('导出 Markdown 失败:', e)
      setExportError(String(e))
    } finally {
      setExporting(false)
    }
  }

  // P5.5-4: 一键清理
  const handleClearAllData = async () => {
    const confirmed = confirm('确定要清空所有数据吗？此操作不可恢复！')
    if (!confirmed) return
    
    try {
      const result = await invoke<{
        deletedActivities: number
        deletedScreenshots: number
        freedBytes: number
      }>('clear_all_data')
      
      alert(`清理完成！\n删除 ${result.deletedActivities} 条活动记录\n删除 ${result.deletedScreenshots} 张截图\n释放 ${(result.freedBytes / 1024 / 1024).toFixed(2)} MB`)
      
      // 刷新存储统计
      loadStorageStats()
    } catch (e) {
      console.error('清理数据失败:', e)
      alert('清理失败: ' + e)
    }
  }

  // 数据目录相关处理
  const handleSelectDataDirectory = async () => {
    try {
      const selected = await openFileDialog({
        directory: true,
        multiple: false,
        title: '选择数据存储目录'
      })
      if (selected && typeof selected === 'string') {
        // 更新配置
        await invoke('set_data_directory', { path: selected })
        setDataDirectory(selected)
        alert('数据目录已更新，请重启应用以使更改生效')
      }
    } catch (e) {
      console.error('选择目录失败:', e)
      if (e !== 'Canceled') {
        alert('选择目录失败: ' + e)
      }
    }
  }

  const handleResetDataDirectory = async () => {
    try {
      // 清空配置以使用默认目录
      await invoke('set_data_directory', { path: '' })
      setDataDirectory(null)
      alert('已重置为默认目录，请重启应用以使更改生效')
    } catch (e) {
      console.error('重置目录失败:', e)
      alert('重置失败: ' + e)
    }
  }

  // 加载数据目录
  const loadDataDirectory = async () => {
    try {
      const dir = await invoke<string>('get_data_directory')
      setDataDirectory(dir)
    } catch (e) {
      console.error('加载数据目录失败:', e)
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

      if (!state.config.aiEnabled) {
        try {
          const configWithAiEnabled = { ...state.config, aiEnabled: true }
          await invoke('update_config', { config: configWithAiEnabled })
          dispatch({ type: 'SET_CONFIG', payload: configWithAiEnabled })
          setDraftConfig((prev) => ({ ...prev, aiEnabled: true }))
        } catch (e) {
          console.error('启用 AI 失败:', e)
        }
      }

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
      const safeMsg = msg
        .replace(/Incorrect API key provided:\s*([^\s".\r\n]+)/g, 'Incorrect API key provided: [REDACTED]')
        .replace(/Bearer\s+([^\s"'\r\n]+)/g, 'Bearer [REDACTED]')
        .replace(/sk-[A-Za-z0-9_-]+/g, 'sk-[REDACTED]')
      const isInvalidKey =
        /invalid_api_key/i.test(safeMsg) ||
        /401\s+Unauthorized/i.test(safeMsg) ||
        /Incorrect API key provided/i.test(safeMsg)
      const displayMsg = isInvalidKey
        ? 'API Key 无效或已失效，请在设置中重新保存后再测试'
        : safeMsg
      setTestState((prev) => ({
        ...prev,
        [type]: { testing: false, result: 'error', message: `连接失败：${displayMsg}` },
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
        compressionQuality: draftConfig.compressionQuality,
        targetResolutionScale: draftConfig.targetResolutionScale,
      }

      await invoke('update_config', { config: updatedConfig })
      dispatch({ type: 'SET_CONFIG', payload: updatedConfig })
      onClose()
    } catch (e) {
      console.error('保存配置失败:', e)
      alert('保存配置失败: ' + e)
    }
  }

  // Handle body overflow when modal is open
  useEffect(() => {
    // Save original overflow and prevent background scrolling
    const originalStyle = window.getComputedStyle(document.body).overflow
    document.body.style.overflow = 'hidden'

    return () => {
      document.body.style.overflow = originalStyle
    }
  }, [])

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
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* 背景遮罩 - 毛玻璃 */}
      <div className="absolute inset-0 bg-black/70 backdrop-blur-md" onClick={onClose} />
      
      {/* 弹窗主体 - 官网风格 */}
      <div className="relative w-full max-w-3xl h-[80vh] bg-black/90 backdrop-blur-xl border border-white/10 rounded-xl overflow-hidden shadow-2xl animate-in zoom-in-95 duration-200 flex flex-col">
        {/* 斜切装饰 - 官网特色 */}
        <div className="absolute top-0 right-0 w-24 h-24 pointer-events-none">
          <div className="absolute top-0 right-0 w-full h-full bg-gradient-to-bl from-neon-cyan/10 to-transparent transform skew-x-12 translate-x-8 -translate-y-4" />
        </div>
        
        {/* 顶部装饰线 */}
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-neon-cyan/50 to-transparent" />
        
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-white/5">
          <h2 className="text-lg font-bold text-white flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-neon-cyan/20 text-neon-cyan flex items-center justify-center">
              <Settings className="w-4 h-4" />
            </div>
            设置
          </h2>
          <button
            onClick={onClose}
            className="p-2 text-zinc-500 hover:text-white hover:bg-white/5 rounded-lg transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="flex flex-1 overflow-hidden min-h-0 flex-row">
          {/* Sidebar */}
          <div className="w-48 border-r border-glass-border/50 bg-surface/30 p-4 space-y-2">
            <button
              onClick={() => setActiveTab('general')}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-colors ${activeTab === 'general'
                ? 'bg-neon-cyan/20 text-neon-cyan'
                : 'text-gray-400 hover:bg-white/5 hover:text-white'
                }`}
            >
              <Bot className="w-4 h-4" />
              模型设置
            </button>
            <button
              onClick={() => setActiveTab('privacy')}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-colors ${activeTab === 'privacy'
                ? 'bg-neon-cyan/20 text-neon-cyan'
                : 'text-gray-400 hover:bg-white/5 hover:text-white'
                }`}
            >
              <Shield className="w-4 h-4" />
              隐私与屏蔽
            </button>
            <button
              onClick={() => setActiveTab('storage')}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-colors ${activeTab === 'storage'
                ? 'bg-neon-cyan/20 text-neon-cyan'
                : 'text-gray-400 hover:bg-white/5 hover:text-white'
                }`}
            >
              <HardDrive className="w-4 h-4" />
              存储管理
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-6">
            {activeTab === 'general' ? (
              <div className="space-y-8">
                <section className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-8 h-8 rounded-lg bg-neon-cyan/20 text-neon-cyan flex items-center justify-center">
                        <Sparkles className="w-5 h-5" />
                      </div>
                      <div>
                        <h3 className="text-lg font-semibold text-white">AI 能力</h3>
                        <p className="text-sm text-gray-400">开启后允许使用大模型功能（智能搜索、上下文助理等）</p>
                      </div>
                    </div>
                    <button
                      onClick={() =>
                        setDraftConfig((prev) => ({
                          ...prev,
                          aiEnabled: !prev.aiEnabled,
                        }))
                      }
                      className={`w-12 h-6 rounded-full transition-colors relative ${draftConfig.aiEnabled ? 'bg-neon-cyan' : 'bg-gray-600'
                        }`}
                    >
                      <div
                        className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${draftConfig.aiEnabled ? 'translate-x-6' : 'translate-x-0'
                          }`}
                      />
                    </button>
                  </div>
                </section>

                {/* Proactive Context Assistant Toggle (only when AI is enabled) */}
                {/* Proactive Context Assistant Toggle (only when AI is enabled) */}
                {draftConfig.aiEnabled && (
                  <section className="space-y-4">
                    <div className="flex items-center justify-between p-4 rounded-xl bg-surface/30 border border-glass-border/30">
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 rounded-lg bg-neon-cyan/20 text-neon-cyan flex items-center justify-center">
                          <Sparkles className="w-5 h-5" />
                        </div>
                        <div>
                          <h4 className="text-sm font-semibold text-white">上下文助理</h4>
                          <p className="text-xs text-gray-400">检测窗口切换时主动推送 AI 建议</p>
                        </div>
                      </div>
                      <button
                        onClick={() =>
                          setDraftConfig((prev) => ({
                            ...prev,
                            enableProactiveAssistant: !prev.enableProactiveAssistant,
                          }))
                        }
                        className={`w-12 h-6 rounded-full transition-colors relative ${draftConfig.enableProactiveAssistant ? 'bg-neon-cyan' : 'bg-gray-600'
                          }`}
                      >
                        <div
                          className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${draftConfig.enableProactiveAssistant ? 'translate-x-6' : 'translate-x-0'
                            }`}
                        />
                      </button>
                    </div>
                  </section>
                )}

                {/* Autostart Settings */}
                <section className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-8 h-8 rounded-lg bg-neon-cyan/20 text-neon-cyan flex items-center justify-center">
                        <Power className="w-5 h-5" />
                      </div>
                      <div>
                        <h3 className="text-lg font-semibold text-white">开机自启动</h3>
                        <p className="text-sm text-gray-400">系统启动时自动运行 MemFlow</p>
                      </div>
                    </div>
                    <button
                      onClick={async () => {
                        if (autostartLoading) return
                        try {
                          setAutostartLoading(true)
                          if (autostartEnabled) {
                            await invoke('disable_autostart')
                            setAutostartEnabled(false)
                          } else {
                            await invoke('enable_autostart')
                            setAutostartEnabled(true)
                          }
                        } catch (e) {
                          console.error('设置自启动失败:', e)
                          alert('设置自启动失败: ' + e)
                        } finally {
                          setAutostartLoading(false)
                        }
                      }}
                      disabled={autostartLoading}
                      className={`w-12 h-6 rounded-full transition-colors relative ${autostartEnabled ? 'bg-neon-cyan' : 'bg-gray-600'
                        } disabled:opacity-50`}
                    >
                      {autostartLoading && (
                        <div className="absolute inset-0 flex items-center justify-center">
                          <Loader2 className="w-3 h-3 animate-spin text-white" />
                        </div>
                      )}
                      <div
                        className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${autostartEnabled ? 'translate-x-6' : 'translate-x-0'
                          }`}
                      />
                    </button>
                  </div>
                </section>

                {/* Recording Settings */}
                <section className="space-y-4">
                  <div className="flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-neon-cyan/20 text-neon-cyan flex items-center justify-center">
                      <Eye className="w-5 h-5" />
                    </div>
                    <h3 className="text-lg font-semibold text-white">录制设置</h3>
                  </div>
                  <div className="p-4 rounded-xl bg-surface/50 border border-glass-border/30 space-y-4">
                    <InputField
                      label="录制间隔 (ms)"
                      value={String(draftConfig.recordingInterval || 5000)}
                      onChange={(v) => {
                        const val = parseInt(v) || 5000;
                        setDraftConfig(prev => ({ ...prev, recordingInterval: val }));
                      }}
                      type="text"
                      placeholder="5000"
                      hint="越小越精准，但会增加存储占用 (最少 100ms)"
                    />

                    <div className="space-y-1.5 pt-2">
                      <label className="block text-sm font-medium text-gray-300">
                        图片质量 (压缩率) <span className="text-neon-cyan ml-2">{draftConfig.compressionQuality || 80}%</span>
                      </label>
                      <input
                        type="range"
                        min="10"
                        max="100"
                        step="10"
                        value={draftConfig.compressionQuality || 80}
                        onChange={(e) => setDraftConfig(prev => ({ ...prev, compressionQuality: parseInt(e.target.value) }))}
                        className="w-full accent-neon-cyan h-2 rounded-lg appearance-none cursor-pointer bg-glass-border"
                      />
                      <p className="text-xs text-gray-500">数值越小文件越小，但画面会变模糊 (推荐 60-80)</p>
                    </div>

                    <div className="space-y-1.5 pt-2">
                      <label className="block text-sm font-medium text-gray-300">
                        分辨率缩放 <span className="text-neon-cyan ml-2">{draftConfig.targetResolutionScale || 1.0}x</span>
                      </label>
                      <input
                        type="range"
                        min="0.5"
                        max="1.0"
                        step="0.1"
                        value={draftConfig.targetResolutionScale || 1.0}
                        onChange={(e) => setDraftConfig(prev => ({ ...prev, targetResolutionScale: parseFloat(e.target.value) }))}
                        className="w-full accent-neon-cyan h-2 rounded-lg appearance-none cursor-pointer bg-glass-border"
                      />
                      <p className="text-xs text-gray-500">降低分辨率可大幅减小体积 (如 0.5 表示长宽各缩小一半)</p>
                    </div>
                  </div>
                </section>

                {/* ==================== Chat Model Section ==================== */}
                <section className="space-y-4">
                  <div className="flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-neon-cyan to-neon-purple flex items-center justify-center">
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
                          className={`text-sm flex items-center gap-1 ${testState.chat.result === 'success'
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
                        className={`w-10 h-6 rounded-full transition-colors relative ${formState.embedding.useSharedKey ? 'bg-neon-cyan' : 'bg-gray-600'
                          }`}
                      >
                        <div
                          className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${formState.embedding.useSharedKey ? 'translate-x-4' : 'translate-x-0'
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
                              className="px-4 py-2 rounded-lg bg-neon-cyan text-black font-medium hover:bg-neon-cyan/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
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
                          className={`text-sm flex items-center gap-1 ${testState.embedding.result === 'success'
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
            ) : activeTab === 'privacy' ? (
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
                      className={`w-12 h-6 rounded-full transition-colors relative ${draftConfig.ocrRedactionEnabled ? 'bg-emerald-500' : 'bg-gray-600'
                        }`}
                    >
                      <div
                        className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${draftConfig.ocrRedactionEnabled ? 'translate-x-6' : 'translate-x-0'
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
                          className={`p-3 rounded-lg border text-left transition-colors ${draftConfig.ocrRedactionLevel === 'basic'
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
                          className={`p-3 rounded-lg border text-left transition-colors ${draftConfig.ocrRedactionLevel === 'strict'
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
                      className={`w-12 h-6 rounded-full transition-colors relative ${draftConfig.privacyModeEnabled ? 'bg-amber-500' : 'bg-gray-600'
                        }`}
                    >
                      <div
                        className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${draftConfig.privacyModeEnabled ? 'translate-x-6' : 'translate-x-0'
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
                            className={`px-3 py-1.5 rounded-lg text-sm border transition-colors ${draftConfig.privacyModeUntil &&
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

                <section className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-8 h-8 rounded-lg bg-neon-cyan/20 text-neon-cyan flex items-center justify-center">
                        <Sparkles className="w-5 h-5" />
                      </div>
                      <div>
                        <h3 className="text-lg font-semibold text-white">主动式 AI 助理</h3>
                        <p className="text-sm text-gray-400">根据当前窗口推送相关记忆与建议</p>
                      </div>
                    </div>
                    <button
                      onClick={() =>
                        setDraftConfig((prev) => ({
                          ...prev,
                          enableProactiveAssistant: !prev.enableProactiveAssistant,
                        }))
                      }
                      className={`w-12 h-6 rounded-full transition-colors relative ${draftConfig.enableProactiveAssistant ? 'bg-neon-cyan' : 'bg-gray-600'
                        }`}
                    >
                      <div
                        className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${draftConfig.enableProactiveAssistant ? 'translate-x-6' : 'translate-x-0'
                          }`}
                      />
                    </button>
                  </div>
                </section>

                <div className="h-px bg-glass-border/50" />

                <section className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-8 h-8 rounded-lg bg-purple-500/20 text-purple-400 flex items-center justify-center">
                        <Gauge className="w-5 h-5" />
                      </div>
                      <div>
                        <h3 className="text-lg font-semibold text-white">专注度分析</h3>
                        <p className="text-sm text-gray-400">
                          仅记录输入强度与应用切换频率，不记录具体按键内容
                        </p>
                      </div>
                    </div>
                    <button
                      onClick={() =>
                        setDraftConfig((prev) => ({
                          ...prev,
                          enableFocusAnalytics: !prev.enableFocusAnalytics,
                        }))
                      }
                      className={`w-12 h-6 rounded-full transition-colors relative ${draftConfig.enableFocusAnalytics ? 'bg-purple-500' : 'bg-gray-600'
                        }`}
                    >
                      <div
                        className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${draftConfig.enableFocusAnalytics ? 'translate-x-6' : 'translate-x-0'
                          }`}
                      />
                    </button>
                  </div>
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
                      className={`w-12 h-6 rounded-full transition-colors relative ${draftConfig.blocklistEnabled ? 'bg-red-500' : 'bg-gray-600'
                        }`}
                    >
                      <div
                        className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${draftConfig.blocklistEnabled ? 'translate-x-6' : 'translate-x-0'
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
            ) : activeTab === 'storage' ? (
              // ==================== Storage Management Tab ====================
              <div className="space-y-8 animate-in fade-in slide-in-from-right-4 duration-300">
                {/* Storage Stats */}
                <section className="space-y-4">
                  <div className="flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-neon-cyan/20 text-neon-cyan flex items-center justify-center">
                      <HardDrive className="w-5 h-5" />
                    </div>
                    <h3 className="text-lg font-semibold text-white">存储使用情况</h3>
                  </div>
                  
                  {storageLoading ? (
                    <div className="flex justify-center py-8">
                      <Loader2 className="w-6 h-6 animate-spin text-neon-cyan" />
                    </div>
                  ) : storageError ? (
                    <div className="p-4 rounded-xl bg-red-500/10 border border-red-500/30 text-red-400">
                      {storageError}
                    </div>
                  ) : storageStats && (
                    <div className="space-y-4">
                      {/* Usage Bar */}
                      <div className="p-4 rounded-xl bg-surface/50 border border-glass-border/30">
                        <div className="flex justify-between items-center mb-2">
                          <span className="text-sm text-gray-300">存储使用</span>
                          <span className="text-sm font-medium text-white">
                            {storageStats.totalSizeMb.toFixed(2)} MB / {storageStats.maxStorageGb} GB
                          </span>
                        </div>
                        <div className="w-full h-3 bg-gray-700 rounded-full overflow-hidden">
                          <div 
                            className={`h-full transition-all ${
                              storageStats.usagePercent > 90 ? 'bg-red-500' :
                              storageStats.usagePercent > 70 ? 'bg-amber-500' : 'bg-neon-cyan'
                            }`}
                            style={{ width: `${Math.min(storageStats.usagePercent, 100)}%` }}
                          />
                        </div>
                        <p className="text-xs text-gray-500 mt-2">
                          使用率: {storageStats.usagePercent.toFixed(1)}%
                        </p>
                      </div>

                      {/* Stats Grid */}
                      <div className="grid grid-cols-2 gap-4">
                        <div className="p-4 rounded-xl bg-surface/30 border border-glass-border/30">
                          <div className="flex items-center gap-2 text-gray-400 mb-1">
                            <Database className="w-4 h-4" />
                            <span className="text-xs">截图</span>
                          </div>
                          <p className="text-xl font-semibold text-white">{storageStats.screenshotsCount}</p>
                          <p className="text-xs text-gray-500">{storageStats.screenshotsSizeMb.toFixed(2)} MB</p>
                        </div>
                        <div className="p-4 rounded-xl bg-surface/30 border border-glass-border/30">
                          <div className="flex items-center gap-2 text-gray-400 mb-1">
                            <HardDrive className="w-4 h-4" />
                            <span className="text-xs">活动记录</span>
                          </div>
                          <p className="text-xl font-semibold text-white">{storageStats.activitiesCount}</p>
                          <p className="text-xs text-gray-500">{storageStats.databaseSizeMb.toFixed(2)} MB</p>
                        </div>
                      </div>

                      {/* Next GC */}
                      {storageStats.nextGcTime && (
                        <p className="text-xs text-gray-500 text-center">
                          下次自动清理: {storageStats.nextGcTime}
                        </p>
                      )}
                    </div>
                  )}
                </section>

                <div className="h-px bg-glass-border/50" />

                {/* Data Directory */}
                <section className="space-y-4">
                  <div className="flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-blue-500/20 text-blue-400 flex items-center justify-center">
                      <FolderOpen className="w-5 h-5" />
                    </div>
                    <h3 className="text-lg font-semibold text-white">数据目录</h3>
                  </div>

                  <div className="p-4 rounded-xl bg-surface/50 border border-glass-border/30 space-y-4">
                    <p className="text-sm text-gray-400">自定义数据存储位置（留空使用默认位置）</p>

                    <div className="space-y-3">
                      <div className="flex gap-2">
                        <input
                          type="text"
                          value={dataDirectory || '默认位置'}
                          readOnly
                          placeholder="使用默认应用数据目录"
                          className="flex-1 px-4 py-2 bg-surface border border-glass-border rounded-lg text-gray-300 placeholder:text-gray-500"
                        />
                        <button
                          onClick={handleSelectDataDirectory}
                          className="px-4 py-2 rounded-lg bg-surface border border-glass-border hover:bg-white/10 text-gray-300 transition-colors flex items-center gap-2"
                        >
                          <FolderOpen className="w-4 h-4" />
                          浏览
                        </button>
                      </div>

                      {(draftConfig.dataDirectory || dataDirectory) && (
                        <button
                          onClick={handleResetDataDirectory}
                          className="text-xs text-red-400 hover:text-red-300 transition-colors flex items-center gap-1"
                        >
                          <Trash2 className="w-3 h-3" />
                          重置为默认目录
                        </button>
                      )}

                      <p className="text-xs text-gray-500">
                        ⚠️ 修改数据目录后需要重启应用才能生效
                      </p>
                    </div>
                  </div>
                </section>

                <div className="h-px bg-glass-border/50" />

                {/* Retention Policy */}
                <section className="space-y-4">
                  <div className="flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-emerald-500/20 text-emerald-500 flex items-center justify-center">
                      <Sparkles className="w-5 h-5" />
                    </div>
                    <h3 className="text-lg font-semibold text-white">保留策略</h3>
                  </div>
                  
                  <div className="p-4 rounded-xl bg-surface/50 border border-glass-border/30 space-y-4">
                    <div className="space-y-2">
                      <label className="block text-sm font-medium text-gray-300">
                        保留最近 <span className="text-neon-cyan">{draftConfig.retentionDays || 30}</span> 天的数据
                      </label>
                      <input
                        type="range"
                        min="7"
                        max="365"
                        step="7"
                        value={draftConfig.retentionDays || 30}
                        onChange={(e) => setDraftConfig(prev => ({ ...prev, retentionDays: parseInt(e.target.value) }))}
                        className="w-full accent-neon-cyan h-2 rounded-lg appearance-none cursor-pointer bg-glass-border"
                      />
                      <p className="text-xs text-gray-500">超过此天数的数据将被自动清理</p>
                    </div>

                    <div className="space-y-2">
                      <label className="block text-sm font-medium text-gray-300">
                        最大存储空间 <span className="text-neon-cyan">{draftConfig.maxStorageGb || 10} GB</span>
                      </label>
                      <input
                        type="range"
                        min="1"
                        max="100"
                        step="1"
                        value={draftConfig.maxStorageGb || 10}
                        onChange={(e) => setDraftConfig(prev => ({ ...prev, maxStorageGb: parseInt(e.target.value) }))}
                        className="w-full accent-neon-cyan h-2 rounded-lg appearance-none cursor-pointer bg-glass-border"
                      />
                      <p className="text-xs text-gray-500">超过此限制时将优先清理更早的数据</p>
                    </div>
                  </div>
                </section>

                <div className="h-px bg-glass-border/50" />

                {/* Export & Clear */}
                <section className="space-y-4">
                  <div className="flex items-center gap-2">
                    <div className="w-8 h-8 rounded-lg bg-purple-500/20 text-purple-400 flex items-center justify-center">
                      <Download className="w-5 h-5" />
                    </div>
                    <h3 className="text-lg font-semibold text-white">数据导出与清理</h3>
                  </div>
                  
                  <div className="p-4 rounded-xl bg-surface/50 border border-glass-border/30 space-y-4">
                    <p className="text-sm text-gray-400">导出您的活动记录为可读格式</p>
                    
                    <div className="flex gap-3">
                      <button
                        onClick={handleExportJson}
                        disabled={exporting}
                        className="flex-1 px-4 py-2 rounded-lg bg-surface border border-glass-border hover:bg-white/10 text-gray-300 transition-colors disabled:opacity-50 flex items-center justify-center gap-2"
                      >
                        {exporting ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
                        导出 JSON
                      </button>
                      <button
                        onClick={handleExportMarkdown}
                        disabled={exporting}
                        className="flex-1 px-4 py-2 rounded-lg bg-surface border border-glass-border hover:bg-white/10 text-gray-300 transition-colors disabled:opacity-50 flex items-center justify-center gap-2"
                      >
                        {exporting ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
                        导出 Markdown
                      </button>
                    </div>
                    
                    {exportError && (
                      <p className="text-xs text-red-400">{exportError}</p>
                    )}
                  </div>

                  <div className="p-4 rounded-xl bg-red-500/10 border border-red-500/30 space-y-4">
                    <div className="flex items-center gap-2 text-red-400">
                      <AlertCircle className="w-5 h-5" />
                      <h4 className="font-medium">危险区域</h4>
                    </div>
                    <p className="text-sm text-gray-400">
                      一键清理将删除所有活动记录、截图和 OCR 数据。此操作不可恢复！
                    </p>
                    <button
                      onClick={handleClearAllData}
                      className="w-full px-4 py-2 rounded-lg bg-red-500 hover:bg-red-600 text-white transition-colors flex items-center justify-center gap-2"
                    >
                      <Trash className="w-4 h-4" />
                      一键清理所有数据
                    </button>
                  </div>
                </section>
              </div>
            ) : null}
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
            className="px-6 py-2 rounded-lg bg-neon-cyan text-black font-semibold hover:bg-neon-cyan/90 transition-colors shadow-lg shadow-neon-cyan/20"
          >
            保存配置
          </button>
        </div>
      </div>
    </div>
  )
}
