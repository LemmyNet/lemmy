-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


-- Add the column
alter table site add column enable_downvotes boolean default true not null;
alter table site add column open_registration boolean default true not null;
alter table site add column enable_nsfw boolean default true not null;

-- Reload the view
drop view site_view;

create view site_view as 
select *,
(select name from user_ u where s.creator_id = u.id) as creator_name,
(select count(*) from user_) as number_of_users,
(select count(*) from post) as number_of_posts,
(select count(*) from comment) as number_of_comments,
(select count(*) from community) as number_of_communities
from site s;
