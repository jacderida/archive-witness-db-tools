CREATE TABLE videographers (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE reporters (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE people (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    historical_title VARCHAR
);

CREATE TABLE jumper_timestamps (
    id SERIAL PRIMARY KEY,
    timestamp INTERVAL NOT NULL
);

CREATE TABLE videos (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL UNIQUE,
    description TEXT,
    timestamps TEXT,
    duration INTERVAL,
    link VARCHAR,
    nist_notes TEXT,
    master_id INTEGER NOT NULL REFERENCES master_videos(id)
);

CREATE TABLE videos_videographers (
    video_id INTEGER NOT NULL REFERENCES videos(id),
    videographer_id INTEGER NOT NULL REFERENCES videographers(id),
    PRIMARY KEY (video_id, videographer_id)
);

CREATE TABLE videos_reporters (
    video_id INTEGER NOT NULL REFERENCES videos(id),
    reporter_id INTEGER NOT NULL REFERENCES reporters(id),
    PRIMARY KEY (video_id, reporter_id)
);

CREATE TABLE videos_people (
    video_id INTEGER NOT NULL REFERENCES videos(id),
    person_id INTEGER NOT NULL REFERENCES people(id),
    PRIMARY KEY (video_id, person_id)
);

CREATE TABLE videos_jumper_timestamps (
    video_id INTEGER NOT NULL REFERENCES videos(id),
    jumper_timestamp_id INTEGER NOT NULL REFERENCES jumper_timestamps(id),
    PRIMARY KEY (video_id, jumper_timestamp_id)
);

CREATE TABLE videos_release_files (
    video_id INTEGER NOT NULL REFERENCES videos(id),
    release_file_id INTEGER NOT NULL REFERENCES release_files(id),
    PRIMARY KEY (video_id, release_file_id)
);
