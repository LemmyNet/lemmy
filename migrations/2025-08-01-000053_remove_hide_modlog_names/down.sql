-- You need to remake all the columns after the changed one.
--
-- 1. Create old column, and add _new to every one after
-- 2. Update the _new to the old
-- 3. Drop the old
-- 4. Rename the new
ALTER TABLE local_site
    ADD COLUMN hide_modlog_mod_names boolean DEFAULT TRUE NOT NULL,
    ADD COLUMN application_email_admins_new boolean DEFAULT FALSE,
    ADD COLUMN slur_filter_regex_new text,
    ADD COLUMN actor_name_max_length_new integer DEFAULT 20,
    ADD COLUMN federation_enabled_new boolean DEFAULT TRUE,
    ADD COLUMN captcha_enabled_new boolean DEFAULT FALSE,
    ADD COLUMN captcha_difficulty_new character varying(255) DEFAULT 'medium'::character varying,
    ADD COLUMN published_new timestamp with time zone DEFAULT now(),
    ADD COLUMN updated_new timestamp with time zone,
    ADD COLUMN registration_mode_new public.registration_mode_enum DEFAULT 'RequireApplication'::public.registration_mode_enum,
    ADD COLUMN reports_email_admins_new boolean DEFAULT FALSE,
    ADD COLUMN federation_signed_fetch_new boolean DEFAULT TRUE,
    ADD COLUMN default_post_listing_mode_new public.post_listing_mode_enum DEFAULT 'List'::public.post_listing_mode_enum,
    ADD COLUMN default_post_sort_type_new public.post_sort_type_enum DEFAULT 'Active'::public.post_sort_type_enum,
    ADD COLUMN default_comment_sort_type_new public.comment_sort_type_enum DEFAULT 'Hot'::public.comment_sort_type_enum,
    ADD COLUMN oauth_registration_new boolean DEFAULT FALSE,
    ADD COLUMN post_upvotes_new public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum,
    ADD COLUMN post_downvotes_new public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum,
    ADD COLUMN comment_upvotes_new public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum,
    ADD COLUMN comment_downvotes_new public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum,
    ADD COLUMN default_post_time_range_seconds_new integer,
    ADD COLUMN disallow_nsfw_content_new boolean DEFAULT FALSE,
    ADD COLUMN users_new int DEFAULT 1,
    ADD COLUMN posts_new int DEFAULT 0,
    ADD COLUMN comments_new int DEFAULT 0,
    ADD COLUMN communities_new int DEFAULT 0,
    ADD COLUMN users_active_day_new int DEFAULT 0,
    ADD COLUMN users_active_week_new int DEFAULT 0,
    ADD COLUMN users_active_month_new int DEFAULT 0,
    ADD COLUMN users_active_half_year_new int DEFAULT 0,
    ADD COLUMN disable_email_notifications_new boolean DEFAULT FALSE;

-- Update
UPDATE
    local_site
SET
    (application_email_admins_new,
        slur_filter_regex_new,
        actor_name_max_length_new,
        federation_enabled_new,
        captcha_enabled_new,
        captcha_difficulty_new,
        published_new,
        updated_new,
        registration_mode_new,
        reports_email_admins_new,
        federation_signed_fetch_new,
        default_post_listing_mode_new,
        default_post_sort_type_new,
        default_comment_sort_type_new,
        oauth_registration_new,
        post_upvotes_new,
        post_downvotes_new,
        comment_upvotes_new,
        comment_downvotes_new,
        default_post_time_range_seconds_new,
        disallow_nsfw_content_new,
        users_new,
        posts_new,
        comments_new,
        communities_new,
        users_active_day_new,
        users_active_week_new,
        users_active_month_new,
        users_active_half_year_new,
        disable_email_notifications_new) = (application_email_admins,
        slur_filter_regex,
        actor_name_max_length,
        federation_enabled,
        captcha_enabled,
        captcha_difficulty,
        published,
        updated,
        registration_mode,
        reports_email_admins,
        federation_signed_fetch,
        default_post_listing_mode,
        default_post_sort_type,
        default_comment_sort_type,
        oauth_registration,
        post_upvotes,
        post_downvotes,
        comment_upvotes,
        comment_downvotes,
        default_post_time_range_seconds,
        disallow_nsfw_content,
        users,
        posts,
        comments,
        communities,
        users_active_day,
        users_active_week,
        users_active_month,
        users_active_half_year,
        disable_email_notifications);

-- Drop
ALTER TABLE local_site
    DROP COLUMN application_email_admins,
    DROP COLUMN slur_filter_regex,
    DROP COLUMN actor_name_max_length,
    DROP COLUMN federation_enabled,
    DROP COLUMN captcha_enabled,
    DROP COLUMN captcha_difficulty,
    DROP COLUMN published,
    DROP COLUMN updated,
    DROP COLUMN registration_mode,
    DROP COLUMN reports_email_admins,
    DROP COLUMN federation_signed_fetch,
    DROP COLUMN default_post_listing_mode,
    DROP COLUMN default_post_sort_type,
    DROP COLUMN default_comment_sort_type,
    DROP COLUMN oauth_registration,
    DROP COLUMN post_upvotes,
    DROP COLUMN post_downvotes,
    DROP COLUMN comment_upvotes,
    DROP COLUMN comment_downvotes,
    DROP COLUMN default_post_time_range_seconds,
    DROP COLUMN disallow_nsfw_content,
    DROP COLUMN users,
    DROP COLUMN posts,
    DROP COLUMN comments,
    DROP COLUMN communities,
    DROP COLUMN users_active_day,
    DROP COLUMN users_active_week,
    DROP COLUMN users_active_month,
    DROP COLUMN users_active_half_year,
    DROP COLUMN disable_email_notifications;

-- Rename
ALTER TABLE local_site RENAME COLUMN application_email_admins_new TO application_email_admins;

ALTER TABLE local_site RENAME COLUMN slur_filter_regex_new TO slur_filter_regex;

ALTER TABLE local_site RENAME COLUMN actor_name_max_length_new TO actor_name_max_length;

ALTER TABLE local_site RENAME COLUMN federation_enabled_new TO federation_enabled;

ALTER TABLE local_site RENAME COLUMN captcha_enabled_new TO captcha_enabled;

ALTER TABLE local_site RENAME COLUMN captcha_difficulty_new TO captcha_difficulty;

ALTER TABLE local_site RENAME COLUMN published_new TO published;

ALTER TABLE local_site RENAME COLUMN updated_new TO updated;

ALTER TABLE local_site RENAME COLUMN registration_mode_new TO registration_mode;

ALTER TABLE local_site RENAME COLUMN reports_email_admins_new TO reports_email_admins;

ALTER TABLE local_site RENAME COLUMN federation_signed_fetch_new TO federation_signed_fetch;

ALTER TABLE local_site RENAME COLUMN default_post_listing_mode_new TO default_post_listing_mode;

ALTER TABLE local_site RENAME COLUMN default_post_sort_type_new TO default_post_sort_type;

ALTER TABLE local_site RENAME COLUMN default_comment_sort_type_new TO default_comment_sort_type;

ALTER TABLE local_site RENAME COLUMN oauth_registration_new TO oauth_registration;

ALTER TABLE local_site RENAME COLUMN post_upvotes_new TO post_upvotes;

ALTER TABLE local_site RENAME COLUMN post_downvotes_new TO post_downvotes;

ALTER TABLE local_site RENAME COLUMN comment_upvotes_new TO comment_upvotes;

ALTER TABLE local_site RENAME COLUMN comment_downvotes_new TO comment_downvotes;

ALTER TABLE local_site RENAME COLUMN default_post_time_range_seconds_new TO default_post_time_range_seconds;

ALTER TABLE local_site RENAME COLUMN disallow_nsfw_content_new TO disallow_nsfw_content;

ALTER TABLE local_site RENAME COLUMN users_new TO users;

ALTER TABLE local_site RENAME COLUMN posts_new TO posts;

ALTER TABLE local_site RENAME COLUMN comments_new TO comments;

ALTER TABLE local_site RENAME COLUMN communities_new TO communities;

ALTER TABLE local_site RENAME COLUMN users_active_day_new TO users_active_day;

ALTER TABLE local_site RENAME COLUMN users_active_week_new TO users_active_week;

ALTER TABLE local_site RENAME COLUMN users_active_month_new TO users_active_month;

ALTER TABLE local_site RENAME COLUMN users_active_half_year_new TO users_active_half_year;

ALTER TABLE local_site RENAME COLUMN disable_email_notifications_new TO disable_email_notifications;

ALTER TABLE local_site
    ALTER COLUMN application_email_admins SET NOT NULL,
    ALTER COLUMN actor_name_max_length SET NOT NULL,
    ALTER COLUMN captcha_difficulty SET NOT NULL,
    ALTER COLUMN captcha_enabled SET NOT NULL,
    ALTER COLUMN comment_downvotes SET NOT NULL,
    ALTER COLUMN comments SET NOT NULL,
    ALTER COLUMN communities SET NOT NULL,
    ALTER COLUMN comment_upvotes SET NOT NULL,
    ALTER COLUMN default_comment_sort_type SET NOT NULL,
    ALTER COLUMN default_post_listing_mode SET NOT NULL,
    ADD CONSTRAINT local_site_default_sort_type_not_null NOT NULL default_post_sort_type,
    ALTER COLUMN disable_email_notifications SET NOT NULL,
    ALTER COLUMN disallow_nsfw_content SET NOT NULL,
    ALTER COLUMN federation_enabled SET NOT NULL,
    ALTER COLUMN federation_signed_fetch SET NOT NULL,
    ALTER COLUMN oauth_registration SET NOT NULL,
    ALTER COLUMN post_downvotes SET NOT NULL,
    ALTER COLUMN post_upvotes SET NOT NULL,
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN posts SET NOT NULL,
    ALTER COLUMN users SET NOT NULL,
    ALTER COLUMN registration_mode SET NOT NULL,
    ALTER COLUMN reports_email_admins SET NOT NULL,
    ALTER COLUMN users_active_day SET NOT NULL,
    ALTER COLUMN users_active_week SET NOT NULL,
    ALTER COLUMN users_active_month SET NOT NULL,
    ALTER COLUMN users_active_half_year SET NOT NULL;

