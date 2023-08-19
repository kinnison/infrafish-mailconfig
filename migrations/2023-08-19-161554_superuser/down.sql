-- No superuser concept any more

ALTER TABLE mailuser
  DROP COLUMN superuser;
