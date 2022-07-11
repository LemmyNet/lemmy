-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


-- Renaming description to sidebar
alter table site rename column description to sidebar;

-- Adding a short description column
alter table site add column description varchar(150);
