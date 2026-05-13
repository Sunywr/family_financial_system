import { client, type ApiResponse, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface StrategyConfig {
  id: number
  user_id: number
  investment_type: string
  target_code?: string | null
  strategy_name: string
  enabled: boolean
  risk_level: string
  preferred_min_score: number
  cooldown_days: number
  take_profit_rate: string
  stop_loss_rate: string
  notes?: string | null
}

export interface StrategyPayload {
  user_id: number
  investment_type: string
  target_code?: string | null
  strategy_name: string
  enabled: boolean
  risk_level: string
  preferred_min_score: number
  cooldown_days: number
  take_profit_rate: string
  stop_loss_rate: string
  notes?: string | null
}

export async function fetchStrategies() {
  const response = await client.get<ApiResponse<PageData<StrategyConfig>>>('/strategies', {
    params: withCurrentUserId({ page: 1, page_size: 50 })
  })
  return response.data.data
}

export async function createStrategy(payload: StrategyPayload) {
  const response = await client.post<ApiResponse<StrategyConfig>>('/strategies', payload)
  return response.data.data
}

export async function updateStrategy(id: number, payload: Omit<StrategyPayload, 'user_id'>) {
  const response = await client.put<ApiResponse<StrategyConfig>>(`/strategies/${id}`, payload)
  return response.data.data
}

export async function deleteStrategy(id: number) {
  const response = await client.delete<ApiResponse<{ deleted: boolean }>>(`/strategies/${id}`)
  return response.data.data
}
