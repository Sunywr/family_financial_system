CREATE TABLE IF NOT EXISTS users (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    username VARCHAR(64) NOT NULL COMMENT 'Login username',
    display_name VARCHAR(64) NOT NULL COMMENT 'Display name',
    password_hash VARCHAR(255) NOT NULL COMMENT 'Password hash',
    role VARCHAR(32) NOT NULL DEFAULT 'member' COMMENT 'User role',
    enabled TINYINT(1) NOT NULL DEFAULT 1 COMMENT 'Whether the user is enabled',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    UNIQUE KEY uk_users_username (username)
) COMMENT='System users';

CREATE TABLE IF NOT EXISTS config_items (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    config_type VARCHAR(64) NOT NULL COMMENT 'Config item type',
    name VARCHAR(64) NOT NULL COMMENT 'Stable item code',
    display_name VARCHAR(128) NOT NULL COMMENT 'Display name',
    is_builtin TINYINT(1) NOT NULL DEFAULT 0 COMMENT 'Whether item is built in',
    enabled TINYINT(1) NOT NULL DEFAULT 1 COMMENT 'Whether item is enabled',
    sort_order INT NOT NULL DEFAULT 0 COMMENT 'Sort order',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    UNIQUE KEY uk_config_items_type_name (config_type, name),
    KEY idx_config_items_type_sort (config_type, sort_order)
) COMMENT='System config items';

CREATE TABLE IF NOT EXISTS credit_cards (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    user_id BIGINT UNSIGNED NOT NULL COMMENT 'Owner user id',
    name VARCHAR(64) NOT NULL COMMENT 'Credit card name',
    billing_day TINYINT UNSIGNED NOT NULL COMMENT 'Billing day in month',
    repayment_day TINYINT UNSIGNED NOT NULL COMMENT 'Repayment day in month',
    credit_limit DECIMAL(18,2) NOT NULL DEFAULT 0.00 COMMENT 'Credit limit',
    enabled TINYINT(1) NOT NULL DEFAULT 1 COMMENT 'Whether card is enabled',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    KEY idx_credit_cards_user_enabled (user_id, enabled)
) COMMENT='Credit card configurations';
