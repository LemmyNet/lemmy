-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


ALTER TABLE community DROP COLUMN followers_url;
ALTER TABLE community DROP COLUMN inbox_url;
ALTER TABLE community DROP COLUMN shared_inbox_url;

ALTER TABLE user_ DROP COLUMN inbox_url;
ALTER TABLE user_ DROP COLUMN shared_inbox_url;
