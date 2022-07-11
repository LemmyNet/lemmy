-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


-- 0 is All, 1 is Local, 2 is Subscribed

alter table only local_user alter column default_listing_type set default 2;
