-- 0 is All, 1 is Local, 2 is Subscribed

alter table only local_user alter column default_listing_type set default 2;
