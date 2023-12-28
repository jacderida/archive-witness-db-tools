CREATE TYPE content_type AS ENUM ('image', 'video');

CREATE TABLE content (
    id SERIAL PRIMARY KEY,
    content_type content_type NOT NULL,
    file_path VARCHAR,
    release_id INTEGER REFERENCES releases(id)
);

CREATE INDEX idx_content_type ON content(content_type);
CREATE INDEX idx_content_release_id ON content(release_id);
