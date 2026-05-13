#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 5 ]]; then
  echo "usage: $0 <backup_file.sql.gz> <db_host> <db_port> <db_name> <db_user> [db_password]"
  exit 1
fi

BACKUP_FILE="$1"
DB_HOST="$2"
DB_PORT="$3"
DB_NAME="$4"
DB_USER="$5"
DB_PASSWORD="${6:-${MYSQL_PWD:-}}"

export MYSQL_PWD="$DB_PASSWORD"
gunzip -c "$BACKUP_FILE" | mysql --host="$DB_HOST" --port="$DB_PORT" --user="$DB_USER" "$DB_NAME"
echo "restore finished: $BACKUP_FILE -> $DB_NAME"
