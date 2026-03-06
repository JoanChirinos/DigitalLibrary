#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Deploying DigitalLibrary..."
echo ""

# Pull latest code
echo "Pulling latest code..."
cd "$PROJECT_ROOT"
git pull

# Build frontend
echo "Building frontend..."
cd "$PROJECT_ROOT/frontend"
npm install
npm run build

# Build backend
echo "Building backend (release mode)..."
cd "$PROJECT_ROOT/backend"
cargo build --release

# Restart backend
echo "Restarting backend..."
launchctl kickstart -k "gui/$(id -u)/com.digitallibrary.backend"

# Wait for startup
sleep 2

# Check if it's running
if launchctl list | grep -q "com.digitallibrary.backend"; then
    echo "Backend restarted successfully"
    echo ""
    echo "Service status:"
    launchctl list | grep digitallibrary
    echo ""
    echo "Recent logs:"
    tail -5 "$PROJECT_ROOT/backend/stdout.log"
else
    echo "Backend failed to start"
    echo "Check logs: tail $PROJECT_ROOT/backend/stderr.log"
    exit 1
fi

echo ""
echo "Deployment complete!"
echo "Site: https://library.joanchirinos.com"
