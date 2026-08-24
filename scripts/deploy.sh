#!/usr/bin/env bash
set -euo pipefail

# Deploy DigitalLibrary: (pull), snapshot DB, rebuild frontend + backend, restart,
# verify, and auto-rollback to the previous commit's state if the backend is unhealthy.
#
# Migrations run automatically on backend startup (run_pending_migrations().expect()),
# so a deploy can mutate the live DB or panic on a bad migration. We snapshot the DB
# and preserve the current backend binary BEFORE switching. If the new backend fails
# to come up healthy, we roll back:
#   - restore the preserved binary   (fast + deterministic; avoids a slow Rust rebuild during an incident)
#   - git reset --hard to old commit (source matches the restored binary)
#   - rebuild the frontend           (served by the backend; fast, so no need to preserve it)
#   - restore the pre-migration DB snapshot
# ...then restart and exit non-zero so CI shows red.
#
# CI passes ROLLBACK_SHA (captured before it reset the tree) + SKIP_PULL=1. Run
# standalone (no env) for a manual deploy and it fetches/resets origin/main itself.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(dirname "$SCRIPT_DIR")"
NODE_BIN="$HOME/.nvm/versions/node/v24.15.0/bin"
LAUNCHD_LABEL="com.digitallibrary.backend"
BIN="$REPO/backend/target/release/digital-library"
BIN_ROLLBACK="$BIN.rollback"
DB="$REPO/backend/library.db"
HEALTH_URL="http://127.0.0.1:8008/"

export PATH="$NODE_BIN:$PATH"
source "$HOME/.cargo/env"

GUI="gui/$(id -u)/$LAUNCHD_LABEL"

wait_healthy() {
  for _ in $(seq 1 20); do
    if [ "$(curl -s -o /dev/null -w '%{http_code}' "$HEALTH_URL")" = "200" ]; then
      return 0
    fi
    sleep 1
  done
  return 1
}

build_frontend() {
  cd "$REPO/frontend"
  npm install
  npm run build
}

cd "$REPO"
OLD_SHA="${ROLLBACK_SHA:-$(git rev-parse HEAD)}"
echo "==> Rollback target commit: $OLD_SHA"

echo "==> Pre-deploy DB snapshot (rollback point)"
"$REPO/scripts/backup.sh" predeploy
DB_ROLLBACK="$(ls -1t "$HOME/DigitalLibrary-backups"/library_*_predeploy.db.gz | head -1)"
echo "    snapshot: $DB_ROLLBACK"

echo "==> Preserving current backend binary"
[ -f "$BIN" ] && cp "$BIN" "$BIN_ROLLBACK"

if [ "${SKIP_PULL:-0}" != "1" ]; then
  echo "==> Fetching origin/main"
  git fetch origin main
  git reset --hard origin/main
fi

echo "==> Building backend (cargo build --release)"
cd "$REPO/backend"
cargo build --release

echo "==> Building frontend"
build_frontend

echo "==> Restarting backend (launchctl kickstart -k)"
launchctl kickstart -k "$GUI"

echo "==> Waiting for backend to become healthy..."
if wait_healthy; then
  echo "==> Backend healthy."
  rm -f "$BIN_ROLLBACK"
else
  echo "!!> Backend UNHEALTHY after deploy. Rolling back to $OLD_SHA." >&2

  if [ -f "$BIN_ROLLBACK" ]; then
    cp "$BIN_ROLLBACK" "$BIN"
    rm -f "$BIN_ROLLBACK"
    echo "!!> Restored previous backend binary." >&2
  fi

  cd "$REPO"
  git reset --hard "$OLD_SHA"
  echo "!!> Source reset to $OLD_SHA; rebuilding frontend." >&2
  build_frontend

  echo "!!> Restoring DB from $DB_ROLLBACK" >&2
  launchctl kill TERM "$GUI" 2>/dev/null || true
  sleep 2
  gunzip -c "$DB_ROLLBACK" > "$DB"
  rm -f "$DB-shm" "$DB-wal"
  launchctl kickstart -k "$GUI"

  if wait_healthy; then
    echo "!!> Rollback succeeded; previous version is live." >&2
  else
    echo "!!> ROLLBACK FAILED; backend still down. Manual intervention needed." >&2
  fi
  exit 1
fi

echo "==> Verifying public URL..."
curl -s -o /dev/null -w "public /: %{http_code}\n" https://library.joanchirinos.com/

echo "==> Deploy complete."
