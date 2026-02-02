ALTER TABLE tag RENAME TO community_tag;

ALTER TABLE post_tag RENAME tag_id TO community_tag_id;

ALTER TABLE post_tag RENAME TO post_community_tag;

