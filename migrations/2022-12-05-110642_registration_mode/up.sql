-- create enum for registration modes
create type registration_mode_enum as enum
    ('closed', 'require_application', 'open');

-- use this enum for registration mode setting
alter table local_site add column
    registration_mode registration_mode_enum not null default 'require_application';

-- generate registration mode value from previous settings
with subquery as (
    select open_registration, require_application,
        case
            when open_registration=false then 'closed'::registration_mode_enum
            when open_registration=true and require_application=true then 'require_application'
            else 'open'
        end
    from local_site
)
update local_site
set registration_mode = subquery.case
from subquery;

-- drop old registration settings
alter table local_site drop column open_registration;
alter table local_site drop column require_application;
