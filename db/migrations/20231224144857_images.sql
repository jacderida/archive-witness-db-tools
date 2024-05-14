CREATE TABLE photographers (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE images (
    id INTEGER PRIMARY KEY REFERENCES content(id),
    album VARCHAR,
    caption VARCHAR,
    date_recorded TIMESTAMP,
    file_metadata VARCHAR NOT NULL,
    file_size BIGINT NOT NULL,
    horizontal_pixels SMALLINT NOT NULL,
    name VARCHAR NOT NULL,
    notes TEXT,
    received_from VARCHAR,
    shot_from VARCHAR,
    vertical_pixels SMALLINT NOT NULL
);

CREATE TABLE images_tags (
    image_id INTEGER NOT NULL REFERENCES images(id),
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    PRIMARY KEY (image_id, tag_id)
);

CREATE TABLE images_photographers (
    photographer_id INTEGER NOT NULL REFERENCES photographers(id),
    image_id INTEGER NOT NULL REFERENCES images(id),
    PRIMARY KEY (photographer_id, image_id)
);

CREATE INDEX idx_images_photographers_photographer_id ON images_photographers(photographer_id);
CREATE INDEX idx_images_photographers_image_id ON images_photographers(image_id);
CREATE INDEX idx_image_tags_image_id ON images_tags(image_id);
CREATE INDEX idx_image_tags_tag_id ON images_tags(tag_id);
