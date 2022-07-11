-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


alter table site_aggregates 
  drop column users_active_day,
  drop column users_active_week,
  drop column users_active_month,
  drop column users_active_half_year;

alter table community_aggregates 
  drop column users_active_day,
  drop column users_active_week,
  drop column users_active_month,
  drop column users_active_half_year;

drop function site_aggregates_activity(i text);
drop function community_aggregates_activity(i text);
