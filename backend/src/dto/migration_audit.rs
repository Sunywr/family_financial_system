use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MigrationAuditSummaryDto {
    pub legacy_database_configured: bool,
    pub legacy_database_reachable: bool,
    pub generated_at: DateTime<Utc>,
    pub totals: MigrationAuditTotalsDto,
    pub modules: Vec<MigrationAuditModuleDto>,
    pub users: Vec<MigrationAuditUserDto>,
    pub remarks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MigrationAuditTotalsDto {
    pub source_rows: u64,
    pub target_rows: u64,
    pub matched_modules: u64,
    pub total_modules: u64,
    pub permission_snapshots: u64,
}

#[derive(Debug, Serialize)]
pub struct MigrationAuditModuleDto {
    pub label: String,
    pub source_table: String,
    pub target_table: String,
    pub source_count: u64,
    pub target_count: u64,
    pub difference: i64,
    pub status: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MigrationAuditUserDto {
    pub id: u64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub enabled: bool,
    pub permission_snapshot_imported: bool,
}
