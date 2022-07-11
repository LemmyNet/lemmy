-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


create or replace function generate_unique_changeme() 
returns text language sql 
as $$
  select 'http://changeme.invalid/' || substr(md5(random()::text), 0, 25);
$$;
