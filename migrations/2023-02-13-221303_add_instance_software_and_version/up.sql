-- Add Software and Version columns from nodeinfo to the instance table
ALTER TABLE instance
    ADD COLUMN software varchar(255);

ALTER TABLE instance
    ADD COLUMN version varchar(255);

