import { client, type ApiResponse } from './client'
import type { PageData } from '@/types/api'

export interface InvestmentTransaction {
  id: number
  investment_id: number
  source_bill_id: number
  transaction_date: string
  action: string
  shares: string
  amount: string
  unit_price: string
  realized_profit: string
  remark?: string | null
}

export async function fetchInvestmentTransactions(keyword = '') {
  const response = await client.get<ApiResponse<PageData<InvestmentTransaction>>>(
    '/investment-transactions',
    { params: { page: 1, page_size: 50, keyword: keyword || undefined } }
  )
  return response.data.data
}
