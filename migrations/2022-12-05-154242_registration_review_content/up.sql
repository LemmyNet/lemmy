-- add new registration mode
alter type registration_mode_enum add value 'review_content';

-- add table for comment review queue
create table review_comment (
  id            serial    primary key,
  comment_id    int       unique references comment on update cascade on delete cascade not null,
  approved      bool      not null default false,
  approver_id   int       references local_user on update cascade on delete cascade,   -- user who approved comment
  published     timestamp not null default now(),
  updated       timestamp null
);

-- rename accepted_application column so we can also use it for review_content mode
alter table local_user rename column accepted_application to approved;