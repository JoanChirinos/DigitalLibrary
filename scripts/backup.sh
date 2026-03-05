#!/bin/bash
set -e

# Configuration
BACKUP_DIR="$HOME/DigitalLibrary-backups"
DB_PATH="$HOME/Desktop/DigitalLibrary/backend/library.db"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
KEEP_BACKUPS=30

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Backup using SQLite's .backup command (safe even while DB is in use)
if [ -f "$DB_PATH" ]; then
    sqlite3 "$DB_PATH" ".backup '$BACKUP_DIR/library_$TIMESTAMP.db'"
    echo "$(date): Backup created: library_$TIMESTAMP.db" >> "$BACKUP_DIR/backup.log"
    
    # Keep only the last N backups
    ls -t "$BACKUP_DIR"/library_*.db | tail -n +$((KEEP_BACKUPS + 1)) | xargs rm -f 2>/dev/null || true
else
    echo "$(date): ERROR - Database not found at $DB_PATH" >> "$BACKUP_DIR/backup.log"
    exit 1
fi
