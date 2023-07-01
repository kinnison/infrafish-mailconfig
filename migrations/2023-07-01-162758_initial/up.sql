-- Set up Infrafish mailconfig

CREATE TABLE mailuser (
    id SERIAL NOT NULL PRIMARY KEY,
    username VARCHAR NOT NULL UNIQUE
);

CREATE TABLE maildomain (
    id SERIAL NOT NULL PRIMARY KEY,
    owner INTEGER NOT NULL REFERENCES mailuser (id),
    domainname VARCHAR NOT NULL UNIQUE
);

CREATE TYPE mailentrykind AS ENUM ('login', 'account', 'alias', 'bouncer', 'blackhole');

CREATE TABLE mailentry (
    id SERIAL NOT NULL PRIMARY KEY,
    maildomain INTEGER NOT NULL REFERENCES maildomain (id),
    name VARCHAR NOT NULL UNIQUE,
    kind mailentrykind NOT NULL,
    password VARCHAR,
    expansion VARCHAR,

    CONSTRAINT only_one_value CHECK (
        ( CASE WHEN password IS NULL THEN 0 ELSE 1 END
        + CASE WHEN expansion IS NULL THEN 0 ELSE 1 END
        ) = 1
    ),

    CONSTRAINT password_for_right_kind CHECK (
        kind NOT IN ('login', 'account') OR password IS NOT NULL
    ),

    CONSTRAINT expansion_for_right_kind CHECK (
        kind NOT IN ('alias', 'bouncer', 'blackhole') OR expansion IS NOT NULL
    )
);

