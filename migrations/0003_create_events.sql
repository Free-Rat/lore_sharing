CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    reference TEXT NOT NULL,
    image TEXT,
    thumbnail TEXT,
    author_id INTEGER NOT NULL REFERENCES users(id)
);
