-- Add migration script here
PRAGMA foreign_keys = ON;
CREATE TABLE practice_regiment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,       -- Automatically incrementing ID
    date TIMESTAMP NOT NULL      -- Date of the practice regiment
);

CREATE TABLE practice_piece (
    id INTEGER PRIMARY KEY AUTOINCREMENT,             -- Automatically incrementing ID
    practice_regiment_id INTEGER NOT NULL, -- Foreign key to `practice_regiment` table
    name TEXT NOT NULL,                    -- Each individual piece name
    FOREIGN KEY (practice_regiment_id) REFERENCES practice_regiment(id) ON DELETE CASCADE
);
