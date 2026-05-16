import { client, type ApiResponse } from './client'

export interface DashboardPendingItem {
  id?: number
  name: string
  type: string
  due_date: string
  amount: string
}

export interface DashboardSalaryPrepSummary {
  window_start: string
  window_end: string
  salary_day: number
  salary_target_date: string
  days_until_salary: number
  pending_count: number
  credit_count: number
  cycle_count: number
  credit_due_before_salary: string
  cycle_due_before_salary: string
  due_before_salary: string
  credit_due_in_window: string
  cycle_due_in_window: string
  source: string
}

export interface DashboardPositionSummary {
  personal_wealth: string
  personal_stock: string
  total_investment: string
  holding_profit: string
  total_profit: string
  avg_profit_rate: string
  avg_annual_rate_wealth: string
  family_wealth: string
  family_stock: string
  stock_idle_cash: string
  source: string
}

export interface DashboardRepayTrendItem {
  date: string
  credit_due: string
  cycle_due: string
  credit_names: string
  cycle_names: string
  total_due: string
  before_salary: boolean
}

export interface DashboardRepayTrendSummary {
  source: string
  window_start: string
  window_end: string
  salary_target_date: string
  items: DashboardRepayTrendItem[]
  before_salary_total: string
  window_total: string
}

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
  salary_prep: DashboardSalaryPrepSummary
  position_summary: DashboardPositionSummary
  repay_trend: DashboardRepayTrendSummary
  credit_cards: DashboardPendingItem[]
  cycle_debts: DashboardPendingItem[]
  pending_all: DashboardPendingItem[]
}

export interface CashTrendPoint {
  date: string
  cash_balance: string
  anchor_date?: string | null
  mark_points?: CashTrendMarkPoint[]
}

export interface CashTrendMarkPoint {
  debt_id: number
  debt_name: string
  debt_type: string
  due_date: string
  amount: string
  balance_after: string
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
