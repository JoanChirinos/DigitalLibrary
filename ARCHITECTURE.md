# DigitalLibrary — Complete Project Documentation

## Overview

A home library management system for cataloging books with multi-library support, authentication, barcode scanning, and statistics. Built for Joan and Siena to track their personal book collections.

---

## Features

### Core Functionality

**Multi-Library with Authentication**
- Each library has a unique name and passkey (bcrypt hashed)
- Login screen shows library cards, click to select and enter passkey
- Create new libraries from the login screen
- Token-based auth (UUID tokens, 30-day cookie, in-memory storage)
- Auto-logout on 401 responses
- Logout button in top-right

**Book Management**
- Add books with: title, authors (first/last name), scan date/time, ISBN, cover URL
- ISBN lookup via Open Library API (auto-fills title, authors, cover, suggests genres)
- Barcode scanning via device camera (html5-qrcode, EAN-13 detection)
- Rate limiting on ISBN lookups (5 seconds between requests, cookie-based)
- Edit books in-place (card expands to inline form, scan date immutable)
- Delete with confirmation modal
- Book detail view (click card to see large cover + all metadata in modal)

**Author Handling**
- Stored as first_name + last_name for proper sorting
- Fuzzy autocomplete when adding (Fuse.js, threshold 0.5)
- Find-or-create on book creation (checks existing authors per library)
- Complex names (3+ words) pre-fill fields for manual split adjustment

**Tagging System**
- Unified system: owners, genres, and custom tags all use the same `tags` table with a `kind` field
- Genre tags forced lowercase
- Create tags inline when adding/editing books
- Tag manager page for bulk creation/deletion
- Delete with confirmation modal (removes from all books via CASCADE)

**Library View**
- Sort by: author last name (default), title, or most recent
- Filter by: owner (pills), genre (dropdown), custom tags (dropdown)
- Search: fuzzy search across titles and author names (Fuse.js, threshold 0.4)
- Pagination: 5/20/50 per page, shows "X / Y" with prev/next buttons
- Book cards show: cover thumbnail (lazy loaded), title, authors, tags, scan date
- Edit and delete buttons per book

**Stats Dashboard**
- Total counts (books, authors, tags) with optional tag/date filters
- Books per tag chart (horizontal bar, filterable by kind, excludes filtered tags)
- Books per author chart (horizontal bar, top 20, sorted by last name)
- Collection growth chart (vertical bar, group by day/month/year)
- All charts respond to tag filters at the top

**UI/UX**
- Dark mode toggle (respects system preference, persists in cookie)
- Tab navigation (Library/Tags/Stats) with URL hash persistence
- DaisyUI 5 + Tailwind CSS 4 styling
- Lucide icons
- Responsive layout (max 900px centered)

---

## Technical Stack

### Backend (Rust)
- **Framework:** Axum 0.8 on Tokio
- **Database:** SQLite via Diesel 2.2 ORM
- **Migrations:** Embedded with diesel_migrations (auto-run on startup)
- **Auth:** bcrypt for password hashing, UUID tokens, in-memory token store
- **CORS:** Explicit headers (Content-Type, Authorization) for local dev
- **Port:** 8008

### Frontend (Svelte 5 + TypeScript)
- **Framework:** Svelte 5 (runes: `$state`, `$derived`, `$effect`)
- **Build:** Vite 6
- **Styling:** Tailwind CSS 4 + DaisyUI 5
- **Icons:** Lucide
- **Charts:** Chart.js (tree-shaken imports)
- **Fuzzy Search:** Fuse.js (authors, books)
- **Barcode:** html5-qrcode
- **Port:** 1738

---

## Key Design Decisions

### Why unified tagging (owner/genre/custom)?
- **Decision:** All tags in one table with a `kind` column
- **Justification:** Simpler schema, one set of endpoints, uniform filtering/stats. Owners and genres are just labels like any other tag.
- **Tradeoff:** Slightly less semantic than separate tables, but the flexibility outweighs it

### Why first_name + last_name for authors?
- **Decision:** Split author names into two fields
- **Justification:** Proper sorting by last name, better for multi-word names (e.g., "Ursula K. Le Guin")
- **Tradeoff:** More complex input (two fields), parsing ambiguity for 3+ word names (handled with pre-fill for manual adjustment)

### Why store scan_date as ISO datetime string (UTC)?
- **Decision:** TEXT column with ISO 8601 format (`2026-03-05T14:30:00Z`)
- **Justification:** Precise ordering (second resolution), string comparison works for sorting, human-readable in DB, no timezone conversion bugs
- **Tradeoff:** Slightly larger than epoch integers, but negligible

### Why client-side fuzzy search instead of backend?
- **Decision:** Fuse.js in the frontend for author autocomplete and book search
- **Justification:** Instant feedback, no network latency, works offline, simpler backend
- **Tradeoff:** Doesn't scale to 100k+ books, but fine for home library use

### Why in-memory token store instead of database?
- **Decision:** HashMap in Arc<RwLock<>> for tokens
- **Justification:** Simple, fast, no DB writes on every auth check
- **Tradeoff:** Tokens lost on server restart (users have to re-login), but acceptable for a home server

### Why no raw SQL in queries?
- **Decision:** All queries through Diesel's type-safe query builder
- **Justification:** Injection-proof by construction, compile-time type checking, refactor-safe
- **Tradeoff:** More verbose than raw SQL, some queries (like growth bucketing) done in Rust instead of SQL

### Why library_id column instead of separate DB files?
- **Decision:** Single SQLite file with library_id foreign keys
- **Justification:** Simpler deployment, single backup, easier to add cross-library features later
- **Tradeoff:** Every query needs library_id filtering, but Diesel makes this trivial

---

## Limitations

### Performance
- **Stats endpoints** make 2 queries each (filtered book IDs, then aggregation). Could be optimized to 1 with subqueries, but negligible for <50k books.
- **Growth bucketing** loads all dates into memory and groups in Rust. Could use SQL `strftime` + `GROUP BY`, but Diesel's type system makes this difficult. Fine for home library scale.
- **Author/tag autocomplete** searches entire dataset client-side. Doesn't scale to massive libraries, but instant for <10k books.

### Security
- **Tokens never expire** — Once logged in, token is valid until server restart or manual logout. Could add expiration timestamps.
- **No rate limiting on auth endpoints** — Could brute-force passkeys. Acceptable for a home server not exposed to internet.
- **CORS wide open** — `allow_origin(Any)` for local dev. Should be restricted in production.
- **Passkeys not validated** — No minimum length, complexity requirements. Users can set weak passkeys.

### Functionality
- **No user roles/permissions** — Everyone with a library's passkey has full access. No read-only mode.
- **No audit log** — Can't see who added/edited/deleted what and when.
- **No undo** — Deletes are permanent (except via backup restore).
- **No bulk operations** — Can't select multiple books to tag/delete at once.
- **No book series/collections** — Books are flat, no grouping beyond tags.
- **No reading progress** — Can't track "currently reading" or "finished on X date".

### Barcode Scanning
- **Requires HTTPS for camera on non-localhost** — Works on localhost, but needs SSL cert for LAN access.
- **Only EAN-13 barcodes** — Won't read ISBN-10 barcodes (though ISBN-10 can be typed manually).
- **No offline mode** — ISBN lookup requires internet connection.

---

## Future Work

### High Priority
- **Export/import** — Download library as JSON, upload to restore or migrate
- **Deploy guide** — Instructions for production (build frontend, serve from Axum, systemd service, reverse proxy)
- **Token expiration** — Add timestamps, auto-refresh, or session timeout

### Medium Priority
- **Better error handling** — Toast notifications instead of silent failures
- **Loading states** — Spinners while fetching data
- **Confirmation on navigation** — Warn if leaving edit form with unsaved changes
- **Book cover upload** — Upload images instead of just URL input, store in `covers/` directory
- **Bulk operations** — Select multiple books, apply tags or delete

### Low Priority
- **Audit log** — Track all changes with user, timestamp, action
- **Reading progress** — "Currently reading", "Finished", dates
- **Series/collections** — Group related books
- **Advanced search** — Filter by ISBN, date range, multiple authors
- **Mobile app** — Native iOS/Android with offline support
- **Shared libraries** — Multiple users per library with different permissions

---

## Backup Strategy

**Automated:** launchd job runs daily at 2am, keeps last 30 backups in `~/DigitalLibrary-backups/`

**Manual:** Run `./scripts/backup.sh` anytime

**Restore:** Copy backup file to `backend/library.db`, restart server

See `BACKUP.md` for full instructions.

---

## Development Notes

### Running Locally

**Backend:**
```bash
cd backend
cargo run
```
Runs on http://localhost:8008

**Frontend:**
```bash
cd frontend
npm install
npm run dev
```
Runs on http://localhost:1738

### Database

- **File:** `backend/library.db` (auto-created on first run)
- **Migrations:** `backend/migrations/` (embedded, auto-applied)
- **Schema:** 6 tables (libraries, books, authors, tags, book_authors, book_tags)

### Testing

No automated tests. Manual testing via:
- Browser UI
- curl for API endpoints
- SQLite CLI for DB inspection

### Code Organization

**Backend:**
- `main.rs` — Server setup, CORS, state management
- `auth.rs` — Token generation, validation, helper functions
- `db.rs` — Connection pool, migration runner
- `models.rs` — Diesel structs (Queryable, Insertable)
- `schema.rs` — Diesel table definitions
- `routes/` — API handlers (auth, books, tags, stats)

**Frontend:**
- `App.svelte` — Root component, auth check, tab navigation, theme toggle
- `stores.ts` — Reactive stores for books and tags
- `api.ts` — All backend API calls with auth headers
- `components/` — Login, BookList, BookForm, TagManager, StatsGraph

---

## Deployment Checklist

When deploying to production:

- [ ] Build frontend: `cd frontend && npm run build`
- [ ] Serve frontend from Axum or nginx
- [ ] Update CORS to restrict origins
- [ ] Use HTTPS (required for camera access on non-localhost)
- [ ] Set up systemd service or launchd daemon
- [ ] Configure firewall (only expose necessary ports)
- [ ] Set up automated backups (launchd or cron)
- [ ] Consider token expiration
- [ ] Add rate limiting on auth endpoints
- [ ] Monitor logs

---

## License

See LICENSE file.
