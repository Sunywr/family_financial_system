CREATE TABLE IF NOT EXISTS intel_items (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    user_id BIGINT UNSIGNED NOT NULL COMMENT 'Owner user id',
    title VARCHAR(128) NOT NULL COMMENT 'Intel title',
    source VARCHAR(128) NULL COMMENT 'Intel source',
    item_date DATE NOT NULL COMMENT 'Intel date',
    status VARCHAR(32) NOT NULL DEFAULT 'draft' COMMENT 'draft active archived',
    tags VARCHAR(255) NULL COMMENT 'Comma separated tags',
    summary VARCHAR(255) NULL COMMENT 'Short summary',
    content TEXT NULL COMMENT 'Intel content',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    KEY idx_intel_items_user_date (user_id, item_date),
    KEY idx_intel_items_status (status)
) COMMENT='Intel placeholder items';
