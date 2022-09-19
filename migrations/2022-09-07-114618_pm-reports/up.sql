create table private_message_report (
  id            serial    primary key,
  creator_id    int       references person on update cascade on delete cascade not null,   -- user reporting comment
  private_message_id    int       references private_message on update cascade on delete cascade not null, -- comment being reported
  original_pm_text  text      not null,
  reason        text      not null,
  resolved      bool      not null default false,
  resolver_id   int       references person on update cascade on delete cascade,   -- user resolving report
  published     timestamp not null default now(),
  updated       timestamp null,
  unique(private_message_id, creator_id) -- users should only be able to report a pm once
);
