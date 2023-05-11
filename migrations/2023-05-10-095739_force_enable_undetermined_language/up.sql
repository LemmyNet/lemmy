-- force enable undetermined language for all users
insert into local_user_language (local_user_id, language_id)
    select id, 0 from local_user
    on conflict (local_user_id, language_id) do nothing;
