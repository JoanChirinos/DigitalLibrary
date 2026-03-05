-- Create libraries table
CREATE TABLE libraries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    passkey_hash TEXT NOT NULL
);

-- Add library_id to existing tables
ALTER TABLE books ADD COLUMN library_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE authors ADD COLUMN library_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE tags ADD COLUMN library_id INTEGER NOT NULL DEFAULT 1;
