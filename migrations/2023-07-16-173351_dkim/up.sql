-- Add support for domainkeys

CREATE TABLE maildomainkey (
    id SERIAL NOT NULL PRIMARY KEY,
    maildomain INTEGER NOT NULL REFERENCES maildomain(id),
    selector VARCHAR NOT NULL,
    privkey TEXT NOT NULL,
    pubkey TEXT NOT NULL,
    signing BOOLEAN NOT NULL,

    CONSTRAINT maildomainkey_selector_valid CHECK (selector ~* '^[a-z][a-z0-9-]*$'),
    CONSTRAINT maildomainkey_selector_uniq UNIQUE(maildomain, selector)
);
