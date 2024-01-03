CREATE TABLE nist_videos (
    video_id INTEGER PRIMARY KEY,
    video_title VARCHAR(120) NOT NULL,
    network VARCHAR(50),
    broadcast_date DATE,
    duration_min INTEGER NOT NULL,
    subject VARCHAR(50),
    notes TEXT
);

CREATE TABLE nist_tapes (
    tape_id INTEGER PRIMARY KEY,
    video_id INTEGER NOT NULL REFERENCES nist_videos(video_id),
    tape_name VARCHAR(120) NOT NULL,
    tape_source VARCHAR(50) NOT NULL,
    copy INTEGER NOT NULL,
    derived_from INTEGER NOT NULL,
    format VARCHAR(50) NOT NULL,
    duration_min INTEGER NOT NULL,
    batch BOOLEAN NOT NULL,
    clips BOOLEAN NOT NULL,
    timecode BOOLEAN NOT NULL
);
