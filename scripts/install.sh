#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Installing DigitalLibrary launchd services..."
echo "Project root: $PROJECT_ROOT"

# Generate backup plist
sed "s|{{PROJECT_ROOT}}|$PROJECT_ROOT|g" "$SCRIPT_DIR/com.digitallibrary.backup.plist.template" > ~/Library/LaunchAgents/com.digitallibrary.backup.plist
echo "✓ Created ~/Library/LaunchAgents/com.digitallibrary.backup.plist"

# Generate backend plist
cat > ~/Library/LaunchAgents/com.digitallibrary.backend.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.digitallibrary.backend</string>
    
    <key>ProgramArguments</key>
    <array>
        <string>$PROJECT_ROOT/backend/target/release/digital-library</string>
    </array>
    
    <key>WorkingDirectory</key>
    <string>$PROJECT_ROOT/backend</string>
    
    <key>RunAtLoad</key>
    <true/>
    
    <key>KeepAlive</key>
    <true/>
    
    <key>StandardOutPath</key>
    <string>$PROJECT_ROOT/backend/stdout.log</string>
    
    <key>StandardErrorPath</key>
    <string>$PROJECT_ROOT/backend/stderr.log</string>
</dict>
</plist>
EOF
echo "✓ Created ~/Library/LaunchAgents/com.digitallibrary.backend.plist"

# Load services
launchctl load ~/Library/LaunchAgents/com.digitallibrary.backup.plist 2>/dev/null || echo "  (backup service already loaded)"
launchctl load ~/Library/LaunchAgents/com.digitallibrary.backend.plist 2>/dev/null || echo "  (backend service already loaded)"

echo ""
echo "✓ Installation complete!"
echo ""
echo "Services installed:"
launchctl list | grep digitallibrary
echo ""
echo "Backup location: ~/DigitalLibrary-backups/"
echo "Backend logs: $PROJECT_ROOT/backend/stdout.log"
