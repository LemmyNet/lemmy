-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


drop view person_alias_1, person_alias_2;

alter table person drop column ban_expires;
alter table community_person_ban drop column expires;

create view person_alias_1 as select * from person;
create view person_alias_2 as select * from person;
