CREATE TABLE timeline_merges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_timeline_id INTEGER NOT NULL REFERENCES timelines(id),
    target_timeline_id INTEGER NOT NULL REFERENCES timelines(id),
    merged_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
