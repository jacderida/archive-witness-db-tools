CREATE TYPE person_type AS ENUM (
    'eyewitness',
    'fire',
    'portauthority',
    'police',
    'reporter',
    'survivor',
    'victim',
    'videographer'
);

CREATE TABLE people (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    description VARCHAR,
    historical_title VARCHAR,
    types person_type[] NOT NULL
);

INSERT INTO people (name, description, types)
VALUES (
    'John DelGiorno',
    'John DelGiorno was a reporter for WABC-TV in New York. On September 11, 2001, he delivering '
    'reports and controlling the camera from NewsCopter7. This camera feed supplied much of the ABC '
    'coverage in the morning.',
    ARRAY['reporter', 'videographer']::person_type[]
);
