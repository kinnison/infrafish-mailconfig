-- Add remote MX support to maildomain

ALTER TABLE maildomain
   ADD COLUMN remotemx VARCHAR;
