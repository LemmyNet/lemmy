-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


alter table activity alter column ap_id drop not null;

create unique index idx_activity_unique_apid on activity ((data ->> 'id'::text));

drop index idx_activity_ap_id;
