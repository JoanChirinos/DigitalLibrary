-- Rebuild authors and tags so their UNIQUE constraints are scoped per library
-- (and include middle_name for authors). SQLite can't drop a UNIQUE constraint
-- in place, so we recreate each table preserving ids. FKs are disabled during
-- the swap so book_authors/book_tags rows (referenced by id) are left intact.
-- Runs outside a transaction (metadata.toml) so the foreign_keys pragma applies.
PRAGMA foreign_keys=OFF;

CREATE TABLE authors_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name TEXT NOT NULL,
    middle_name TEXT,
    last_name TEXT NOT NULL,
    library_id INTEGER NOT NULL DEFAULT 1,
    UNIQUE(library_id, first_name, middle_name, last_name)
);
INSERT INTO authors_new (id, first_name, middle_name, last_name, library_id)
    SELECT id, first_name, middle_name, last_name, library_id FROM authors;
DROP TABLE authors;
ALTER TABLE authors_new RENAME TO authors;

CREATE TABLE tags_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'custom',
    library_id INTEGER NOT NULL DEFAULT 1,
    UNIQUE(library_id, name, kind)
);
INSERT INTO tags_new (id, name, kind, library_id)
    SELECT id, name, kind, library_id FROM tags;
DROP TABLE tags;
ALTER TABLE tags_new RENAME TO tags;

PRAGMA foreign_keys=ON;
