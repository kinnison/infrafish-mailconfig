-- Undo fix of maildomain name uniqueness, this will break stuff

ALTER TABLE mailentry DROP CONSTRAINT mailentry_name_uniq;

ALTER TABLE mailentry ADD CONSTRAINT mailentry_name_key UNIQUE (name);
