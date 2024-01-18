-- Remove mailing lists, reverting them to aliases

-- This reversion to aliases is destructive
UPDATE mailentry SET kind='alias' WHERE kind='list';

ALTER TABLE mailentry DROP CONSTRAINT expansion_for_right_kind;
ALTER TABLE mailentry ADD CONSTRAINT expansion_for_right_kind CHECK (
        kind NOT IN ('alias', 'bouncer', 'blackhole') OR expansion IS NOT NULL
    );
