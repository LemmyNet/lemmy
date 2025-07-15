ALTER TABLE comment
    ADD COLUMN is_counted boolean NOT NULL DEFAULT FALSE;

ALTER TABLE comment_actions
    ADD COLUMN like_is_counted boolean,
    ADD CONSTRAINT comment_actions_check_like_is_counted CHECK ((liked_at IS NULL) = (like_is_counted IS NULL));

ALTER TABLE comment_report
    ADD COLUMN is_counted boolean NOT NULL DEFAULT FALSE;

ALTER TABLE community_actions
    ADD COLUMN follow_is_counted boolean,
    ADD CONSTRAINT community_actions_check_follow_is_counted CHECK ((followed_at IS NULL) = (follow_is_counted IS NULL));

ALTER TABLE community_report
    ADD COLUMN is_counted boolean NOT NULL DEFAULT FALSE;

ALTER TABLE post
    ADD COLUMN is_counted boolean NOT NULL DEFAULT FALSE;

ALTER TABLE post_actions
    ADD COLUMN like_is_counted boolean,
    ADD CONSTRAINT post_actions_check_like_is_counted CHECK ((liked_at IS NULL) = (like_is_counted IS NULL));

ALTER TABLE post_report
    ADD COLUMN is_counted boolean NOT NULL DEFAULT FALSE;

