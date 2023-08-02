-- Delete the empty public keys
DELETE FROM community
WHERE public_key IS NULL;

DELETE FROM person
WHERE public_key IS NULL;

-- Make it required
ALTER TABLE community
    ALTER COLUMN public_key SET NOT NULL;

ALTER TABLE person
    ALTER COLUMN public_key SET NOT NULL;

