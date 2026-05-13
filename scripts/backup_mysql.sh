#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 5 ]]; then
  echo "usage: $0 <db_host> <db_port> <db_name> <db_user> <backup_dir> [db_password]"
  exit 1
fi

DB_HOST="$1"
DB_PORT="$2"
DB_NAME="$3"
DB_USER="$4"
BACKUP_DIR="$5"
DB_PASSWORD="${6:-${MYSQL_PWD:-}}"

TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BACKUP_DIR"

export MYSQL_PWD="$DB_PASSWORD"
OUTPUT_FILE="$BACKUP_DIR/${DB_NAME}-${TIMESTAMP}.sql.gz"

mysqldump \
  --single-transaction \
  --quick \
  --routines \
  --events \
  --host="$DB_HOST" \
  --port="$DB_PORT" \
  --user="$DB_USER" \
  "$DB_NAME" | gzip > "$OUTPUT_FILE"

find "$BACKUP_DIR" -type f -name "${DB_NAME}-*.sql.gz" -mtime +14 -delete
echo "backup created: $OUTPUT_FILE"
