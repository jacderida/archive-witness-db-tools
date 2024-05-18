CREATE TYPE category AS ENUM (
    'amateurfootage',
    'compilation',
    'documentary',
    'news',
    'professionalfootage',
    'survivoraccount'
);
CREATE TYPE event_type AS ENUM (
    'camerasource',
    'jumper',
    'key',
    'normal',
    'person',
    'pentagonattack',
    'report',
    'wtc1collapse',
    'wtc1impact',
    'wtc2collapse',
    'wtc2impact'
);

CREATE TABLE master_videos (
    id SERIAL PRIMARY KEY,
    categories category[] NOT NULL,
    title VARCHAR NOT NULL UNIQUE,
    date DATE,
    description TEXT NOT NULL,
    links VARCHAR[],
    nist_notes TEXT
);

CREATE TABLE event_timestamps (
    id SERIAL PRIMARY KEY,
    description VARCHAR NOT NULL,
    timestamp INTERVAL NOT NULL,
    event_type event_type NOT NULL,
    time_of_day TIME,
    master_video_id INTEGER NOT NULL REFERENCES master_videos(id)
);

CREATE TABLE master_videos_news_broadcasts (
    master_video_id INTEGER NOT NULL REFERENCES master_videos(id),
    news_broadcast_id INTEGER NOT NULL REFERENCES news_broadcasts(id),
    PRIMARY KEY (master_video_id, news_broadcast_id)
);

CREATE TABLE master_videos_release_files (
    master_video_id INTEGER NOT NULL REFERENCES master_videos(id),
    release_file_id INTEGER NOT NULL REFERENCES release_files(id),
    PRIMARY KEY (master_video_id, release_file_id)
);

CREATE TABLE master_videos_people (
    master_video_id INTEGER NOT NULL REFERENCES master_videos(id),
    person_id INTEGER NOT NULL REFERENCES people(id),
    PRIMARY KEY (master_video_id, person_id)
);

CREATE TABLE videos (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL UNIQUE,
    description TEXT,
    duration INTERVAL NOT NULL,
    link VARCHAR NOT NULL,
    is_primary BOOLEAN NOT NULL,
    master_id INTEGER NOT NULL REFERENCES master_videos(id)
);
