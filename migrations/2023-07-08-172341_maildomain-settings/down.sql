-- Remove maildomain settings

ALTER TABLE maildomain
  DROP COLUMN sender_verify,
  DROP COLUMN grey_listing,
  DROP COLUMN virus_check,
  DROP COLUMN spamcheck_threshold;
