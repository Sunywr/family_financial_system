ALTER TABLE job_runs
    ADD COLUMN scheduled_at DATETIME NULL COMMENT 'Scheduled execution time' AFTER status,
    ADD COLUMN trigger_type VARCHAR(32) NOT NULL DEFAULT 'scheduler' COMMENT 'scheduler manual bootstrap' AFTER scheduled_at;

CREATE TABLE IF NOT EXISTS dashboard_snapshots (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    user_id BIGINT UNSIGNED NOT NULL COMMENT 'Owner user id',
    snapshot_date DATE NOT NULL COMMENT 'Snapshot date',
    cash_balance DECIMAL(18,2) NOT NULL DEFAULT 0.00 COMMENT 'Cash balance',
    total_assets DECIMAL(18,2) NOT NULL DEFAULT 0.00 COMMENT 'Total assets',
    outstanding_amount DECIMAL(18,2) NOT NULL DEFAULT 0.00 COMMENT 'Outstanding amount',
    wealth_amount DECIMAL(18,2) NOT NULL DEFAULT 0.00 COMMENT 'Wealth amount',
    stock_amount DECIMAL(18,2) NOT NULL DEFAULT 0.00 COMMENT 'Stock amount',
    income DECIMAL(18,2) NOT NULL DEFAULT 0.00 COMMENT 'Income in range',
    expense DECIMAL(18,2) NOT NULL DEFAULT 0.00 COMMENT 'Expense in range',
    net_cash_flow DECIMAL(18,2) NOT NULL DEFAULT 0.00 COMMENT 'Net cash flow in range',
    calibration_status VARCHAR(32) NOT NULL DEFAULT 'uncalibrated' COMMENT 'Calibration status',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    UNIQUE KEY uk_dashboard_snapshots_user_date (user_id, snapshot_date),
    KEY idx_dashboard_snapshots_snapshot_date (snapshot_date)
) COMMENT='Dashboard daily snapshots';
