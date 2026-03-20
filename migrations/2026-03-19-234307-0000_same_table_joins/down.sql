UPDATE
    notification
SET
    post_id = NULL,
    comment_id = NULL
WHERE
    modlog_id IS NOT NULL;

UPDATE
    notification
SET
    post_id = NULL
WHERE
    comment_id IS NOT NULL;

ALTER TABLE notification
    DROP COLUMN instance_id,
    DROP COLUMN community_id,
    ADD CONSTRAINT notification_check CHECK (num_nonnulls (post_id, comment_id, private_message_id, modlog_id) = 1);

ALTER TABLE comment
    DROP COLUMN community_id;

ALTER TABLE report_combined
    DROP COLUMN item_creator_id,
    DROP COLUMN report_creator_id,
    DROP COLUMN resolver_id,
    DROP COLUMN post_id,
    DROP COLUMN comment_id,
    DROP COLUMN community_id,
    DROP COLUMN private_message_id;

ALTER TABLE person_saved_combined
    DROP COLUMN community_id,
    ALTER COLUMN post_id DROP NOT NULL,
    ADD CONSTRAINT person_saved_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1),
    ADD CONSTRAINT person_saved_combined_person_id_comment_id_key UNIQUE (person_id, comment_id),
    ADD CONSTRAINT person_saved_combined_person_id_post_id_key UNIQUE (person_id, post_id);

ALTER TABLE person_liked_combined
    ALTER COLUMN post_id DROP NOT NULL;

UPDATE
    person_liked_combined
SET
    post_id = NULL
WHERE
    comment_id IS NOT NULL;

ALTER TABLE person_liked_combined
    DROP COLUMN community_id,
    ADD CONSTRAINT person_liked_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1),
    ADD CONSTRAINT person_liked_combined_person_id_comment_id_key UNIQUE (person_id, comment_id),
    ADD CONSTRAINT person_liked_combined_person_id_post_id_key UNIQUE (person_id, post_id);

ALTER TABLE person_content_combined
    ALTER COLUMN post_id DROP NOT NULL;

UPDATE
    person_content_combined
SET
    post_id = NULL
WHERE
    comment_id IS NOT NULL;

ALTER TABLE person_content_combined
    DROP COLUMN community_id,
    ADD CONSTRAINT person_content_combined_check CHECK (num_nonnulls (post_id, comment_id) = 1),
    ADD CONSTRAINT person_content_combined_comment_id_key UNIQUE (comment_id),
    ADD CONSTRAINT person_content_combined_post_id_key UNIQUE (post_id);

DROP INDEX idx_person_content_combined_post, idx_person_content_combined_comment;

