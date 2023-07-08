-- Allow and Deny lists are needed by the frontend

CREATE TABLE allowdenylist (
    id SERIAL NOT NULL PRIMARY KEY,
    maildomain INTEGER NOT NULL REFERENCES maildomain (id),
    allow BOOLEAN NOT NULL,
    value VARCHAR NOT NULL
);
