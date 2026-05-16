#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="${ENV_FILE:-$PROJECT_ROOT/.env}"

load_dotenv() {
  local file="$1"
  if [[ ! -f "$file" ]]; then
    return 0
  fi

  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%%$'\r'}"
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ "$line" =~ ^[[:space:]]*$ ]] && continue

    local key="${line%%=*}"
    local value="${line#*=}"
    key="$(echo "$key" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
    value="$(echo "$value" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"

    if [[ "$value" =~ ^\".*\"$ ]]; then
      value="${value:1:${#value}-2}"
    elif [[ "$value" =~ ^\'.*\'$ ]]; then
      value="${value:1:${#value}-2}"
    fi

    if [[ -n "$key" ]]; then
      export "$key=$value"
    fi
  done < "$file"
}

load_dotenv "$ENV_FILE"

DB_HOST="${1:-${DB_HOST:-${FFS_DB_HOST:-}}}"
DB_PORT="${2:-${DB_PORT:-${FFS_DB_PORT:-}}}"
DB_NAME="${3:-${DB_NAME:-${FFS_DB_NAME:-}}}"
DB_USER="${4:-${DB_USER:-${FFS_DB_USER:-}}}"
DB_PASSWORD="${5:-${DB_PASSWORD:-${FFS_DB_PASSWORD:-${MYSQL_PWD:-}}}}"
MIGRATIONS_DIR="${6:-${MIGRATIONS_DIR:-$PROJECT_ROOT/migrations}}"

if [[ -z "$DB_HOST" || -z "$DB_PORT" || -z "$DB_NAME" || -z "$DB_USER" ]]; then
  echo "usage: $0 [db_host] [db_port] [db_name] [db_user] [db_password] [migrations_dir]"
  echo "error: missing database config. Set args or .env vars: DB_HOST/DB_PORT/DB_NAME/DB_USER"
  echo "hint: also supports FFS_DB_HOST/FFS_DB_PORT/FFS_DB_NAME/FFS_DB_USER"
  exit 1
fi

if [[ ! "$DB_PORT" =~ ^[0-9]+$ ]]; then
  echo "error: DB_PORT must be numeric, got '$DB_PORT'"
  exit 1
fi

if [[ ! -d "$MIGRATIONS_DIR" ]]; then
  echo "error: migrations directory not found: $MIGRATIONS_DIR"
  exit 1
fi

mapfile -t MIGRATION_FILES < <(find "$MIGRATIONS_DIR" -maxdepth 1 -type f -name '*.sql' | sort)

if [[ ${#MIGRATION_FILES[@]} -eq 0 ]]; then
  echo "no .sql files found in $MIGRATIONS_DIR"
  exit 0
fi

if [[ -n "$DB_PASSWORD" ]]; then
  export MYSQL_PWD="$DB_PASSWORD"
else
  unset MYSQL_PWD || true
fi

echo "starting migrations to $DB_NAME@$DB_HOST:$DB_PORT"
for file in "${MIGRATION_FILES[@]}"; do
  echo "running: $file"
  mysql --host="$DB_HOST" --port="$DB_PORT" --user="$DB_USER" "$DB_NAME" < "$file"
done

echo "all migrations completed successfully"