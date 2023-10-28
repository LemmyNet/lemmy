ALTER TABLE captcha_answer
    DROP COLUMN id,
    ADD PRIMARY KEY (uuid),
    DROP CONSTRAINT captcha_answer_uuid_key;

ALTER TABLE comment_aggregates
    DROP COLUMN id,
    ADD PRIMARY KEY (comment_id),
    DROP CONSTRAINT comment_aggregates_comment_id_key;

ALTER TABLE comment_like
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id, comment_id),
    DROP CONSTRAINT comment_like_comment_id_person_id_key;

DROP INDEX idx_comment_like_person;

ALTER TABLE comment_saved
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id, comment_id),
    DROP CONSTRAINT comment_saved_comment_id_person_id_key;

DROP INDEX idx_comment_saved_person_id;

ALTER TABLE community_aggregates
    DROP COLUMN id,
    ADD PRIMARY KEY (community_id),
    DROP CONSTRAINT community_aggregates_community_id_key;

ALTER TABLE person_aggregates
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id),
    DROP CONSTRAINT person_aggregates_person_id_key;

ALTER TABLE post_aggregates
    DROP COLUMN id,
    ADD PRIMARY KEY (post_id),
    DROP CONSTRAINT post_aggregates_post_id_key;

ALTER TABLE post_saved
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id, post_id),
    DROP CONSTRAINT post_saved_post_id_person_id_key;

DROP INDEX idx_post_saved_person_id;

\

ALTER TABLE site_aggregates
    DROP COLUMN id,
    ADD PRIMARY KEY (site_id);

