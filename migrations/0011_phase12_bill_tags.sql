CREATE TABLE IF NOT EXISTS bill_tags (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    user_id BIGINT UNSIGNED NOT NULL COMMENT 'Owner user id',
    name VARCHAR(64) NOT NULL COMMENT 'Tag name',
    color VARCHAR(32) NULL COMMENT 'Tag color',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    UNIQUE KEY uk_bill_tags_user_name (user_id, name),
    KEY idx_bill_tags_user (user_id)
) COMMENT='Bill tags catalog';
