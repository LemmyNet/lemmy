-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


alter table post 
drop column ap_id, 
drop column local;

alter table comment 
drop column ap_id, 
drop column local;
