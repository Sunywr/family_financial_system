-- Phase 1 bootstrap schema for HFS.
-- Business tables will be added in later phases.

CREATE TABLE IF NOT EXISTS job_configs (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    job_code VARCHAR(64) NOT NULL COMMENT 'Unique job code',
    job_name VARCHAR(128) NOT NULL COMMENT 'Display name',
    cron_expr VARCHAR(64) NOT NULL COMMENT 'Cron expression',
    enabled TINYINT(1) NOT NULL DEFAULT 1 COMMENT 'Whether the job is enabled',
    batch_size INT UNSIGNED NOT NULL DEFAULT 100 COMMENT 'Batch size for each run',
    concurrency INT UNSIGNED NOT NULL DEFAULT 1 COMMENT 'Max concurrent workers',
    timeout_seconds INT UNSIGNED NOT NULL DEFAULT 300 COMMENT 'Timeout for one run',
    retry_count INT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'Retry count on failure',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    UNIQUE KEY uk_job_configs_job_code (job_code)
) COMMENT='Scheduler job configuration';

CREATE TABLE IF NOT EXISTS job_runs (
    id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT COMMENT 'Primary key',
    job_id BIGINT UNSIGNED NOT NULL COMMENT 'Job config id',
    status VARCHAR(32) NOT NULL COMMENT 'Run status',
    started_at DATETIME NOT NULL COMMENT 'Run start time',
    finished_at DATETIME NULL COMMENT 'Run end time',
    duration_ms BIGINT UNSIGNED NULL COMMENT 'Duration in milliseconds',
    message VARCHAR(255) NULL COMMENT 'Summary message',
    error_message TEXT NULL COMMENT 'Failure details',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'Created time',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'Updated time',
    deleted_at DATETIME NULL COMMENT 'Soft delete time',
    KEY idx_job_runs_job_started (job_id, started_at)
) COMMENT='Scheduler job run logs';
