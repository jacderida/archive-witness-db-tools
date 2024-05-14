CREATE TABLE news_networks (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    description TEXT NOT NULL
);

INSERT INTO news_networks (name, description) VALUES
('ABC News', 'ABC News is the news division of the American Broadcasting Company.'),
('Fox News', 'Fox News is a news channel owned by Fox Corporation.'),
('NBC', 'Placeholder'),
('CNN', 'Placeholder'),
('CBS', 'Placeholder');

CREATE TABLE news_affiliates (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    description TEXT NOT NULL,
    region VARCHAR NOT NULL,
    news_network_id INTEGER NOT NULL REFERENCES news_networks(id)
);

INSERT INTO news_affiliates (name, description, region, news_network_id)
VALUES (
    'WABC-TV',
    'WABC-TV, on channel 7, is a New York affiliate of the ABC network. They are among many '
    'affiliates who use the ‘ABC7’ branding.',
    'NYC',
    1
);

CREATE TABLE news_broadcasts (
    id SERIAL PRIMARY KEY,
    date DATE,
    description TEXT,
    news_network_id INTEGER REFERENCES news_networks(id),
    news_affiliate_id INTEGER REFERENCES news_affiliates(id)
);

INSERT INTO news_broadcasts (date, description, news_network_id, news_affiliate_id)
VALUES (
    '2001-09-11',
    'When American Airlines Flight 11 crashed into the North Tower of the World Trade Center on the '
    'morning of September 11, like many ABC affiliates, WABC-TV were broadcasting *Good Morning '
    'America*, a show produced by the national ABC News division. After resuming from commercial '
    'break at 0851, the channel broke away from national programming to their own *Eyewitness News '
    'Special Report*, anchored by Steve Bartelstein, who was later accompanied by Lori Stokes and '
    'others. However, at various points they did return to the national coverage.\n\n'
    'The national coverage from ABC News is also utilised at certain points.\n\n'
    'WABC-TV was one of many local television stations who had transmitter facilities located on '
    'the top floor of the North Tower of the World Trade Center. One of their maintenance '
    'engineers, Donald DiFranco, was a victim of the attacks.',
    1,
    1
);
