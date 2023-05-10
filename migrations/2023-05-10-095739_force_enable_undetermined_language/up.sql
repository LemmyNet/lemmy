-- force enable undetermined language for all users
insert into local_user_language (local_user_id, language_id)
    select id, 0 from (
        select id from local_user where id not in (
            select local_user_id fromlocal_user_language where language_id = 0
        )
    ) as foo;