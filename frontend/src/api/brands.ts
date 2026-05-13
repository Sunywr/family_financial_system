import { client, type ApiResponse, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface Brand {
  id: number
  user_id: number
  category_id: number
  category_name: string
  brand_name: string
  score: number
  board_type: string
  review?: string | null
  remark?: string | null
}

export async function fetchBrands() {
  const response = await client.get<ApiResponse<PageData<Brand>>>('/brands', {
    params: withCurrentUserId({ page: 1, page_size: 20 })
  })
  return response.data.data
}
