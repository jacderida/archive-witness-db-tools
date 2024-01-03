CREATE TABLE master_videos (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    date DATE,
    description TEXT,
    format VARCHAR,
    network VARCHAR,
    source VARCHAR,
    notes TEXT
)
