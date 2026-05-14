import { client, type ApiResponse } from './client'
import type { PageData } from '@/types/api'

export interface User {
  id: number
  username: string
  display_name: string
  role: string
  enabled: boolean
  created_at: string
  updated_at: string
}

export async function fetchUsers(page = 1, pageSize = 20) {
  const response = await client.get<ApiResponse<PageData<User>>>('/users', {
    params: { page, page_size: pageSize }
  })
  return response.data.data
}
