import { client, type ApiResponse, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface Asset {
  id: number
  user_id: number
  source_bill_id?: number | null
  name: string
  category_name: string
  amount: string
  status: string
  created_at?: string
  remark?: string | null
}

export async function fetchAssets(keyword = '', page = 1, pageSize = 20) {
  const response = await client.get<ApiResponse<PageData<Asset>>>('/assets', {
    params: withCurrentUserId({ page, page_size: pageSize, keyword: keyword || undefined })
  })
  return response.data.data
}
