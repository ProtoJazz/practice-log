-- Add migration script here
CREATE TABLE practice_piece_log_entry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,            -- Automatically incrementing ID
    practice_piece_id INTEGER NOT NULL,              -- Foreign key to `practice_piece`
    bpm INTEGER NOT NULL,                               -- The BPM value at the time of the log
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,    -- The timestamp of the log entry
    FOREIGN KEY (practice_piece_id) REFERENCES practice_piece(id) ON DELETE CASCADE
);
