-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


alter table user_ add column default_sort_type smallint default 0 not null;
alter table user_ add column default_listing_type smallint default 1 not null;
