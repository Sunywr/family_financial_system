import { client, type ApiResponse } from './client'

export interface HealthResponse {
  service: string
  version: string
  environment: string
  database_configured: boolean
  database_reachable: boolean
}

export async function fetchHealth() {
  const response = await client.get<ApiResponse<HealthResponse>>('/health')
  return response.data.data
}
