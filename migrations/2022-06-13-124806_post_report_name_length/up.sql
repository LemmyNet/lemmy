-- adjust length limit to match post.name
ALTER TABLE post_report
    ALTER COLUMN original_post_name TYPE varchar(200);

