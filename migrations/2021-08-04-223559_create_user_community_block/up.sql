-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


create table person_block (
  id serial primary key,
  person_id int references person on update cascade on delete cascade not null,
  target_id int references person on update cascade on delete cascade not null,
  published timestamp not null default now(),
  unique(person_id, target_id)
);

create table community_block (
  id serial primary key,
  person_id int references person on update cascade on delete cascade not null,
  community_id int references community on update cascade on delete cascade not null,
  published timestamp not null default now(),
  unique(person_id, community_id)
);
