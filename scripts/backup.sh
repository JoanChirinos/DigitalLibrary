#!/usr/bin/env bash
set -euo pipefail

# Back up the DigitalLibrary SQLite DB: WAL-safe, integrity-checked, gzipped,
# 30-backup retention. Optional label arg (default "daily"); deploy.sh passes
# "predeploy" to mark pre-deploy rollback snapshots.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

DB_PATH="$PROJECT_ROOT/backend/library.db"
BACKUP_DIR="$HOME/DigitalLibrary-backups"
RETENTION=30
LABEL="${1:-daily}"
STAMP="$(date +%Y%m%d_%H%M%S)"
DEST="$BACKUP_DIR/library_${STAMP}_${LABEL}.db"

mkdir -p "$BACKUP_DIR"

if [ ! -f "$DB_PATH" ]; then
  echo "$(date): ERROR - Database not found at $DB_PATH" >> "$BACKUP_DIR/backup.log"
  exit 1
fi

# Online backup (safe with WAL / concurrent writes)
/usr/bin/sqlite3 "$DB_PATH" ".backup '$DEST'"

# Verify the copy before trusting it
if ! /usr/bin/sqlite3 "$DEST" "PRAGMA integrity_check;" | grep -q "^ok$"; then
  echo "$(date): ERROR - integrity check FAILED for $DEST" >> "$BACKUP_DIR/backup.log"
  rm -f "$DEST"
  exit 1
fi

gzip -f "$DEST"
echo "$(date): backup OK -> ${DEST}.gz" >> "$BACKUP_DIR/backup.log"

# Prune: keep newest $RETENTION
ls -1t "$BACKUP_DIR"/library_*.db.gz 2>/dev/null | tail -n +$((RETENTION + 1)) | while read -r old; do
  rm -f "$old"
  echo "$(date): pruned $old" >> "$BACKUP_DIR/backup.log"
done
