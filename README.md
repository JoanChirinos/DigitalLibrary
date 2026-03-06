# DigitalLibrary

A home library management system with multi-library support, authentication, barcode scanning, and statistics.

## Features

- 📚 **Multi-library support** — Separate libraries with passkey authentication
- 📷 **Barcode scanning** — Scan ISBN barcodes with your camera
- 🔍 **ISBN lookup** — Auto-fill book details from Open Library API
- ✏️ **In-place editing** — Edit books directly in the list
- 🏷️ **Unified tagging** — Owners, genres, and custom tags in one system
- 📊 **Stats dashboard** — Charts for books per tag/author, collection growth
- 🔎 **Fuzzy search** — Search books and authors with typo tolerance
- 🌓 **Dark mode** — Theme toggle with system preference detection
- 💾 **Automated backups** — Daily backups via launchd/cron

## Stack

- **Backend:** Rust + Axum + Diesel + SQLite
- **Frontend:** Svelte 5 + TypeScript + Vite + Tailwind CSS + DaisyUI
- **Auth:** bcrypt + UUID tokens
- **Charts:** Chart.js
- **Barcode:** html5-qrcode
- **Search:** Fuse.js

## Quick Start

### Backend

```bash
cd backend
cargo run
```

Server runs on `http://localhost:8008`. Database auto-creates on first run.

### Frontend

```bash
cd frontend
npm install
npm run dev
```

UI runs on `http://localhost:1738`. Both backend and frontend must be running.

### First Use

1. Open `http://localhost:1738`
2. Click "Create New Library"
3. Enter a name and passkey
4. Start adding books!

## Documentation

- **[ARCHITECTURE.md](ARCHITECTURE.md)** — Complete technical documentation, design decisions, limitations
- **[project.md](project.md)** — Development notes and implementation details
- **[BACKUP.md](BACKUP.md)** — Automated backup setup instructions
- **[DEPLOY.md](DEPLOY.md)** — Production deployment guide (nginx + Cloudflare)

## Key Concepts

**Libraries** — Isolated collections with separate authentication. Perfect for multiple users or separating personal/shared books.

**Tags** — Everything is a tag with a `kind`:
- `owner` — Who owns the book (Joan, Siena)
- `genre` — Book categories (sci-fi, fantasy, etc.)
- `custom` — Anything else (favorite, to-read, signed, etc.)

**Authors** — Stored as first + last name for proper alphabetical sorting. Fuzzy matching helps avoid duplicates.

**Scan Date** — Stored as UTC timestamp with second precision. Displayed in local timezone. Used for "most recent" sorting and growth charts.

## Development

**Requirements:**
- Rust 1.86+ (for Diesel 2.2+)
- Node 20+ (for Vite 6 + Svelte 5)
- SQLite 3

**Database location:** `backend/library.db`

**Migrations:** Auto-applied on startup from `backend/migrations/`

## License

See [LICENSE](LICENSE) file.
