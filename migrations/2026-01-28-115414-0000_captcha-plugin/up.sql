DROP TABLE captcha_answer;

ALTER TABLE local_site
    DROP COLUMN captcha_enabled;

ALTER TABLE local_site
    DROP COLUMN captcha_difficulty;

