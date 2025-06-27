DROP TABLE person_content_combined;

DROP TABLE person_saved_combined;

DELETE FROM history_status
WHERE dest IN ('person_content_combined', 'person_saved_combined');

