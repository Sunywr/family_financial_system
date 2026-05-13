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
  product_code?: string
  product_name?: string
  organization_name?: string
  share_amount?: string
}

export async function fetchBills(keyword = '') {
  const response = await client.get<ApiResponse<PageData<Bill>>>('/bills', {
    params: withCurrentUserId({ page: 1, page_size: 20, keyword: keyword || undefined })
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
