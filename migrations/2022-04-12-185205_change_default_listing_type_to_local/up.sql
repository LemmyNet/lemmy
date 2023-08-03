-- 0 is All, 1 is Local, 2 is Subscribed
ALTER TABLE ONLY local_user
    ALTER COLUMN default_listing_type SET DEFAULT 1;

