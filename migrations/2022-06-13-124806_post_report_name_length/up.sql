-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only

-- adjust length limit to match post.name
alter table post_report alter column original_post_name type varchar(200);
