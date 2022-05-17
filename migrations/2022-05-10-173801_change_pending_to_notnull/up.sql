-- Make the pending column not null

update community_follower set pending = true where pending is null;

alter table community_follower
  alter column pending set not null,
  alter column pending drop default;

