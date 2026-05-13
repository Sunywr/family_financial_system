import { client, type ApiResponse } from './client'

export interface MigrationAuditModule {
  label: string
  source_table: string
  target_table: string
  source_count: number
  target_count: number
  difference: number
  status: string
  note?: string | null
}

export interface MigrationAuditUser {
  id: number
  username: string
  display_name: string
  role: string
  enabled: boolean
  permission_snapshot_imported: boolean
}

export interface MigrationAuditSummary {
  legacy_database_configured: boolean
  legacy_database_reachable: boolean
  generated_at: string
  totals: {
    source_rows: number
    target_rows: number
    matched_modules: number
    total_modules: number
    permission_snapshots: number
  }
  modules: MigrationAuditModule[]
  users: MigrationAuditUser[]
  remarks: string[]
}

export async function fetchMigrationAuditSummary() {
  const response = await client.get<ApiResponse<MigrationAuditSummary>>('/migration-audit/summary')
  return response.data.data
}
