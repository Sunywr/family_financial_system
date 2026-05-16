ALTER TABLE bills
    ADD COLUMN related_debt_id BIGINT UNSIGNED NULL COMMENT 'Related debt id' AFTER related_asset_id,
    ADD KEY idx_bills_related_debt (related_debt_id);
