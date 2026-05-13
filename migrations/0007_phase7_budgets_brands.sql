CREATE TABLE IF NOT EXISTS budgets (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    user_id BIGINT UNSIGNED NOT NULL COMMENT 'Owner user id',
    budget_month DATE NOT NULL COMMENT 'Budget month first day',
    category_id BIGINT UNSIGNED NOT NULL COMMENT 'Budget category config id',
    category_name VARCHAR(128) NOT NULL COMMENT 'Cached budget category name',
    planned_amount DECIMAL(18,2) NOT NULL DEFAULT 0.00 COMMENT 'Planned amount',
    manual_adjusted TINYINT(1) NOT NULL DEFAULT 0 COMMENT 'Whether manually adjusted',
    remark VARCHAR(255) NULL COMMENT 'Remark',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    UNIQUE KEY uk_budgets_user_month_category (user_id, budget_month, category_id),
    KEY idx_budgets_user_month (user_id, budget_month)
) COMMENT='Monthly budgets';

CREATE TABLE IF NOT EXISTS brands (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    user_id BIGINT UNSIGNED NOT NULL COMMENT 'Owner user id',
    category_id BIGINT UNSIGNED NOT NULL COMMENT 'Brand category config id',
    category_name VARCHAR(128) NOT NULL COMMENT 'Cached category name',
    brand_name VARCHAR(128) NOT NULL COMMENT 'Brand name',
    score INT NOT NULL DEFAULT 0 COMMENT 'Brand score',
    board_type VARCHAR(16) NOT NULL DEFAULT 'normal' COMMENT 'red black normal',
    review TEXT NULL COMMENT 'Review content',
    remark VARCHAR(255) NULL COMMENT 'Remark',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    KEY idx_brands_user_category (user_id, category_id),
    KEY idx_brands_user_board_type (user_id, board_type)
) COMMENT='Brand red and black list';
