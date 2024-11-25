ALTER TABLE post
    ALTER COLUMN url TYPE varchar(512);

ANALYZE post (url);

