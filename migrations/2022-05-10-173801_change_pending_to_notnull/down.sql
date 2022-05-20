-- This file should undo anything in `up.sql`

alter table community_follower
  alter column pending drop not null,
  alter column pending set default false;
