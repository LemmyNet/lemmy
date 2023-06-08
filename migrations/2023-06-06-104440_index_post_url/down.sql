-- Change back the column type
alter table post alter column url type text;

-- Drop the index
drop index idx_post_url;
