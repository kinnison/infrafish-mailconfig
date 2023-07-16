-- Authentication tokens for mail admin

CREATE TABLE mailauthtoken (
    id SERIAL NOT NULL PRIMARY KEY,
    mailuser INTEGER NOT NULL REFERENCES mailuser(id),
    token VARCHAR NOT NULL,
    label VARCHAR NOT NULL,
    CONSTRAINT mailauthtoken_token_uniq UNIQUE (token)
);

INSERT INTO mailauthtoken (mailuser, token, label)
  SELECT id, md5(gen_random_uuid()::varchar), 'Initial access token'
  FROM mailuser;
