-- This file should undo anything in `up.sql`
alter table community drop column spoiler;
alter table post drop column spoiler;