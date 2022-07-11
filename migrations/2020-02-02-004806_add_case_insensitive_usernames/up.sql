-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


-- Add case insensitive username and email uniqueness

-- An example of showing the dupes:
-- select
--   max(id) as id,
--   lower(name) as lname,
--   count(*)
-- from user_
-- group by lower(name)
-- having count(*) > 1;

-- Delete username dupes, keeping the first one
delete
from user_
where id not in (
  select min(id)
  from user_
  group by lower(name), lower(fedi_name)
);

-- The user index 
create unique index idx_user_name_lower on user_ (lower(name));

-- Email lower
create unique index idx_user_email_lower on user_ (lower(email));

-- Set empty emails properly to null
update user_ set email = null where email = '';

