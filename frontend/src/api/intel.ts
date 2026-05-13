import { client, type ApiResponse, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface IntelItem {
  id: number
  user_id: number
  title: string
  source?: string | null
  item_date: string
  status: string
  tags?: string | null
  summary?: string | null
  content?: string | null
}

export interface IntelPayload {
  user_id: number
  title: string
  source?: string | null
  item_date: string
  status: string
  tags?: string | null
  summary?: string | null
  content?: string | null
}

export async function fetchIntel() {
  const response = await client.get<ApiResponse<PageData<IntelItem>>>('/intel', {
    params: withCurrentUserId({ page: 1, page_size: 50 })
  })
  return response.data.data
}

export async function createIntel(payload: IntelPayload) {
  const response = await client.post<ApiResponse<IntelItem>>('/intel', payload)
  return response.data.data
}

export async function updateIntel(id: number, payload: Omit<IntelPayload, 'user_id'>) {
  const response = await client.put<ApiResponse<IntelItem>>(`/intel/${id}`, payload)
  return response.data.data
}

export async function deleteIntel(id: number) {
  const response = await client.delete<ApiResponse<{ deleted: boolean }>>(`/intel/${id}`)
  return response.data.data
}
