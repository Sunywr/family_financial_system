-- One-off cleanup for legacy non-credit-card auto-generated cycle-debt bills.
-- Scope: bills.special_status = 'debt_cycle_auto' and related debt is non-credit-card (or missing).
-- Safety: run preview queries first, then execute UPDATE in a transaction.

-- 1) Preview impacted rows (count)
SELECT COUNT(*) AS impacted_rows
FROM bills b
LEFT JOIN debts d
       ON d.id = b.related_debt_id
      AND d.deleted_at IS NULL
WHERE b.related_debt_id IS NOT NULL
  AND b.special_status = 'debt_cycle_auto'
  AND b.deleted_at IS NULL
  AND (d.id IS NULL OR d.payment_method <> 'credit_card');

-- 2) Preview sample rows
SELECT b.id,
       b.user_id,
       b.account_date,
       b.category_name,
       b.amount,
       b.related_debt_id,
       b.special_status,
       d.payment_method AS debt_payment_method,
       d.status AS debt_status
FROM bills b
LEFT JOIN debts d
       ON d.id = b.related_debt_id
      AND d.deleted_at IS NULL
WHERE b.related_debt_id IS NOT NULL
  AND b.special_status = 'debt_cycle_auto'
  AND b.deleted_at IS NULL
  AND (d.id IS NULL OR d.payment_method <> 'credit_card')
ORDER BY b.account_date ASC, b.id ASC
LIMIT 200;

-- 3) Execute cleanup (soft delete)
START TRANSACTION;

UPDATE bills b
LEFT JOIN debts d
       ON d.id = b.related_debt_id
      AND d.deleted_at IS NULL
SET b.deleted_at = CURRENT_TIMESTAMP,
    b.updated_at = CURRENT_TIMESTAMP
WHERE b.related_debt_id IS NOT NULL
  AND b.special_status = 'debt_cycle_auto'
  AND b.deleted_at IS NULL
  AND (d.id IS NULL OR d.payment_method <> 'credit_card');

SELECT ROW_COUNT() AS updated_rows;

COMMIT;

-- 4) Post-check (should be 0)
SELECT COUNT(*) AS remaining_rows
FROM bills b
LEFT JOIN debts d
       ON d.id = b.related_debt_id
      AND d.deleted_at IS NULL
WHERE b.related_debt_id IS NOT NULL
  AND b.special_status = 'debt_cycle_auto'
  AND b.deleted_at IS NULL
  AND (d.id IS NULL OR d.payment_method <> 'credit_card');
