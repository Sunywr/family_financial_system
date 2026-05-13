import { client, type ApiResponse, requireCurrentUserId, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface Budget {
  id: number
  user_id: number
  budget_month: string
  category_id: number
  category_name: string
  planned_amount: string
  actual_amount: string
  manual_adjusted: boolean
  remark?: string | null
}

export async function fetchBudgets() {
  const month = new Date().toISOString().slice(0, 7) + '-01'
  const response = await client.get<ApiResponse<PageData<Budget>>>('/budgets', {
    params: withCurrentUserId({ page: 1, page_size: 50, budget_month: month })
  })
  return response.data.data
}

export async function generateBudgets(userId: number, budgetMonth: string) {
  const response = await client.post<ApiResponse<{ generated: number }>>('/budgets/generate', {
    user_id: userId || requireCurrentUserId(),
    budget_month: budgetMonth
  })
  return response.data.data
}
