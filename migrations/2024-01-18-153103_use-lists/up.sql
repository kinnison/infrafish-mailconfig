-- Use the new list type in mailentry

ALTER TABLE mailentry DROP CONSTRAINT expansion_for_right_kind;
ALTER TABLE mailentry ADD CONSTRAINT expansion_for_right_kind CHECK (
        kind NOT IN ('alias', 'bouncer', 'blackhole', 'list') OR expansion IS NOT NULL
    );
