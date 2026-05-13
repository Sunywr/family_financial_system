CREATE TABLE IF NOT EXISTS debts (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    user_id BIGINT UNSIGNED NOT NULL COMMENT 'Owner user id',
    source_bill_id BIGINT UNSIGNED NULL COMMENT 'Source bill id',
    start_date DATE NOT NULL COMMENT 'Debt start date',
    end_date DATE NULL COMMENT 'Debt end date',
    repay_deadline DATE NULL COMMENT 'Repayment deadline',
    category_id BIGINT UNSIGNED NOT NULL COMMENT 'Debt category config id',
    category_name VARCHAR(128) NOT NULL COMMENT 'Cached debt category name',
    amount DECIMAL(18,2) NOT NULL COMMENT 'Debt amount',
    period_count INT UNSIGNED NOT NULL DEFAULT 1 COMMENT 'Number of periods',
    period_unit VARCHAR(16) NOT NULL DEFAULT 'month' COMMENT 'day month year',
    period_value INT UNSIGNED NOT NULL DEFAULT 1 COMMENT 'Period value',
    payment_method VARCHAR(32) NOT NULL COMMENT 'cash or credit_card',
    status VARCHAR(32) NOT NULL DEFAULT 'pending' COMMENT 'pending settled cancelled',
    remark VARCHAR(255) NULL COMMENT 'Remark',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    KEY idx_debts_user_start (user_id, start_date),
    KEY idx_debts_source_bill (source_bill_id)
) COMMENT='Debt records';

CREATE TABLE IF NOT EXISTS presales (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    user_id BIGINT UNSIGNED NOT NULL COMMENT 'Owner user id',
    source_bill_id BIGINT UNSIGNED NULL COMMENT 'Source bill id',
    deposit_date DATE NOT NULL COMMENT 'Deposit date',
    final_payment_date DATE NULL COMMENT 'Final payment date',
    category_id BIGINT UNSIGNED NOT NULL COMMENT 'Presale category config id',
    category_name VARCHAR(128) NOT NULL COMMENT 'Cached presale category name',
    deposit_amount DECIMAL(18,2) NOT NULL COMMENT 'Deposit amount',
    final_payment_amount DECIMAL(18,2) NOT NULL DEFAULT 0.00 COMMENT 'Final payment amount',
    status VARCHAR(32) NOT NULL DEFAULT 'pending_final_payment' COMMENT 'pending_final_payment paid_final_payment cancelled',
    remark VARCHAR(255) NULL COMMENT 'Remark',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    KEY idx_presales_user_deposit (user_id, deposit_date),
    KEY idx_presales_source_bill (source_bill_id)
) COMMENT='Presale records';

CREATE TABLE IF NOT EXISTS assets (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    user_id BIGINT UNSIGNED NOT NULL COMMENT 'Owner user id',
    source_bill_id BIGINT UNSIGNED NULL COMMENT 'Source bill id',
    name VARCHAR(128) NOT NULL COMMENT 'Asset name',
    category_id BIGINT UNSIGNED NOT NULL COMMENT 'Asset category config id',
    category_name VARCHAR(128) NOT NULL COMMENT 'Cached asset category name',
    amount DECIMAL(18,2) NOT NULL COMMENT 'Asset amount',
    remark VARCHAR(255) NULL COMMENT 'Remark',
    status VARCHAR(32) NOT NULL DEFAULT 'active' COMMENT 'active disposed',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    KEY idx_assets_user_created (user_id, created_at),
    KEY idx_assets_source_bill (source_bill_id)
) COMMENT='Fixed assets';
