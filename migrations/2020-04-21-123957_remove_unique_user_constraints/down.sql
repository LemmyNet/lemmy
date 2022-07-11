-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


-- The username index
drop index idx_user_name_lower_actor_id;
create unique index idx_user_name_lower on user_ (lower(name));

