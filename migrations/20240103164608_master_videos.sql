CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE
);

INSERT INTO categories (name) VALUES
('news'),
('documentary'),
('amateur-footage'),
('professional-footage'),
('compilation');

CREATE TABLE networks (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE
);

INSERT INTO networks (name) VALUES
('ABC'),
('Fox'),
('NBC'),
('CNN'),
('CBS');

CREATE TABLE master_videos (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL UNIQUE,
    date DATE,
    description TEXT
);

CREATE TABLE video_urls (
    master_video_id INTEGER NOT NULL REFERENCES master_videos(id),
    url TEXT NOT NULL,
    PRIMARY KEY (master_video_id, url)
);

CREATE TABLE networks_master_videos (
    network_id INTEGER NOT NULL REFERENCES networks(id),
    master_video_id INTEGER NOT NULL REFERENCES master_videos(id),
    PRIMARY KEY (network_id, master_video_id)
);

CREATE TABLE categories_master_videos (
    category_id INTEGER NOT NULL REFERENCES categories(id),
    master_video_id INTEGER NOT NULL REFERENCES master_videos(id),
    PRIMARY KEY (category_id, master_video_id)
);
