create type post_view_mode_enum as enum ('List', 'Card', 'SmallCard');

alter table local_user add column post_view_mode post_view_mode_enum default 'List' not null;
