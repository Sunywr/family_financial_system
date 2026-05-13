use chrono::Utc;

use crate::{
    common::state::AppState,
    dto::migration_audit::{
        MigrationAuditModuleDto, MigrationAuditSummaryDto, MigrationAuditTotalsDto,
        MigrationAuditUserDto,
    },
    error::app_error::AppError,
    repository::migration_audit_repository,
};

struct TablePair {
    label: &'static str,
    source_table: &'static str,
    target_table: &'static str,
    note: Option<&'static str>,
}

const TABLE_PAIRS: [TablePair; 10] = [
    TablePair {
        label: "用户",
        source_table: "auth_user",
        target_table: "users",
        note: None,
    },
    TablePair {
        label: "信用卡",
        source_table: "account_creditcard",
        target_table: "credit_cards",
        note: None,
    },
    TablePair {
        label: "账单",
        source_table: "account_account",
        target_table: "bills",
        note: None,
    },
    TablePair {
        label: "债务",
        source_table: "account_debt",
        target_table: "debts",
        note: None,
    },
    TablePair {
        label: "预售",
        source_table: "account_presale",
        target_table: "presales",
        note: None,
    },
    TablePair {
        label: "投资",
        source_table: "account_invest",
        target_table: "investments",
        note: None,
    },
    TablePair {
        label: "资产",
        source_table: "account_asset",
        target_table: "assets",
        note: None,
    },
    TablePair {
        label: "品牌",
        source_table: "account_brand",
        target_table: "brands",
        note: None,
    },
    TablePair {
        label: "余额校准",
        source_table: "account_balancefix",
        target_table: "balance_calibrations",
        note: None,
    },
    TablePair {
        label: "预算",
        source_table: "account_budget",
        target_table: "budgets",
        note: Some("旧库允许同用户同月同分类多条，新库按唯一键合并导入。"),
    },
];

pub async fn build_summary(state: &AppState) -> Result<MigrationAuditSummaryDto, AppError> {
    let target_db = state.db()?;
    let legacy_database_configured = state.legacy_db.is_some();
    let permission_snapshots =
        migration_audit_repository::count_permission_snapshots(target_db).await?;
    let permission_snapshot_user_ids =
        migration_audit_repository::list_permission_snapshot_user_ids(target_db).await?;
    let users = migration_audit_repository::list_users(target_db).await?;

    let mut user_rows = Vec::with_capacity(users.len());
    for user in users {
        user_rows.push(MigrationAuditUserDto {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            role: user.role,
            enabled: user.enabled,
            permission_snapshot_imported: permission_snapshot_user_ids.contains(&user.id),
        });
    }

    if !legacy_database_configured {
        return Ok(MigrationAuditSummaryDto {
            legacy_database_configured,
            legacy_database_reachable: false,
            generated_at: Utc::now(),
            totals: MigrationAuditTotalsDto {
                source_rows: 0,
                target_rows: 0,
                matched_modules: 0,
                total_modules: TABLE_PAIRS.len() as u64,
                permission_snapshots,
            },
            modules: Vec::new(),
            users: user_rows,
            remarks: vec![
                "未配置 legacy_database，当前只能查看导入后的用户和权限快照。".to_string(),
            ],
        });
    }

    let legacy_db = state.legacy_db()?;
    let legacy_database_reachable: bool =
        migration_audit_repository::ping_legacy_database(legacy_db)
            .await
            .unwrap_or_default();

    if !legacy_database_reachable {
        return Ok(MigrationAuditSummaryDto {
            legacy_database_configured,
            legacy_database_reachable,
            generated_at: Utc::now(),
            totals: MigrationAuditTotalsDto {
                source_rows: 0,
                target_rows: 0,
                matched_modules: 0,
                total_modules: TABLE_PAIRS.len() as u64,
                permission_snapshots,
            },
            modules: Vec::new(),
            users: user_rows,
            remarks: vec![
                "legacy_database 已配置，但当前无法连接旧库 pfm。".to_string(),
                "请检查旧库账号、密码、端口和数据库服务状态。".to_string(),
            ],
        });
    }

    let mut modules = Vec::with_capacity(TABLE_PAIRS.len());
    let mut source_rows = 0_u64;
    let mut target_rows = 0_u64;
    let mut matched_modules = 0_u64;

    for pair in TABLE_PAIRS {
        let source_count =
            migration_audit_repository::count_rows(legacy_db, pair.source_table).await?;
        let target_count =
            migration_audit_repository::count_rows(target_db, pair.target_table).await?;
        let difference = target_count as i64 - source_count as i64;
        let status = if difference == 0 {
            matched_modules += 1;
            "matched"
        } else if pair.target_table == "budgets" && target_count < source_count {
            "merged"
        } else {
            "warning"
        };

        source_rows += source_count;
        target_rows += target_count;

        modules.push(MigrationAuditModuleDto {
            label: pair.label.to_string(),
            source_table: pair.source_table.to_string(),
            target_table: pair.target_table.to_string(),
            source_count,
            target_count,
            difference,
            status: status.to_string(),
            note: pair.note.map(ToString::to_string),
        });
    }

    let remarks = vec![
        "权限在 HFS 中按角色映射，旧 Django 细粒度权限已作为快照归档到 intel_items。".to_string(),
        "若重新执行导入脚本，报表会直接对比 legacy_database 与 ffs 当前数据。".to_string(),
    ];

    Ok(MigrationAuditSummaryDto {
        legacy_database_configured,
        legacy_database_reachable,
        generated_at: Utc::now(),
        totals: MigrationAuditTotalsDto {
            source_rows,
            target_rows,
            matched_modules,
            total_modules: TABLE_PAIRS.len() as u64,
            permission_snapshots,
        },
        modules,
        users: user_rows,
        remarks,
    })
}
