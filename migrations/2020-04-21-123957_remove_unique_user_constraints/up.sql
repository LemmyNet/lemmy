-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only

drop index idx_user_name_lower;
create unique index idx_user_name_lower_actor_id on user_ (lower(name), lower(actor_id));
