CREATE TABLE news_networks (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    description TEXT NOT NULL
);

CREATE TABLE news_affiliates (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    description TEXT NOT NULL,
    region VARCHAR NOT NULL,
    news_network_id INTEGER NOT NULL REFERENCES news_networks(id)
);

CREATE TABLE news_broadcasts (
    id SERIAL PRIMARY KEY,
    date DATE,
    description TEXT,
    news_network_id INTEGER REFERENCES news_networks(id),
    news_affiliate_id INTEGER REFERENCES news_affiliates(id)
);
