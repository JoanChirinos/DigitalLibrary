# Production Deployment Guide

## Overview

Deploy DigitalLibrary to run on your MacBook behind nginx and Cloudflare.

**Architecture:**
```
Internet → Cloudflare → MacBook :443 (nginx) → Axum :8008
```

---

## Prerequisites

- nginx installed (`brew install nginx`)
- Cloudflare account with joanchirinos.com
- SSL certificate (Cloudflare Origin or Let's Encrypt)

---

## Step 1: Build Frontend

```bash
cd ~/Desktop/DigitalLibrary/frontend
npm install
npm run build
```

Creates `frontend/dist/` with static files.

---

## Step 2: Build Backend (Release Mode)

```bash
cd ~/Desktop/DigitalLibrary/backend
cargo build --release
```

Binary will be at `target/release/digital-library`.

---

## Step 3: Configure nginx

**Create `/usr/local/etc/nginx/servers/digitallibrary.conf`:**

```nginx
server {
    listen 443 ssl http2;
    server_name library.joanchirinos.com;

    # SSL certificates (update paths)
    ssl_certificate /path/to/fullchain.pem;
    ssl_certificate_key /path/to/privkey.pem;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Proxy everything to Axum (serves static + API)
    location / {
        proxy_pass http://localhost:8008;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name library.joanchirinos.com;
    return 301 https://$server_name$request_uri;
}
```

**Test config:**
```bash
sudo nginx -t
```

**Start/reload nginx:**
```bash
sudo brew services start nginx
# Or if already running:
sudo nginx -s reload
```

---

## Step 4: SSL Certificate

### Option A: Cloudflare Origin Certificate (recommended)

1. Cloudflare dashboard → SSL/TLS → Origin Server
2. Create certificate
3. Download `cert.pem` and `key.pem`
4. Save to `/usr/local/etc/nginx/ssl/`
5. Update nginx config paths

### Option B: Let's Encrypt

```bash
brew install certbot
sudo certbot certonly --standalone -d library.joanchirinos.com
```

Certs in `/etc/letsencrypt/live/library.joanchirinos.com/`

Update nginx config:
```nginx
ssl_certificate /etc/letsencrypt/live/library.joanchirinos.com/fullchain.pem;
ssl_certificate_key /etc/letsencrypt/live/library.joanchirinos.com/privkey.pem;
```

---

## Step 5: Cloudflare DNS

**In Cloudflare dashboard:**

Add A record:
- `library.joanchirinos.com` → your MacBook public IP
- Enable "Proxied" (orange cloud)

**Cloudflare SSL settings:**
- SSL/TLS mode: "Full" or "Full (strict)"

---

## Step 6: Auto-Start Backend (launchd)

**Create `~/Library/LaunchAgents/com.digitallibrary.backend.plist`:**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.digitallibrary.backend</string>
    
    <key>ProgramArguments</key>
    <array>
        <string>/Users/joanchir/Desktop/DigitalLibrary/backend/target/release/digital-library</string>
    </array>
    
    <key>WorkingDirectory</key>
    <string>/Users/joanchir/Desktop/DigitalLibrary/backend</string>
    
    <key>RunAtLoad</key>
    <true/>
    
    <key>KeepAlive</key>
    <true/>
    
    <key>StandardOutPath</key>
    <string>/Users/joanchir/Desktop/DigitalLibrary/backend/stdout.log</string>
    
    <key>StandardErrorPath</key>
    <string>/Users/joanchir/Desktop/DigitalLibrary/backend/stderr.log</string>
</dict>
</plist>
```

**Load it:**
```bash
launchctl load ~/Library/LaunchAgents/com.digitallibrary.backend.plist
```

**Check status:**
```bash
launchctl list | grep digitallibrary
```

---

## Step 7: Start Everything

**Manual start (for testing):**
```bash
cd ~/Desktop/DigitalLibrary/backend
cargo run --release
```

**Or use launchd** (auto-starts on boot, restarts on crash)

---

## Testing

**Local:**
- http://localhost:8008 — Should show the app

**Public (after DNS):**
- https://library.joanchirinos.com

---

## Troubleshooting

**nginx won't start:**
```bash
sudo nginx -t  # Check syntax
tail -f /usr/local/var/log/nginx/error.log
```

**Port 443 already in use:**
```bash
lsof -i :443  # See what's using it
# If it's your old Node setup, stop it first
```

**Backend not starting:**
```bash
tail -f ~/Desktop/DigitalLibrary/backend/stderr.log
```

**Camera not working:**
- HTTPS required for camera access
- Check browser console for permission errors
- Verify SSL cert is valid

**Cloudflare not routing:**
- Check DNS propagation: `dig library.joanchirinos.com`
- Verify Cloudflare SSL mode is "Full"
- Check firewall allows port 443

---

## Updating

**After code changes:**

```bash
# Rebuild frontend
cd ~/Desktop/DigitalLibrary/frontend
npm run build

# Rebuild backend
cd ~/Desktop/DigitalLibrary/backend
cargo build --release

# Restart backend
launchctl unload ~/Library/LaunchAgents/com.digitallibrary.backend.plist
launchctl load ~/Library/LaunchAgents/com.digitallibrary.backend.plist
```

---

## Uninstall

```bash
# Stop backend
launchctl unload ~/Library/LaunchAgents/com.digitallibrary.backend.plist
rm ~/Library/LaunchAgents/com.digitallibrary.backend.plist

# Remove nginx config
rm /usr/local/etc/nginx/servers/digitallibrary.conf
sudo nginx -s reload

# Remove DNS record from Cloudflare
```
