import { client, type ApiResponse, withCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface Bill {
  id: number
  user_id: number
  account_date: string
  category_id: number
  category_name: string
  bill_type: string
  payment_method: string
  is_fixed_asset: boolean
  amount: string
  tags: string[]
  remark?: string | null
  transfer_group_id?: string | null
  transfer_target_type?: string | null
  transfer_target_user_id?: number | null
  transfer_target_user_name?: string | null
  credit_card_id?: number | null
  credit_card_name?: string | null
  is_installment?: boolean
  installment_months?: number | null
  investment_action?: string | null
  related_investment_id?: number | null
  related_investment_name?: string | null
  related_asset_id?: number | null
  related_asset_name?: string | null
  related_debt_id?: number | null
  product_name?: string | null
  product_code?: string | null
  organization_name?: string | null
  special_status: string
}

export interface BillOptions {
  payment_methods: string[]
  normal_bill_types: string[]
  investment_bill_types: string[]
  transfer_target_types: string[]
}

export interface CreateBillPayload {
  user_id: number
  account_date: string
  category_id: number
  bill_type: string
  payment_method: string
  is_fixed_asset: boolean
  amount: string
  tags?: string[]
  remark?: string
  transfer_target_type?: string
  transfer_target_user_id?: number
  credit_card_id?: number
  is_installment?: boolean
  installment_months?: number
  investment_action?: string
  related_investment_id?: number
  product_code?: string
  product_name?: string
  organization_name?: string
  share_amount?: string
  related_asset_id?: number
  related_debt_id?: number
}

export type UpdateBillPayload = Omit<CreateBillPayload, 'user_id'>

export interface BillListFilters {
  category_id?: number
  payment_method?: string
  credit_card_id?: number
  start_date?: string
  end_date?: string
}

export async function fetchBills(keyword = '', page = 1, pageSize = 20, filters: BillListFilters = {}) {
  const response = await client.get<ApiResponse<PageData<Bill>>>('/bills', {
    params: {
      page,
      page_size: pageSize,
      keyword: keyword || undefined,
      category_id: filters.category_id,
      payment_method: filters.payment_method,
      credit_card_id: filters.credit_card_id,
      start_date: filters.start_date,
      end_date: filters.end_date
    }
  })
  return response.data.data
}

export async function fetchBillOptions() {
  const response = await client.get<ApiResponse<BillOptions>>('/bills/options')
  return response.data.data
}

export async function createBill(payload: CreateBillPayload) {
  const response = await client.post<ApiResponse<Bill>>('/bills', payload)
  return response.data.data
}

export async function updateBill(id: number, payload: UpdateBillPayload) {
  const response = await client.put<ApiResponse<Bill>>(`/bills/${id}`, payload)
  return response.data.data
}

export async function deleteBill(id: number) {
  const response = await client.delete<ApiResponse<{ deleted: boolean }>>(`/bills/${id}`)
  return response.data.data
}
