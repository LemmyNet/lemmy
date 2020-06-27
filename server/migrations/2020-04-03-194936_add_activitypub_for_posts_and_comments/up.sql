-- Add federation columns to post, comment

alter table post
-- TODO uniqueness constraints should be added on these 3 columns later
add column ap_id character varying(255) not null default 'http://fake.com', -- This needs to be checked and updated in code, building from the site url if local
add column local boolean not null default true
;

alter table comment
-- TODO uniqueness constraints should be added on these 3 columns later
add column ap_id character varying(255) not null default 'http://fake.com', -- This needs to be checked and updated in code, building from the site url if local
add column local boolean not null default true
;

