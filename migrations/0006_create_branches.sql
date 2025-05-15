CREATE TABLE branches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    author_id INTEGER NOT NULL REFERENCES users(id),
    original_timeline_id INTEGER NOT NULL REFERENCES timelines(id),
    description TEXT NOT NULL,
    area_start INTEGER NOT NULL,
    area_end INTEGER NOT NULL
);
