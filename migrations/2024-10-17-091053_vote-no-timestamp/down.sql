alter table post_like add column published timestamptz not null default now();
alter table comment_like add column published timestamptz not null default now();