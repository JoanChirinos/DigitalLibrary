-- Remove library_id columns (SQLite doesn't support DROP COLUMN directly, would need table recreation)
-- For now, just drop and recreate from the first migration
DROP TABLE IF EXISTS book_tags;
DROP TABLE IF EXISTS book_authors;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS authors;
DROP TABLE IF EXISTS books;
DROP TABLE IF EXISTS libraries;
