-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


-- There is no restore for this, it would require every view, table, index, etc.
-- If you want to save past this point, you should make a DB backup.

select * from user_ limit 1;
