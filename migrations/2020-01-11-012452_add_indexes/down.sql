-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


drop index idx_post_creator;
drop index idx_post_community;

drop index idx_post_like_post;
drop index idx_post_like_user;

drop index idx_comment_creator;
drop index idx_comment_parent;
drop index idx_comment_post;

drop index idx_comment_like_comment;
drop index idx_comment_like_user;
drop index idx_comment_like_post;

drop index idx_community_creator;
drop index idx_community_category;
