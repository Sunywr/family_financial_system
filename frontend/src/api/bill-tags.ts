import { client, type ApiResponse, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface BillTag {
  id: number
  user_id: number
  name: string
}

export interface SyncLegacyBillTagsResult {
  imported: number
  skipped: number
  source_total: number
  target_total: number
}

export async function fetchBillTagsPage(page = 1, pageSize = 20, keyword = '') {
  const response = await client.get<ApiResponse<PageData<BillTag>>>('/bill-tags', {
    params: withCurrentUserId({ page, page_size: pageSize, keyword: keyword || undefined })
  })
  return response.data.data
}

export async function fetchBillTags(keyword = '') {
  const pageSize = 200
  let page = 1
  let total = 0
  const merged: BillTag[] = []

  while (true) {
    const data = await fetchBillTagsPage(page, pageSize, keyword)
    if (page === 1) total = data.total
    merged.push(...data.list)

    if (merged.length >= total || data.list.length === 0) {
      break
    }
    page += 1
  }

  return {
    list: merged,
    total,
    page: 1,
    page_size: merged.length || pageSize
  }
}

export async function createBillTag(payload: { user_id: number; name: string }) {
  const response = await client.post<ApiResponse<{ id: number }>>('/bill-tags', payload)
  return response.data.data
}

export async function updateBillTag(id: number, payload: { name: string }) {
  const response = await client.put<ApiResponse<{ updated: boolean }>>(`/bill-tags/${id}`, payload)
  return response.data.data
}

export async function deleteBillTag(id: number) {
  const response = await client.delete<ApiResponse<{ deleted: boolean }>>(`/bill-tags/${id}`)
  return response.data.data
}

export async function fetchTopBillTags(userId: number): Promise<BillTag[]> {
  const response = await client.get<ApiResponse<BillTag[]>>('/bill-tags/top', {
    params: { user_id: userId }
  })
  return response.data.data
}

export async function syncBillTagsFromLegacy(userId: number): Promise<SyncLegacyBillTagsResult> {
  const response = await client.post<ApiResponse<SyncLegacyBillTagsResult>>('/bill-tags/sync-legacy', {
    user_id: userId
  })
  return response.data.data
}
