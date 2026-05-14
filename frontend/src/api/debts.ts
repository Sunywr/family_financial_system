import { client, type ApiResponse, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface Debt {
  id: number
  user_id: number
  source_bill_id?: number | null
  start_date: string
  end_date?: string | null
  repay_deadline?: string | null
  category_name: string
  amount: string
  period_count: number
  period_unit: string
  payment_method: string
  status: string
  remark?: string | null
}

export async function fetchDebts(keyword = '', page = 1, pageSize = 20) {
  const response = await client.get<ApiResponse<PageData<Debt>>>('/debts', {
    params: withCurrentUserId({ page, page_size: pageSize, keyword: keyword || undefined })
  })
  return response.data.data
}
