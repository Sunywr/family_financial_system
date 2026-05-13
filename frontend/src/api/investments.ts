import { client, type ApiResponse, withCurrentUserId } from './client'
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

export async function fetchInvestments(params?: {
  investment_type?: string
  show_sold?: boolean
  keyword?: string
}) {
  const response = await client.get<ApiResponse<PageData<Investment>>>('/investments', {
    params: withCurrentUserId({
      page: 1,
      page_size: 50,
      investment_type: params?.investment_type,
      show_sold: params?.show_sold,
      keyword: params?.keyword
    })
  })
  return response.data.data
}

export async function fetchTopInvestments(investmentType?: string) {
  const response = await client.get<ApiResponse<InvestmentRecommendation[]>>('/investments/top', {
    params: withCurrentUserId({ investment_type: investmentType, limit: 20 })
  })
  return response.data.data
}
