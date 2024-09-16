ALTER TABLE comment
    ALTER COLUMN ap_id SET DEFAULT generate_unique_changeme ();

ALTER TABLE post
    ALTER COLUMN ap_id SET DEFAULT generate_unique_changeme ();

ALTER TABLE private_message
    ALTER COLUMN ap_id SET DEFAULT generate_unique_changeme ();

