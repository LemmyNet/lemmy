create table user_mention (
  id serial primary key,
  recipient_id int references user_ on update cascade on delete cascade not null,
  comment_id int references comment on update cascade on delete cascade not null,
  read boolean default false not null,
  published timestamp not null default now(),
  unique(recipient_id, comment_id)
);

create view user_mention_view as
select 
    c.id,
    um.id as user_mention_id,
    c.creator_id,
    c.post_id,
    c.parent_id,
    c.content,
    c.removed,
    um.read,
    c.published,
    c.updated,
    c.deleted,
    c.community_id,
    c.banned,
    c.banned_from_community,
    c.creator_name,
    c.score,
    c.upvotes,
    c.downvotes,
    c.user_id,
    c.my_vote,
    c.saved,
    um.recipient_id
from user_mention um, comment_view c
where um.comment_id = c.id;
