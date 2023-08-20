-- Reset mailentry unique(name) to be mailentry unique(maildomain,name)

ALTER TABLE mailentry DROP CONSTRAINT mailentry_name_key;

ALTER TABLE mailentry ADD CONSTRAINT mailentry_name_uniq UNIQUE (maildomain, name);
