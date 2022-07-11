-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only

-- generate a jwt secret
create extension if not exists pgcrypto;

create table secret(
  id serial primary key,
  jwt_secret varchar not null default gen_random_uuid()
);

insert into secret default values;
