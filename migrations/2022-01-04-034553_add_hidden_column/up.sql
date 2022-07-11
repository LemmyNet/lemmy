-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


alter table community add column hidden boolean default false;


create table mod_hide_community
(
    id serial primary key,
    community_id int references community on update cascade on delete cascade not null,
    mod_person_id int references person on update cascade on delete cascade not null,
    when_ timestamp not null default now(),
    reason text,
    hidden boolean default false
);

