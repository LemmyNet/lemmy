-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


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

