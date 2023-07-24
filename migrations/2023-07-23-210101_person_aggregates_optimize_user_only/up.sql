CREATE OR REPLACE FUNCTION  person_aggregates_comment_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
        update person_aggregates
        set comment_count = comment_count + 1
        where person_id = NEW.creator_id;
    ELSIF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
        update person_aggregates
        set comment_count = comment_count - 1
        where person_id = OLD.creator_id;

        -- If the comment gets deleted, the person's score calculation trigger won't fire,
        -- so you need to re-calculate
        update person_aggregates ua
        set comment_score = cd.score
        from (
                 -- "cl" alias stands for "comment_like", when query has been revised to pull score from comment_aggregates
                 select u.id,
                        coalesce(0, sum(cl.score)) as score
                        -- User join because comments could be empty
                 from person u
                          left join comment c on u.id = c.creator_id and c.deleted = 'f' and c.removed = 'f'
                          left join comment_aggregates cl on c.id = cl.comment_id
                 where u.id = OLD.creator_id
                 group by u.id
             ) cd
        where ua.person_id = OLD.creator_id;
    END IF;
    return null;
end $$;


CREATE OR REPLACE FUNCTION  person_aggregates_post_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
    IF (was_restored_or_created(TG_OP, OLD, NEW)) THEN
        update person_aggregates
        set post_count = post_count + 1 where person_id = NEW.creator_id;

    ELSIF (was_removed_or_deleted(TG_OP, OLD, NEW)) THEN
        update person_aggregates
        set post_count = post_count - 1 where person_id = OLD.creator_id;

        -- If the post gets deleted, the person's post score calculation trigger won't fire,
        -- so you need to re-calculate
        update person_aggregates ua
        set post_score = pd.score
        from (
                 -- "pl" alias stands for "post_like", when query has been revised to pull score from post_aggregates
                 select u.id,
                        coalesce(0, sum(pl.score)) as score
                        -- User join because posts could be empty
                 from person u
                          left join post p on u.id = p.creator_id and p.deleted = 'f' and p.removed = 'f'
                          left join post_aggregates pl on p.id = pl.post_id
                 where u.id = OLD.creator_id
                 group by u.id
             ) pd
        where ua.person_id = OLD.creator_id;

    END IF;
    return null;
end $$;
