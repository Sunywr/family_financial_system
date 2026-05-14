import { client, type ApiResponse } from './client'
import type { PageData } from '@/types/api'

export interface JobConfig {
  id: number
  job_code: string
  job_name: string
  cron_expr: string
  enabled: boolean
  batch_size: number
  concurrency: number
  timeout_seconds: number
  retry_count: number
}

export interface UpdateJobRequest {
  cron_expr: string
  enabled: boolean
  batch_size: number
  concurrency: number
  timeout_seconds: number
  retry_count: number
}

export interface JobRun {
  id: number
  job_id: number
  status: string
  scheduled_at?: string | null
  trigger_type: string
  started_at: string
  finished_at?: string | null
  duration_ms?: number | null
  message?: string | null
  error_message?: string | null
}

export async function fetchJobs(page = 1, pageSize = 20) {
  const response = await client.get<ApiResponse<PageData<JobConfig>>>('/jobs', {
    params: { page, page_size: pageSize }
  })
  return response.data.data
}

export async function fetchJobRuns(page = 1, pageSize = 20) {
  const response = await client.get<ApiResponse<PageData<JobRun>>>('/jobs/runs', {
    params: { page, page_size: pageSize }
  })
  return response.data.data
}

export async function triggerJob(id: number) {
  const response = await client.post<ApiResponse<JobRun>>(`/jobs/${id}/trigger`, {
    trigger_type: 'manual'
  })
  return response.data.data
}

export async function updateJob(id: number, payload: UpdateJobRequest) {
  const response = await client.put<ApiResponse<JobConfig>>(`/jobs/${id}`, payload)
  return response.data.data
}
