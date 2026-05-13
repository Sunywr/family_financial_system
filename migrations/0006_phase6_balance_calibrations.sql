CREATE TABLE IF NOT EXISTS balance_calibrations (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    user_id BIGINT UNSIGNED NOT NULL COMMENT 'Owner user id',
    calibration_date DATE NOT NULL COMMENT 'Calibration date',
    cash_balance DECIMAL(18,2) NOT NULL COMMENT 'Cash balance at calibration date',
    remark VARCHAR(255) NULL COMMENT 'Remark',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    KEY idx_balance_calibrations_user_date (user_id, calibration_date)
) COMMENT='Balance calibration records';
