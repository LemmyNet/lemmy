-- Remove private visibility
ALTER TYPE community_visibility RENAME TO community_visibility__;

CREATE TYPE community_visibility AS enum (
    'Public',
    'LocalOnly'
);

ALTER TABLE community
    ALTER COLUMN visibility DROP DEFAULT;

ALTER TABLE community
    ALTER COLUMN visibility TYPE community_visibility
    USING visibility::text::community_visibility;

ALTER TABLE community
    ALTER COLUMN visibility SET DEFAULT 'Public';

DROP TYPE community_visibility__;

-- Revert community follower changes
create or replace function convert_follower_state(s community_follower_state)
returns bool language sql as $$
    select case
    when s = 'Pending' then true
    else false
    end
$$;
alter table community_follower alter column state type bool 
    using convert_follower_state(state);
drop function convert_follower_state;
alter table community_follower alter column state set default false;
alter table community_follower rename column state to pending;
drop type community_follower_state;