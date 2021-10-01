-- Add the admin_purge table
-- This is just a log that shows an admin purged something from the DB.
-- This can't show any info other than *admin purged an item*
create table admin_purge (
  id serial primary key,
  admin_person_id int references person on update cascade on delete cascade not null,
  when_ timestamp not null default now()
);
