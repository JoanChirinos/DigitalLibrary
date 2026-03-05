# DigitalLibrary

A home library management system for scanning and cataloging books.

## Stack

- **Backend:** Rust + Axum + Diesel + SQLite
- **Frontend:** Svelte 5 + TypeScript + Vite + Tailwind CSS + DaisyUI

## Features

- Add books with title, author(s), scan date, ISBN, and cover URL
- Fuzzy author autocomplete when adding books
- Unified tagging system — owners, genres, and custom tags are all tags with a `kind`
- Sort by author (last name), title, or most recent
- Filter by any combination of tags
- Paginated book list
- Stats dashboard with charts (books per tag, per author, growth over time)
- All stats filterable by tags and date range

## Getting Started

### Backend

```bash
cd backend
cargo run
```

Runs on `http://localhost:8008`. Database auto-creates on first run.

### Frontend

```bash
cd frontend
npm install
npm run dev
```

Runs on `http://localhost:1738`. Both backend and frontend need to be running.
