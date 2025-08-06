ALTER TABLE post
    ADD COLUMN name_new character varying(200),
    ADD COLUMN url_new character varying(2000),
    ADD COLUMN body_new text,
    ADD COLUMN creator_id_new integer,
    ADD COLUMN community_id_new integer,
    ADD COLUMN removed_new boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN locked_new boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN published_new timestamp with time zone DEFAULT now() NOT NULL,
    ADD COLUMN updated_new timestamp with time zone,
    ADD COLUMN deleted_new boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN nsfw_new boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN embed_title_new text,
    ADD COLUMN embed_description_new text,
    ADD COLUMN thumbnail_url_new text,
    ADD COLUMN ap_id_new character varying(255),
    ADD COLUMN local_new boolean DEFAULT TRUE NOT NULL,
    ADD COLUMN embed_video_url_new text,
    ADD COLUMN language_id_new integer DEFAULT 0 NOT NULL,
    ADD COLUMN featured_community_new boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN featured_local_new boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN url_content_type_new text,
    ADD COLUMN alt_text_new text,
    ADD COLUMN scheduled_publish_time_new timestamp with time zone,
    ADD COLUMN comments_new int DEFAULT 0 NOT NULL,
    ADD COLUMN score_new int DEFAULT 0 NOT NULL,
    ADD COLUMN upvotes_new int DEFAULT 0 NOT NULL,
    ADD COLUMN downvotes_new int DEFAULT 0 NOT NULL,
    ADD COLUMN newest_comment_time_necro_new timestamp with time zone DEFAULT now() NOT NULL,
    ADD COLUMN newest_comment_time_new timestamp with time zone DEFAULT now() NOT NULL,
    ADD COLUMN hot_rank_new double precision DEFAULT 0.0001 NOT NULL,
    ADD COLUMN hot_rank_active_new double precision DEFAULT 0.0001 NOT NULL,
    ADD COLUMN controversy_rank_new double precision DEFAULT 0 NOT NULL,
    -- Old column here
    ADD COLUMN instance_id integer,
    ADD COLUMN scaled_rank_new double precision DEFAULT 0.0001 NOT NULL,
    ADD COLUMN report_count_new smallint DEFAULT 0 NOT NULL,
    ADD COLUMN unresolved_report_count_new smallint DEFAULT 0 NOT NULL,
    ADD COLUMN federation_pending_new boolean DEFAULT FALSE NOT NULL;

UPDATE
    post
SET
    (instance_id,
        name_new,
        url_new,
        body_new,
        creator_id_new,
        community_id_new,
        removed_new,
        locked_new,
        published_new,
        updated_new,
        deleted_new,
        nsfw_new,
        embed_title_new,
        embed_description_new,
        thumbnail_url_new,
        ap_id_new,
        local_new,
        embed_video_url_new,
        language_id_new,
        featured_community_new,
        featured_local_new,
        url_content_type_new,
        alt_text_new,
        scheduled_publish_time_new,
        comments_new,
        score_new,
        upvotes_new,
        downvotes_new,
        newest_comment_time_necro_new,
        newest_comment_time_new,
        hot_rank_new,
        hot_rank_active_new,
        controversy_rank_new,
        scaled_rank_new,
        report_count_new,
        unresolved_report_count_new,
        federation_pending_new) = (0,
        name,
        url,
        body,
        creator_id,
        community_id,
        removed,
        LOCKED,
        published,
        updated,
        deleted,
        nsfw,
        embed_title,
        embed_description,
        thumbnail_url,
        ap_id,
        local,
        embed_video_url,
        language_id,
        featured_community,
        featured_local,
        url_content_type,
        alt_text,
        scheduled_publish_time,
        comments,
        score,
        upvotes,
        downvotes,
        newest_comment_time_necro,
        newest_comment_time,
        hot_rank,
        hot_rank_active,
        controversy_rank,
        scaled_rank,
        report_count,
        unresolved_report_count,
        federation_pending);

ALTER TABLE post
    DROP COLUMN name,
    DROP COLUMN url,
    DROP COLUMN body,
    DROP COLUMN creator_id,
    DROP COLUMN community_id,
    DROP COLUMN removed,
    DROP COLUMN LOCKED,
    DROP COLUMN published,
    DROP COLUMN updated,
    DROP COLUMN deleted,
    DROP COLUMN nsfw,
    DROP COLUMN embed_title,
    DROP COLUMN embed_description,
    DROP COLUMN thumbnail_url,
    DROP COLUMN ap_id,
    DROP COLUMN local,
    DROP COLUMN embed_video_url,
    DROP COLUMN language_id,
    DROP COLUMN featured_community,
    DROP COLUMN featured_local,
    DROP COLUMN url_content_type,
    DROP COLUMN alt_text,
    DROP COLUMN scheduled_publish_time,
    DROP COLUMN comments,
    DROP COLUMN score,
    DROP COLUMN upvotes,
    DROP COLUMN downvotes,
    DROP COLUMN newest_comment_time_necro,
    DROP COLUMN newest_comment_time,
    DROP COLUMN hot_rank,
    DROP COLUMN hot_rank_active,
    DROP COLUMN controversy_rank,
    DROP COLUMN scaled_rank,
    DROP COLUMN report_count,
    DROP COLUMN unresolved_report_count,
    DROP COLUMN federation_pending;

ALTER TABLE post RENAME COLUMN name_new TO name;

ALTER TABLE post RENAME COLUMN url_new TO url;

ALTER TABLE post RENAME COLUMN body_new TO body;

ALTER TABLE post RENAME COLUMN creator_id_new TO creator_id;

ALTER TABLE post RENAME COLUMN community_id_new TO community_id;

ALTER TABLE post RENAME COLUMN removed_new TO removed;

ALTER TABLE post RENAME COLUMN locked_new TO LOCKED;

ALTER TABLE post RENAME COLUMN published_new TO published;

ALTER TABLE post RENAME COLUMN updated_new TO updated;

ALTER TABLE post RENAME COLUMN deleted_new TO deleted;

ALTER TABLE post RENAME COLUMN nsfw_new TO nsfw;

ALTER TABLE post RENAME COLUMN embed_title_new TO embed_title;

ALTER TABLE post RENAME COLUMN embed_description_new TO embed_description;

ALTER TABLE post RENAME COLUMN thumbnail_url_new TO thumbnail_url;

ALTER TABLE post RENAME COLUMN ap_id_new TO ap_id;

ALTER TABLE post RENAME COLUMN local_new TO local;

ALTER TABLE post RENAME COLUMN embed_video_url_new TO embed_video_url;

ALTER TABLE post RENAME COLUMN language_id_new TO language_id;

ALTER TABLE post RENAME COLUMN featured_community_new TO featured_community;

ALTER TABLE post RENAME COLUMN featured_local_new TO featured_local;

ALTER TABLE post RENAME COLUMN url_content_type_new TO url_content_type;

ALTER TABLE post RENAME COLUMN alt_text_new TO alt_text;

ALTER TABLE post RENAME COLUMN scheduled_publish_time_new TO scheduled_publish_time;

ALTER TABLE post RENAME COLUMN comments_new TO comments;

ALTER TABLE post RENAME COLUMN score_new TO score;

ALTER TABLE post RENAME COLUMN upvotes_new TO upvotes;

ALTER TABLE post RENAME COLUMN downvotes_new TO downvotes;

ALTER TABLE post RENAME COLUMN newest_comment_time_necro_new TO newest_comment_time_necro;

ALTER TABLE post RENAME COLUMN newest_comment_time_new TO newest_comment_time;

ALTER TABLE post RENAME COLUMN hot_rank_new TO hot_rank;

ALTER TABLE post RENAME COLUMN hot_rank_active_new TO hot_rank_active;

ALTER TABLE post RENAME COLUMN controversy_rank_new TO controversy_rank;

ALTER TABLE post RENAME COLUMN scaled_rank_new TO scaled_rank;

ALTER TABLE post RENAME COLUMN report_count_new TO report_count;

ALTER TABLE post RENAME COLUMN unresolved_report_count_new TO unresolved_report_count;

ALTER TABLE post RENAME COLUMN federation_pending_new TO federation_pending;

-- Update the historical instance_id rows
UPDATE
    post AS p
SET
    instance_id = c.instance_id
FROM
    community AS c
WHERE
    p.community_id = c.id;

ALTER TABLE ONLY post
    ADD CONSTRAINT idx_post_ap_id UNIQUE (ap_id);

CREATE INDEX idx_post_community ON post USING btree (community_id);

CREATE INDEX idx_post_community_active ON post USING btree (community_id, featured_local DESC, hot_rank_active DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_controversy ON post USING btree (community_id, featured_local DESC, controversy_rank DESC, id DESC);

CREATE INDEX idx_post_community_hot ON post USING btree (community_id, featured_local DESC, hot_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_most_comments ON post USING btree (community_id, featured_local DESC, comments DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_newest_comment_time ON post USING btree (community_id, featured_local DESC, newest_comment_time DESC, id DESC);

CREATE INDEX idx_post_community_newest_comment_time_necro ON post USING btree (community_id, featured_local DESC, newest_comment_time_necro DESC, id DESC);

CREATE INDEX idx_post_community_scaled ON post USING btree (community_id, featured_local DESC, scaled_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_score ON post USING btree (community_id, featured_local DESC, score DESC, published DESC, id DESC);

CREATE INDEX idx_post_creator ON post USING btree (creator_id);

CREATE INDEX idx_post_featured_community_active ON post USING btree (community_id, featured_community DESC, hot_rank_active DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_controversy ON post USING btree (community_id, featured_community DESC, controversy_rank DESC, id DESC);

CREATE INDEX idx_post_featured_community_hot ON post USING btree (community_id, featured_community DESC, hot_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_most_comments ON post USING btree (community_id, featured_community DESC, comments DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_newest_comment_time ON post USING btree (community_id, featured_community DESC, newest_comment_time DESC, id DESC);

CREATE INDEX idx_post_featured_community_newest_comment_time_necr ON post USING btree (community_id, featured_community DESC, newest_comment_time_necro DESC, id DESC);

CREATE INDEX idx_post_featured_community_published_asc ON post USING btree (community_id, featured_community DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_featured_community_scaled ON post USING btree (community_id, featured_community DESC, scaled_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_score ON post USING btree (community_id, featured_community DESC, score DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_active ON post USING btree (featured_local DESC, hot_rank_active DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_controversy ON post USING btree (featured_local DESC, controversy_rank DESC, id DESC);

CREATE INDEX idx_post_featured_local_hot ON post USING btree (featured_local DESC, hot_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_most_comments ON post USING btree (featured_local DESC, comments DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_newest_comment_time ON post USING btree (featured_local DESC, newest_comment_time DESC, id DESC);

CREATE INDEX idx_post_featured_local_newest_comment_time_necro ON post USING btree (featured_local DESC, newest_comment_time_necro DESC, id DESC);

CREATE INDEX idx_post_featured_local_published ON post USING btree (featured_local DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_published_asc ON post USING btree (featured_local DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_featured_local_scaled ON post USING btree (featured_local DESC, scaled_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_score ON post USING btree (featured_local DESC, score DESC, published DESC, id DESC);

CREATE INDEX idx_post_language ON post USING btree (language_id);

CREATE INDEX idx_post_nonzero_hotrank ON post USING btree (published DESC)
WHERE ((hot_rank <> (0)::double precision) OR (hot_rank_active <> (0)::double precision));

CREATE INDEX idx_post_published ON post USING btree (published);

CREATE INDEX idx_post_published_asc ON post USING btree (reverse_timestamp_sort (published) DESC);

CREATE INDEX idx_post_scheduled_publish_time ON post USING btree (scheduled_publish_time);

CREATE INDEX idx_post_trigram ON post USING gin (name gin_trgm_ops, body gin_trgm_ops, alt_text gin_trgm_ops);

CREATE INDEX idx_post_url ON post USING btree (url);

CREATE INDEX idx_post_url_content_type ON post USING gin (url_content_type gin_trgm_ops);

ALTER TABLE ONLY post
    ADD CONSTRAINT post_community_id_fkey FOREIGN KEY (community_id) REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY post
    ADD CONSTRAINT post_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY post
    ADD CONSTRAINT post_language_id_fkey FOREIGN KEY (language_id) REFERENCES LANGUAGE (id);

ALTER TABLE ONLY post
    ADD CONSTRAINT post_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE post
    ALTER COLUMN name SET NOT NULL;

ALTER TABLE post
    ALTER COLUMN creator_id SET NOT NULL;

ALTER TABLE post
    ALTER COLUMN community_id SET NOT NULL;

ALTER TABLE post
    ALTER COLUMN ap_id SET NOT NULL;

