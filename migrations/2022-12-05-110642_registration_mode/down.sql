-- add back old registration columns
alter table local_site add column open_registration boolean not null default true;
alter table local_site add column require_application boolean not null default true;

-- regenerate their values
with subquery as (
    select registration_mode,
        case
            when registration_mode='closed' then false
            else true
        end
    from local_site
)
update local_site
set open_registration = subquery.case
from subquery;
with subquery as (
    select registration_mode,
        case
            when registration_mode='open' then false
            else true
        end
    from local_site
)
update local_site
set require_application = subquery.case
from subquery;

-- drop new column and type
alter table local_site drop column registration_mode;
drop type registration_mode_enum;