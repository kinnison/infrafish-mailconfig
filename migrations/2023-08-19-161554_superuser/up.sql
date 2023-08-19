-- Add the concept of whether or not this user is a superuser

ALTER TABLE mailuser
  ADD COLUMN superuser BOOLEAN;

UPDATE mailuser SET superuser=false;
UPDATE mailuser SET superuser=true where id=1;

ALTER TABLE mailuser
  ALTER COLUMN superuser SET NOT NULL;

