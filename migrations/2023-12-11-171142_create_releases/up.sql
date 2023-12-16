CREATE TABLE releases (
    id SERIAL PRIMARY KEY,
    date DATE NOT NULL,
    name VARCHAR NOT NULL,
    directory_name VARCHAR,
    file_count SMALLINT,
    size BIGINT,
    torrent_url VARCHAR
)
