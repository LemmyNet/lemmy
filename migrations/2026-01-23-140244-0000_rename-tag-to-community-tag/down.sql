ALTER TABLE post_community_tag RENAME TO post_tag;

ALTER TABLE post_tag RENAME community_tag_id TO tag_id;

ALTER TABLE community_tag RENAME TO tag;

