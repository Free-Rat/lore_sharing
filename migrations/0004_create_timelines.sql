CREATE TABLE timelines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    author_id INTEGER NOT NULL REFERENCES users(id),
    description TEXT NOT NULL,
    start INTEGER NOT NULL,
    end INTEGER NOT NULL,
    unit TEXT NOT NULL,
    universe_name TEXT NOT NULL REFERENCES universes(name)
);
