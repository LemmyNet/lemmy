CREATE TYPE post_listing_mode_enum AS enum (
    'List',
    'Card',
    'SmallCard'
);

ALTER TABLE local_user
    ADD COLUMN post_listing_mode post_listing_mode_enum DEFAULT 'List' NOT NULL;

