import { client, type ApiResponse, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface Debt {
  id: number
  user_id: number
  source_bill_id?: number | null
  start_date: string
  end_date?: string | null
  repay_deadline?: string | null
  category_id: number
  category_name: string
  amount: string
  period_count: number
  period_unit: string
  period_value: number
  paid_period_count: number
  payment_method: string
  status: string
  remark?: string | null
}

export interface UpdateDebtPayload {
  start_date: string
  end_date?: string | null
  repay_deadline?: string | null
  category_id: number
  amount: string
  period_count: number
  period_unit: string
  period_value: number
  payment_method: string
  status: string
  remark?: string | null
}

export async function fetchDebts(
  keyword = '',
  page = 1,
  pageSize = 20,
  paymentMethod?: 'credit_card' | 'non_credit_card'
) {
  const response = await client.get<ApiResponse<PageData<Debt>>>('/debts', {
    params: withCurrentUserId({
      page,
      page_size: pageSize,
      keyword: keyword || undefined,
      payment_method: paymentMethod || undefined
    })
  })
  return response.data.data
}

export async function updateDebt(id: number, payload: UpdateDebtPayload) {
  const response = await client.put<ApiResponse<Debt>>(`/debts/${id}`, payload)
  return response.data.data
}
