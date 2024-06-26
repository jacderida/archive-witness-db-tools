CREATE TABLE releases (
    id SERIAL PRIMARY KEY,
    date DATE NOT NULL,
    name VARCHAR NOT NULL,
    directory_name VARCHAR,
    file_count SMALLINT,
    size BIGINT,
    torrent_url VARCHAR
);

CREATE TABLE release_files (
    id SERIAL PRIMARY KEY,
    path VARCHAR NOT NULL,
    size BIGINT NOT NULL,
    release_id INTEGER NOT NULL REFERENCES releases(id)
);
