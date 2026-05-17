import { client, type ApiResponse } from './client'
import type { PageData } from '@/types/api'

export interface AutoInvestPlan {
  id: number
  user_id: number
  investment_id: number
  investment_name?: string | null
  name: string
  category_id: number
  category_name: string
  amount: string
  cycle_months: number
  start_date: string
  end_date?: string | null
  last_generated_date?: string | null
  status: string
  remark?: string | null
  created_at: string
  updated_at: string
}

export interface AutoInvestPlanListQuery {
  page?: number
  page_size?: number
  user_id?: number | null
  status?: string | null
}

export interface CreateAutoInvestPlanRequest {
  user_id: number
  investment_id: number
  name: string
  category_id: number
  amount: string
  cycle_months: number
  start_date: string
  end_date?: string | null
  remark?: string | null
}

export interface UpdateAutoInvestPlanRequest {
  name: string
  category_id: number
  amount: string
  cycle_months: number
  start_date: string
  end_date?: string | null
  status: string
  remark?: string | null
}

export async function listAutoInvestPlans(
  params: AutoInvestPlanListQuery,
) {
  return client.get<ApiResponse<PageData<AutoInvestPlan>>>('/auto-invest-plans', { params })
}

export async function getAutoInvestPlan(
  id: number,
) {
  return client.get<ApiResponse<AutoInvestPlan>>(`/auto-invest-plans/${id}`)
}

export async function createAutoInvestPlan(
  data: CreateAutoInvestPlanRequest,
) {
  return client.post<ApiResponse<AutoInvestPlan>>('/auto-invest-plans', data)
}

export async function updateAutoInvestPlan(
  id: number,
  data: UpdateAutoInvestPlanRequest,
) {
  return client.put<ApiResponse<AutoInvestPlan>>(`/auto-invest-plans/${id}`, data)
}

export async function deleteAutoInvestPlan(
  id: number,
) {
  return client.delete<ApiResponse<void>>(`/auto-invest-plans/${id}`)
}
