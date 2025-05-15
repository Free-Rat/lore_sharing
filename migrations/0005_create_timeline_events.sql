CREATE TABLE timeline_events (
    timeline_id INTEGER NOT NULL REFERENCES timelines(id),
    event_id INTEGER NOT NULL REFERENCES events(id),
    position INTEGER NOT NULL,
    PRIMARY KEY (timeline_id, event_id)
);
