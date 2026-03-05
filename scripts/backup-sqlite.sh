#!/usr/bin/env bash
# backup-sqlite.sh — Create a timestamped hot backup of the SQLite database.
#
# Usage:
#   ./scripts/backup-sqlite.sh [database_path] [backup_dir]
#
# Defaults:
#   database_path = /data/app.db  (Fly.io mount)
#   backup_dir    = /data/backups
#
# The script uses SQLite's VACUUM INTO to create an atomic, consistent
# snapshot without interrupting active readers/writers.

set -euo pipefail

DB_PATH="${1:-/data/app.db}"
BACKUP_DIR="${2:-/data/backups}"
TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
BACKUP_FILE="${BACKUP_DIR}/app-${TIMESTAMP}.db"
MAX_BACKUPS="${MAX_BACKUPS:-7}"

if [ ! -f "$DB_PATH" ]; then
  echo "ERROR: database not found at ${DB_PATH}" >&2
  exit 1
fi

mkdir -p "$BACKUP_DIR"

echo "Backing up ${DB_PATH} → ${BACKUP_FILE} ..."
sqlite3 "$DB_PATH" "VACUUM INTO '${BACKUP_FILE}';"

# Prune old backups, keeping the newest MAX_BACKUPS files.
BACKUP_COUNT=$(find "$BACKUP_DIR" -maxdepth 1 -name 'app-*.db' | wc -l)
if [ "$BACKUP_COUNT" -gt "$MAX_BACKUPS" ]; then
  REMOVE_COUNT=$((BACKUP_COUNT - MAX_BACKUPS))
  find "$BACKUP_DIR" -maxdepth 1 -name 'app-*.db' -print0 \
    | sort -z \
    | head -z -n "$REMOVE_COUNT" \
    | xargs -0 rm -f
  echo "Pruned ${REMOVE_COUNT} old backup(s), keeping newest ${MAX_BACKUPS}."
fi

echo "Backup complete: ${BACKUP_FILE} ($(du -h "$BACKUP_FILE" | cut -f1))"
