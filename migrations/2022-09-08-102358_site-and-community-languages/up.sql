create table site_language (
  id serial primary key,
  site_id int references site on update cascade on delete cascade not null,
  language_id int references language on update cascade on delete cascade not null,
  unique (site_id, language_id)
);

create table community_language (
  id serial primary key,
  community_id int references community on update cascade on delete cascade not null,
  language_id int references language on update cascade on delete cascade not null,
  unique (community_id, language_id)
);

-- existing users get all languages enabled
do $$
    declare
        uid integer;
begin
    for uid in select id from local_user
    loop
        insert into local_user_language (local_user_id, language_id)
        (select uid, language.id as lid from language);
    end loop;
end;
$$;

-- existing sites get all languages enabled
do $$
    declare
        sid integer;
begin
    for sid in select id from site
    loop
        insert into site_language (site_id, language_id)
        (select sid, language.id as lid from language);
    end loop;
end;
$$;


-- existing communities get all languages enabled
do $$
    declare
        cid integer;
begin
    for cid in select id from community
    loop
        insert into community_language (community_id, language_id)
        (select cid, language.id as lid from language);
    end loop;
end;
$$;
