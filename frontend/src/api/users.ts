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

export async function fetchUsers() {
  const response = await client.get<ApiResponse<PageData<User>>>('/users', {
    params: { page: 1, page_size: 20 }
  })
  return response.data.data
}
