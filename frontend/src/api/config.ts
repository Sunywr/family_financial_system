import { client, type ApiResponse, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface ConfigItem {
  id: number
  config_type: string
  name: string
  display_name: string
  is_builtin: boolean
  enabled: boolean
  sort_order: number
}

export interface ConfigTypeOption {
  code: string
  label: string
}

export interface CreateConfigItemRequest {
  config_type: string
  name: string
  display_name: string
  enabled: boolean
  sort_order: number
}

export type UpdateConfigItemRequest = Omit<CreateConfigItemRequest, 'config_type' | 'name'>

export interface CreditCard {
  id: number
  user_id: number
  name: string
  billing_day: number
  repayment_day: number
  credit_limit: string
  enabled: boolean
}

export async function fetchConfigItems(configType?: string, page = 1, pageSize = 100) {
  const response = await client.get<ApiResponse<PageData<ConfigItem>>>('/config/items', {
    params: { page, page_size: pageSize, config_type: configType || undefined }
  })
  return response.data.data
}

export async function fetchConfigTypes() {
  const response = await client.get<ApiResponse<ConfigTypeOption[]>>('/config/types')
  return response.data.data
}

export async function createConfigItem(payload: CreateConfigItemRequest) {
  const response = await client.post<ApiResponse<ConfigItem>>('/config/items', payload)
  return response.data.data
}

export async function updateConfigItem(id: number, payload: UpdateConfigItemRequest) {
  const response = await client.put<ApiResponse<ConfigItem>>(`/config/items/${id}`, payload)
  return response.data.data
}

export async function deleteConfigItem(id: number) {
  const response = await client.delete<ApiResponse<{ deleted: boolean }>>(`/config/items/${id}`)
  return response.data.data
}

export async function fetchCreditCards(page = 1, pageSize = 20) {
  const response = await client.get<ApiResponse<PageData<CreditCard>>>('/config/credit-cards', {
    params: withCurrentUserId({ page, page_size: pageSize })
  })
  return response.data.data
}
