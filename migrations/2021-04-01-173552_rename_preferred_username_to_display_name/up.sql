-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


alter table person rename preferred_username to display_name;

-- Regenerate the person_alias views
drop view person_alias_1, person_alias_2;
create view person_alias_1 as select * from person;
create view person_alias_2 as select * from person;
