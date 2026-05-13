import { client, type ApiResponse, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface BillTag {
  id: number
  user_id: number
  name: string
  color?: string | null
}

export async function fetchBillTags(keyword = '') {
  const response = await client.get<ApiResponse<PageData<BillTag>>>('/bill-tags', {
    params: withCurrentUserId({ page: 1, page_size: 100, keyword: keyword || undefined })
  })
  return response.data.data
}

export async function createBillTag(payload: { user_id: number; name: string; color?: string }) {
  const response = await client.post<ApiResponse<{ id: number }>>('/bill-tags', payload)
  return response.data.data
}

export async function updateBillTag(id: number, payload: { name: string; color?: string }) {
  const response = await client.put<ApiResponse<{ updated: boolean }>>(`/bill-tags/${id}`, payload)
  return response.data.data
}

export async function deleteBillTag(id: number) {
  const response = await client.delete<ApiResponse<{ deleted: boolean }>>(`/bill-tags/${id}`)
  return response.data.data
}
