alter table instance_actions rename column blocked_communities_at to blocked_at;

alter table instance_actions drop column blocked_persons_at;

