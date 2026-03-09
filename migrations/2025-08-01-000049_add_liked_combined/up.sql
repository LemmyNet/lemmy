-- Creates combined tables for
-- person_liked: (comment, post)
-- This one is special, because you use the liked date, not the ordinary published
-- Updating the history
CREATE SEQUENCE person_liked_combined_id_seq
    AS integer START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE TABLE person_liked_combined AS
SELECT
    pa.liked,
    -- `ADD COLUMN id serial` is not used for this because it would require either putting the column at the end (might increase the amount of padding bytes) or using an `INSERT` statement (not parallelizable).
    nextval('person_liked_combined_id_seq'::regclass)::int AS id,
    pa.person_id,
    po.creator_id,
    pa.post_id,
    NULL::int AS comment_id,
    pa.vote_is_upvote
FROM
    post_actions pa,
    person p,
    post po
WHERE
    pa.liked IS NOT NULL
    AND p.local = TRUE
    AND pa.person_id = p.id
    AND pa.post_id = po.id
UNION ALL
SELECT
    ca.liked,
    nextval('person_liked_combined_id_seq'::regclass)::int,
    ca.person_id,
    co.creator_id,
    NULL::int,
    ca.comment_id,
    ca.vote_is_upvote
FROM
    comment_actions ca,
    person p,
    comment co
WHERE
    liked IS NOT NULL
    AND p.local = TRUE
    AND ca.person_id = p.id
    AND ca.comment_id = co.id;

ALTER TABLE person_liked_combined
    ALTER COLUMN id SET DEFAULT nextval('person_liked_combined_id_seq'::regclass),
    ALTER COLUMN liked SET NOT NULL,
    ALTER COLUMN vote_is_upvote SET NOT NULL,
    ALTER COLUMN person_id SET NOT NULL,
    ALTER COLUMN creator_id SET NOT NULL,
    ADD CONSTRAINT person_liked_combined_person_id_fkey FOREIGN KEY (person_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_liked_combined_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_liked_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_liked_combined_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    ADD UNIQUE (person_id, post_id),
    ADD UNIQUE (person_id, comment_id),
    ADD PRIMARY KEY (id),
    ADD CONSTRAINT person_liked_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1);

ALTER SEQUENCE person_liked_combined_id_seq OWNED BY person_liked_combined.id;

CREATE INDEX idx_person_liked_combined_person ON person_liked_combined (person_id);

CREATE INDEX idx_person_liked_combined_creator ON person_liked_combined (creator_id);

CREATE INDEX idx_person_liked_combined_person_voted_at ON person_liked_combined (person_id, liked DESC, id DESC);

