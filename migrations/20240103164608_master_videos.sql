CREATE TABLE networks (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE master_videos (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    date DATE,
    description TEXT,
    notes TEXT
);

CREATE TABLE networks_master_videos (
    network_id INTEGER NOT NULL REFERENCES networks(id),
    master_video_id INTEGER NOT NULL REFERENCES master_videos(id),
    PRIMARY KEY (network_id, master_video_id)
);
