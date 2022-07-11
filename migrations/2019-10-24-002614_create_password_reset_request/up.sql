-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


create table password_reset_request (
  id serial primary key,
  user_id int references user_ on update cascade on delete cascade not null,
  token_encrypted text not null,
  published timestamp not null default now()
);
