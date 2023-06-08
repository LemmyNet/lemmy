-- Make a hard limit of 512 for the post.url column
-- Truncate existing long rows.
update post set url = left(url, 512) where length(url) > 512;

-- Enforce the limit
alter table post alter column url type varchar (512);

-- Add the index
create index idx_post_url on post(url);
