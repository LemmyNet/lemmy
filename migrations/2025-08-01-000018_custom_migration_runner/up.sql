DROP SCHEMA IF EXISTS r CASCADE;

CREATE TABLE previously_run_sql (
    -- For compatibility with Diesel
    id boolean PRIMARY KEY,
    -- Too big to be used as primary key
    content text NOT NULL
);

INSERT INTO previously_run_sql (id, content)
    VALUES (TRUE, '');

