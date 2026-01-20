import React, { createContext, useContext, useReducer, ReactNode } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// 类型定义
export interface ActivityLog {
  id: number
  timestamp: number
  appName: string
  windowTitle: string
  imagePath: string
  ocrText?: string
  phash?: string
}

export interface AppState {
  isRecording: boolean
  activities: ActivityLog[]
  currentView: 'timeline' | 'graph' | 'stats' | 'qa' | 'gallery' | 'replay'
  config: AppConfig
  lastSearchParams?: SearchParams
}

export interface AppConfig {
  recordingInterval: number
  ocrEnabled: boolean
  ocrEngine?: string
  ocrRedactionEnabled?: boolean
  ocrRedactionLevel?: 'basic' | 'strict' | string
  aiEnabled: boolean
  enableFocusAnalytics: boolean
  enableProactiveAssistant: boolean
  retentionDays: number
  apiKey?: string
  chatModel?: string
  embeddingModel?: string
  embeddingBaseUrl?: string
  embeddingUseSharedKey?: boolean
  openaiBaseUrl?: string
  anthropicBaseUrl?: string
  blocklistEnabled: boolean
  blocklistMode: string
  privacyModeEnabled: boolean
  privacyModeUntil?: number
  intentParseTimeoutMs?: number
}

export interface SearchParams extends Record<string, unknown> {
  query?: string
  appName?: string
  fromTs?: number
  toTs?: number
  hasOcr?: boolean
  limit?: number
  offset?: number
  orderBy?: 'time' | 'rank'
}

type AppAction =
  | { type: 'SET_RECORDING'; payload: boolean }
  | { type: 'ADD_ACTIVITY'; payload: ActivityLog }
  | { type: 'UPDATE_ACTIVITY_OCR'; payload: { id: number; ocrText: string } }
  | { type: 'SET_ACTIVITIES'; payload: ActivityLog[] }
  | { type: 'SET_VIEW'; payload: 'timeline' | 'graph' | 'stats' | 'qa' | 'gallery' | 'replay' }
  | { type: 'SET_CONFIG'; payload: AppConfig }
  | { type: 'SET_SEARCH_PARAMS'; payload: SearchParams }

const initialState: AppState = {
  isRecording: false,
  activities: [],
  currentView: 'timeline',
  config: {
    recordingInterval: 5000,
    ocrEnabled: true,
    ocrRedactionEnabled: true,
    ocrRedactionLevel: 'basic',
    aiEnabled: false,
    enableFocusAnalytics: false,
    enableProactiveAssistant: false,
    retentionDays: 30,
    blocklistEnabled: false,
    blocklistMode: 'blocklist',
    privacyModeEnabled: false,
  },
}

function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case 'SET_RECORDING':
      return { ...state, isRecording: action.payload }
    case 'ADD_ACTIVITY':
      return { ...state, activities: [action.payload, ...state.activities] }
    case 'UPDATE_ACTIVITY_OCR':
      return {
        ...state,
        activities: state.activities.map((activity) =>
          activity.id === action.payload.id
            ? { ...activity, ocrText: action.payload.ocrText }
            : activity
        ),
      }
    case 'SET_ACTIVITIES':
      return { ...state, activities: action.payload }
    case 'SET_VIEW':
      return { ...state, currentView: action.payload }
    case 'SET_CONFIG':
      return { ...state, config: action.payload }
    case 'SET_SEARCH_PARAMS':
      return { ...state, lastSearchParams: action.payload }
    default:
      return state
  }
}

interface AppContextType {
  state: AppState
  dispatch: React.Dispatch<AppAction>
  startRecording: () => Promise<void>
  stopRecording: () => Promise<void>
  loadActivities: () => Promise<void>
  searchActivities: (params: SearchParams) => Promise<void>
}

const AppContext = createContext<AppContextType | undefined>(undefined)

export function AppProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, initialState)

  const loadConfig = async () => {
    try {
      const config = await invoke<AppConfig>('get_config')
      dispatch({ type: 'SET_CONFIG', payload: config })
    } catch (error) {
      // 如果后端配置尚未初始化或读取失败，则继续使用前端默认值
      console.warn('Failed to load config from backend, using defaults:', error)
    }
  }

  const startRecording = async () => {
    console.log('[DEBUG] Frontend calling start_recording...')
    try {
      await invoke('start_recording')
      console.log('[DEBUG] Backend invoke returned success')
      dispatch({ type: 'SET_RECORDING', payload: true })
    } catch (error) {
      console.error('Failed to start recording:', error)
      alert('Failed to start recording: ' + JSON.stringify(error))
    }
  }

  const stopRecording = async () => {
    try {
      await invoke('stop_recording')
      dispatch({ type: 'SET_RECORDING', payload: false })
    } catch (error) {
      console.error('Failed to stop recording:', error)
    }
  }

  const loadActivities = async () => {
    try {
      const activities = await invoke<ActivityLog[]>('get_activities', {
        limit: 100,
      })
      dispatch({ type: 'SET_ACTIVITIES', payload: activities })
    } catch (error) {
      console.error('Failed to load activities:', error)
    }
  }

  const searchActivities = async (params: SearchParams) => {
    try {
      const result = await invoke<{ items: ActivityLog[]; total: number }>(
        'search_activities',
        {
          query: params.query,
          appName: params.appName,
          fromTs: params.fromTs,
          toTs: params.toTs,
          hasOcr: params.hasOcr,
          limit: params.limit,
          offset: params.offset,
          orderBy: params.orderBy,
        }
      )
      dispatch({ type: 'SET_ACTIVITIES', payload: result.items })
      dispatch({ type: 'SET_SEARCH_PARAMS', payload: params })
    } catch (error) {
      console.error('Failed to search activities:', error)
    }
  }

  // 监听后端事件
  React.useEffect(() => {
    const unlistenRecording = listen('recording-status', (event) => {
      dispatch({ type: 'SET_RECORDING', payload: event.payload as boolean })
    })

    const unlistenActivity = listen('new-activity', (event) => {
      dispatch({ type: 'ADD_ACTIVITY', payload: event.payload as ActivityLog })
    })

    const unlistenOcrUpdate = listen('ocr-updated', (event) => {
      const payload = event.payload as { id: number; ocrText: string }
      dispatch({ type: 'UPDATE_ACTIVITY_OCR', payload })
    })

    const unlistenLog = listen('backend-log', (event) => {
      console.log(`[BACKEND]: ${event.payload}`)
    })

    return () => {
      unlistenRecording.then((fn) => fn())
      unlistenActivity.then((fn) => fn())
      unlistenOcrUpdate.then((fn) => fn())
      unlistenLog.then((fn) => fn())
    }
  }, [])

  // 初始化加载
  React.useEffect(() => {
    loadConfig()
    loadActivities()
  }, [])

  return (
    <AppContext.Provider
      value={{
        state,
        dispatch,
        startRecording,
        stopRecording,
        loadActivities,
        searchActivities,
      }}
    >
      {children}
    </AppContext.Provider>
  )
}

export function useApp() {
  const context = useContext(AppContext)
  if (context === undefined) {
    throw new Error('useApp must be used within an AppProvider')
  }
  return context
}

