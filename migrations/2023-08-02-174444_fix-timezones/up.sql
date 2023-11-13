DROP FUNCTION IF EXISTS hot_rank CASCADE;

SET timezone = 'UTC';

--  Allow ALTER TABLE ... SET DATA TYPE changing between timestamp and timestamptz to avoid a table rewrite when the session time zone is UTC (Noah Misch)
-- In the UTC time zone, these two data types are binary compatible.
ALTER TABLE community_moderator
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE community_follower
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE person_ban
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE community_person_ban
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE community_person_ban
    ALTER COLUMN expires TYPE timestamptz
    USING expires;

ALTER TABLE person
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE person
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE person
    ALTER COLUMN last_refreshed_at TYPE timestamptz
    USING last_refreshed_at;

ALTER TABLE person
    ALTER COLUMN ban_expires TYPE timestamptz
    USING ban_expires;

ALTER TABLE post_like
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE post_saved
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE post_read
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE comment_like
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE comment_saved
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE comment
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE comment
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE mod_remove_post
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE mod_lock_post
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE mod_remove_comment
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE mod_remove_community
    ALTER COLUMN expires TYPE timestamptz
    USING expires;

ALTER TABLE mod_remove_community
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE mod_ban_from_community
    ALTER COLUMN expires TYPE timestamptz
    USING expires;

ALTER TABLE mod_ban_from_community
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE mod_ban
    ALTER COLUMN expires TYPE timestamptz
    USING expires;

ALTER TABLE mod_ban
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE mod_add_community
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE mod_add
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE person_mention
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE mod_feature_post
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE password_reset_request
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE private_message
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE private_message
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE sent_activity
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE received_activity
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE community
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE community
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE community
    ALTER COLUMN last_refreshed_at TYPE timestamptz
    USING last_refreshed_at;

ALTER TABLE post
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE post
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE comment_report
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE comment_report
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE post_report
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE post_report
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE post_aggregates
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE post_aggregates
    ALTER COLUMN newest_comment_time_necro TYPE timestamptz
    USING newest_comment_time_necro;

ALTER TABLE post_aggregates
    ALTER COLUMN newest_comment_time TYPE timestamptz
    USING newest_comment_time;

ALTER TABLE comment_aggregates
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE community_block
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE community_aggregates
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE mod_transfer_community
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE person_block
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE local_user
    ALTER COLUMN validator_time TYPE timestamptz
    USING validator_time;

ALTER TABLE admin_purge_person
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE email_verification
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE admin_purge_community
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE admin_purge_post
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE admin_purge_comment
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE registration_application
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE mod_hide_community
    ALTER COLUMN when_ TYPE timestamptz
    USING when_;

ALTER TABLE site
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE site
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE site
    ALTER COLUMN last_refreshed_at TYPE timestamptz
    USING last_refreshed_at;

ALTER TABLE comment_reply
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE person_post_aggregates
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE private_message_report
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE private_message_report
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE local_site
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE local_site
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE federation_allowlist
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE federation_allowlist
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE federation_blocklist
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE federation_blocklist
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE local_site_rate_limit
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE local_site_rate_limit
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE person_follower
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE tagline
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE tagline
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE custom_emoji
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE custom_emoji
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE instance
    ALTER COLUMN published TYPE timestamptz
    USING published;

ALTER TABLE instance
    ALTER COLUMN updated TYPE timestamptz
    USING updated;

ALTER TABLE captcha_answer
    ALTER COLUMN published TYPE timestamptz
    USING published;

-- same as before just with time zone argument
CREATE OR REPLACE FUNCTION hot_rank (score numeric, published timestamp with time zone)
    RETURNS integer
    AS $$
DECLARE
    hours_diff numeric := EXTRACT(EPOCH FROM (now() - published)) / 3600;
BEGIN
    IF (hours_diff > 0) THEN
        RETURN floor(10000 * log(greatest (1, score + 3)) / power((hours_diff + 2), 1.8))::integer;
    ELSE
        -- if the post is from the future, set hot score to 0. otherwise you can game the post to
        -- always be on top even with only 1 vote by setting it to the future
        RETURN 0;
    END IF;
END;
$$
LANGUAGE plpgsql
IMMUTABLE PARALLEL SAFE;

