-- Settings for mail domains needed by inmails

ALTER TABLE maildomain
  ADD COLUMN sender_verify BOOLEAN,
  ADD COLUMN grey_listing BOOLEAN,
  ADD COLUMN virus_check BOOLEAN,
  ADD COLUMN spamcheck_threshold INTEGER;

UPDATE maildomain SET sender_verify = TRUE, grey_listing = FALSE, virus_check = TRUE, spamcheck_threshold = 150;

ALTER TABLE maildomain
  ALTER COLUMN sender_verify SET NOT NULL,
  ALTER COLUMN grey_listing SET NOT NULL,
  ALTER COLUMN virus_check SET NOT NULL,
  ALTER COLUMN spamcheck_threshold SET NOT NULL;

