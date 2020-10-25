create table comment_report (
  id            serial    primary key,
  creator_id    int       references user_ on update cascade on delete cascade not null,   -- user reporting comment
  comment_id    int       references comment on update cascade on delete cascade not null, -- comment being reported
  comment_text  text      not null,
  reason        text      not null,
  resolved      bool      not null default false,
  resolver_id   int       references user_ on update cascade on delete cascade,   -- user resolving report
  published     timestamp not null default now(),
  updated       timestamp null,
  unique(comment_id, creator_id) -- users should only be able to report a comment once
);

create table post_report (
  id            serial    primary key,
  creator_id    int       references user_ on update cascade on delete cascade not null, -- user reporting post
  post_id       int       references post on update cascade on delete cascade not null,  -- post being reported
  post_name	    varchar(100) not null,
  post_url      text,
  post_body     text,
  reason        text      not null,
  resolved      bool      not null default false,
  resolver_id   int       references user_ on update cascade on delete cascade,   -- user resolving report
  published     timestamp not null default now(),
  updated       timestamp null,
  unique(post_id, creator_id) -- users should only be able to report a post once
);

create or replace view comment_report_view as
select cr.*,
c.post_id,
c.content as current_comment_text,
p.community_id,
f.name as creator_name,
u.id as comment_creator_id,
u.name as comment_creator_name
from comment_report cr
left join comment c on c.id = cr.comment_id
left join post p on p.id = c.post_id
left join user_ u on u.id = c.creator_id
left join user_ f on f.id = cr.creator_id;

create or replace view post_report_view as
select pr.*,
p.name as current_post_name,
p.url as current_post_url,
p.body as current_post_body,
p.community_id,
f.name as creator_name,
u.id as post_creator_id,
u.name as post_creator_name
from post_report pr
left join post p on p.id = pr.post_id
left join user_ u on u.id = p.creator_id
left join user_ f on f.id = pr.creator_id;
