ALTER TABLE person_aggregates
    DROP COLUMN published;

DROP TABLE search_combined;

DELETE FROM history_status
WHERE dest = 'search_combined';

