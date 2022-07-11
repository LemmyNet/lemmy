-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


create index idx_comment_report_published on comment_report (published desc);
create index idx_post_report_published on post_report (published desc);
