ALTER TYPE community_visibility
    ADD value 'Private';

-- Change `community_follower.pending` to `state` enum
create type community_follower_state as enum ('Accepted','Pending','ApprovalRequired');
alter table community_follower alter column pending drop default;
create or replace function convert_follower_state(b bool)
returns community_follower_state language sql as $$
    select case
    when b = true then 'Pending'::community_follower_state
    else 'Accepted'::community_follower_state
    end
$$;
alter table community_follower alter column pending type community_follower_state 
    using convert_follower_state(pending);
drop function convert_follower_state;
alter table community_follower rename column pending to state;