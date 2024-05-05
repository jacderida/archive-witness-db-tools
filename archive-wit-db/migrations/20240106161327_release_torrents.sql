CREATE TABLE release_torrents (
    release_id INTEGER PRIMARY KEY,
    content BYTEA NOT NULL,
    FOREIGN KEY (release_id) REFERENCES releases(id)
)
