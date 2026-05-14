import { client, type ApiResponse, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface Presale {
  id: number
  user_id: number
  source_bill_id?: number | null
  deposit_date: string
  category_name: string
  deposit_amount: string
  final_payment_amount: string
  final_payment_date?: string | null
  status: string
  remark?: string | null
}

export async function fetchPresales(keyword = '', page = 1, pageSize = 20) {
  const response = await client.get<ApiResponse<PageData<Presale>>>('/presales', {
    params: withCurrentUserId({ page, page_size: pageSize, keyword: keyword || undefined })
  })
  return response.data.data
}
