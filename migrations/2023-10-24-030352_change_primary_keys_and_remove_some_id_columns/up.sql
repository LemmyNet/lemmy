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

ALTER TABLE community_block
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id, community_id),
    DROP CONSTRAINT community_block_person_id_community_id_key;

ALTER TABLE community_moderator
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id, community_id),
    DROP CONSTRAINT community_moderator_community_id_person_id_key;

ALTER TABLE community_person_ban
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id, community_id),
    DROP CONSTRAINT community_person_ban_community_id_person_id_key;

ALTER TABLE custom_emoji_keyword
    DROP COLUMN id,
    ADD PRIMARY KEY (custom_emoji_id, keyword),
    DROP CONSTRAINT custom_emoji_keyword_custom_emoji_id_keyword_key;

ALTER TABLE federation_allowlist
    DROP COLUMN id,
    ADD PRIMARY KEY (instance_id),
    DROP CONSTRAINT federation_allowlist_instance_id_key;

ALTER TABLE federation_blocklist
    DROP COLUMN id,
    ADD PRIMARY KEY (instance_id),
    DROP CONSTRAINT federation_blocklist_instance_id_key;

ALTER TABLE person_aggregates
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id),
    DROP CONSTRAINT person_aggregates_person_id_key;

ALTER TABLE person_post_aggregates
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id, post_id),
    DROP CONSTRAINT person_post_aggregates_person_id_post_id_key;

ALTER TABLE post_aggregates
    DROP COLUMN id,
    ADD PRIMARY KEY (post_id),
    DROP CONSTRAINT post_aggregates_post_id_key;

ALTER TABLE post_saved
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id, post_id),
    DROP CONSTRAINT post_saved_post_id_person_id_key;

DROP INDEX idx_post_saved_person_id;

-- Delete duplicates which can exist because of missing `UNIQUE` constraint
DELETE FROM site_aggregates AS a
    USING (
        SELECT
            min(id) AS id,
            site_id
        FROM site_aggregates
        GROUP BY site_id
        HAVING count(*) > 1
    ) AS b
    WHERE a.site_id = b.site_id AND a.id != b.id;

ALTER TABLE site_aggregates
    DROP COLUMN id,
    ADD PRIMARY KEY (site_id);

