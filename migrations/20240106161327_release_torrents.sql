CREATE TABLE release_torrents (
    release_id INTEGER PRIMARY KEY,
    content BYTEA,
    FOREIGN KEY (release_id) REFERENCES releases(id)
)
