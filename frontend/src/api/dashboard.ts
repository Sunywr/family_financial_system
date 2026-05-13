import { client, type ApiResponse } from './client'

export interface DashboardSummary {
  start_date: string
  end_date: string
  cash_balance: string
  total_assets: string
  outstanding_amount: string
  wealth_amount: string
  stock_amount: string
  family_wealth_amount: string
  family_stock_amount: string
  stock_idle_cash: string
  credit_card_outstanding_amount: string
  debt_outstanding_amount: string
  income: string
  expense: string
  net_cash_flow: string
  calibration_status: string
  calibration_date?: string | null
  salary_prep: Record<string, unknown>
  position_summary: Record<string, unknown>
  repay_trend: Record<string, unknown>
  credit_cards: Record<string, unknown>[]
  cycle_debts: Record<string, unknown>[]
  pending_all: Record<string, unknown>[]
}

export interface CashTrendPoint {
  date: string
  cash_balance: string
  anchor_date?: string | null
}

export async function fetchDashboardSummary(params: {
  user_id: number
  start_date: string
  end_date: string
}) {
  const response = await client.get<ApiResponse<DashboardSummary>>('/dashboard/summary', { params })
  return response.data.data
}

export async function fetchCashTrend(params: {
  user_id: number
  start_date: string
  end_date: string
}) {
  const response = await client.get<ApiResponse<CashTrendPoint[]>>('/dashboard/cash-trend', {
    params
  })
  return response.data.data
}
