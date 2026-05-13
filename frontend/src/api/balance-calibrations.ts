import { client, type ApiResponse, requireCurrentUserId } from './client'
import type { PageData } from '@/types/api'

export interface BalanceCalibration {
  id: number
  user_id: number
  calibration_date: string
  cash_balance: string
  remark?: string | null
}

export async function fetchBalanceCalibrations(userId?: number) {
  const resolvedUserId = userId ?? requireCurrentUserId()
  const response = await client.get<ApiResponse<PageData<BalanceCalibration>>>(
    '/balance-calibrations',
    { params: { page: 1, page_size: 20, user_id: resolvedUserId } }
  )
  return response.data.data
}
