import { client, type ApiResponse, requireCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface Investment {
  id: number
  user_id: number
  investment_type: string
  name: string
  code: string
  organization_name: string
  total_shares: string
  total_cost: string
  average_cost: string
  current_price: string
  market_value: string
  realized_profit: string
  unrealized_profit: string
  total_profit: string
  total_profit_rate: string
  latest_remark?: string | null
  status: string
}

export interface InvestmentRecommendation {
  investment_id: number
  user_id: number
  investment_type: string
  name: string
  code: string
  organization_name: string
  current_price: string
  market_value: string
  total_profit_rate: string
  score: number
  suggestion: string
  reason: string
  indicator_date: string
}

export interface UpdateInvestmentPayload {
  current_price: string
  market_value: string
  total_shares: string
  total_cost: string
}

export async function fetchInvestments(params?: {
  user_id?: number
  investment_type?: string
  show_sold?: boolean
  keyword?: string
  page?: number
  page_size?: number
}) {
  const response = await client.get<ApiResponse<PageData<Investment>>>('/investments', {
    params: {
      user_id: params?.user_id,
      page: params?.page ?? 1,
      page_size: params?.page_size ?? 50,
      investment_type: params?.investment_type,
      show_sold: params?.show_sold,
      keyword: params?.keyword
    }
  })
  return response.data.data
}

export async function fetchTopInvestments(investmentType?: string, userId?: number) {
  const response = await client.get<ApiResponse<InvestmentRecommendation[]>>('/investments/top', {
    params: { investment_type: investmentType, limit: 20, user_id: userId ?? requireCurrentUserId() }
  })
  return response.data.data
}

export async function updateInvestment(id: number, payload: UpdateInvestmentPayload) {
  const response = await client.put<ApiResponse<Investment>>(`/investments/${id}`, payload)
  return response.data.data
}
