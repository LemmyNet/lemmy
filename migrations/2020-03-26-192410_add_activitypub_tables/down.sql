-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


drop table activity;

alter table user_ 
drop column actor_id, 
drop column private_key,
drop column public_key,
drop column bio,
drop column local,
drop column last_refreshed_at;

alter table community 
drop column actor_id, 
drop column private_key,
drop column public_key,
drop column local,
drop column last_refreshed_at;
