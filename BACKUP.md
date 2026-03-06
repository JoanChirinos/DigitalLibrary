# Backup Setup

## Automatic Daily Backups

The backup script runs daily at 2:00 AM and keeps the last 30 backups.

### Option A: launchd (macOS, recommended)

Run the install script:

```bash
./scripts/install.sh
```

This generates the plist with correct paths and loads it.

**Verify:**
```bash
launchctl list | grep digitallibrary.backup
```

### Option B: cron (macOS/Linux)

1. Edit your crontab:
```bash
crontab -e
```

2. Add this line (runs daily at 2am):
```
0 2 * * * /Users/joanchir/Desktop/DigitalLibrary/scripts/backup.sh >> /Users/joanchir/DigitalLibrary-backups/cron.log 2>&1
```

3. Save and exit. Verify with:
```bash
crontab -l
```

**Note:** On macOS, cron doesn't run if the Mac is asleep. launchd will catch up when it wakes.

### Manual Backup

Run the script anytime:
```bash
./scripts/backup.sh
```

### Backup Location

Backups are stored in `~/DigitalLibrary-backups/` with filenames like `library_20260305_140000.db`.

Logs are in:
- `~/DigitalLibrary-backups/backup.log` — backup history
- `~/DigitalLibrary-backups/launchd.log` — launchd stdout
- `~/DigitalLibrary-backups/launchd.err` — launchd errors

### Restore from Backup

```bash
# Stop the backend first
cp ~/DigitalLibrary-backups/library_YYYYMMDD_HHMMSS.db ~/Desktop/DigitalLibrary/backend/library.db
# Restart the backend
```

### Uninstall

**launchd:**
```bash
launchctl unload ~/Library/LaunchAgents/com.digitallibrary.backup.plist
rm ~/Library/LaunchAgents/com.digitallibrary.backup.plist
```

**cron:**
```bash
crontab -e
# Remove the DigitalLibrary backup line, save and exit
```
